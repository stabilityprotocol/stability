// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

//! A consensus proposer for "basic" chains which use the primitive inherent-data.

// FIXME #1021 move this into sp-consensus

use account::EthereumSigner;
use codec::Encode;
use futures::{
	channel::oneshot,
	future,
	future::{Future, FutureExt},
	select,
};
use log::{debug, error, info, trace, warn};
use sc_block_builder::{BlockBuilderApi, BlockBuilderProvider};
use sc_client_api::backend;
use sc_telemetry::{telemetry, TelemetryHandle, CONSENSUS_INFO};
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::{ApplyExtrinsicFailed::Validity, Error::ApplyExtrinsicFailed, HeaderBackend};
use sp_consensus::{DisableProofRecording, EnableProofRecording, ProofRecording, Proposal};
use sp_core::traits::SpawnNamed;
use sp_inherents::InherentData;
use sp_runtime::{
	generic::BlockId,
	traits::{BlakeTwo256, Block as BlockT, Hash as HashT, Header as HeaderT, IdentifyAccount},
	Digest, Percent, SaturatedConversion,
};
use stability_runtime::AccountId;
use stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi;
use std::{marker::PhantomData, pin::Pin, sync::Arc, time};

use prometheus_endpoint::Registry as PrometheusRegistry;
use sc_proposer_metrics::{EndProposingReason, MetricsLink as PrometheusMetrics};
use sp_core::crypto::KeyTypeId;
use sp_keystore::{Keystore, KeystorePtr};
use stbl_primitives_fee_compatible_api::CompatibleFeeApi;

/// Default block size limit in bytes used by [`Proposer`].
///
/// Can be overwritten by [`ProposerFactory::set_default_block_size_limit`].
///
/// Be aware that there is also an upper packet size on what the networking code
/// will accept. If the block doesn't fit in such a package, it can not be
/// transferred to other nodes.
pub const DEFAULT_BLOCK_SIZE_LIMIT: usize = 4 * 1024 * 1024 + 512;

const DEFAULT_SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(50);


#[derive(serde::Deserialize)]
pub struct RawZeroGasTransactionResponse {
	transactions: Vec<String>,
}

/// [`Proposer`] factory.
pub struct ProposerFactory<A, B, C, PR> {
	spawn_handle: Box<dyn SpawnNamed>,
	/// The client instance.
	client: Arc<C>,
	/// The transaction pool.
	transaction_pool: Arc<A>,

	/// Reference to Keystore
	keystore: KeystorePtr,

	/// HTTP URL of the private pool from which the node will retrieve zero-gas transactions
	zero_gas_tx_pool: Option<String>,

	/// Prometheus Link,
	metrics: PrometheusMetrics,
	/// The default block size limit.
	///
	/// If no `block_size_limit` is passed to [`sp_consensus::Proposer::propose`], this block size
	/// limit will be used.
	default_block_size_limit: usize,
	/// Soft deadline percentage of hard deadline.
	///
	/// The value is used to compute soft deadline during block production.
	/// The soft deadline indicates where we should stop attempting to add transactions
	/// to the block, which exhaust resources. After soft deadline is reached,
	/// we switch to a fixed-amount mode, in which after we see `MAX_SKIPPED_TRANSACTIONS`
	/// transactions which exhaust resrouces, we will conclude that the block is full.
	soft_deadline_percent: Percent,
	telemetry: Option<TelemetryHandle>,
	/// When estimating the block size, should the proof be included?
	include_proof_in_block_size_estimation: bool,
	/// phantom member to pin the `Backend`/`ProofRecording` type.
	_phantom: PhantomData<(B, PR)>,
}

impl<A, B, C> ProposerFactory<A, B, C, DisableProofRecording> {
	/// Create a new proposer factory.
	///
	/// Proof recording will be disabled when using proposers built by this instance to build
	/// blocks.
	pub fn new(
		spawn_handle: impl SpawnNamed + 'static,
		client: Arc<C>,
		transaction_pool: Arc<A>,
		keystore: KeystorePtr,
		zero_gas_tx_pool: Option<String>,
		prometheus: Option<&PrometheusRegistry>,
		telemetry: Option<TelemetryHandle>,
	) -> Self {
		ProposerFactory {
			spawn_handle: Box::new(spawn_handle),
			transaction_pool,
			keystore,
			zero_gas_tx_pool,
			metrics: PrometheusMetrics::new(prometheus),
			default_block_size_limit: DEFAULT_BLOCK_SIZE_LIMIT,
			soft_deadline_percent: DEFAULT_SOFT_DEADLINE_PERCENT,
			telemetry,
			client,
			include_proof_in_block_size_estimation: false,
			_phantom: PhantomData,
		}
	}
}

impl<A, B, C> ProposerFactory<A, B, C, EnableProofRecording> {
	/// Create a new proposer factory with proof recording enabled.
	///
	/// Each proposer created by this instance will record a proof while building a block.
	///
	/// This will also include the proof into the estimation of the block size. This can be disabled
	/// by calling [`ProposerFactory::disable_proof_in_block_size_estimation`].
	pub fn with_proof_recording(
		spawn_handle: impl SpawnNamed + 'static,
		client: Arc<C>,
		transaction_pool: Arc<A>,
		keystore: KeystorePtr,
		zero_gas_tx_pool: Option<String>,
		prometheus: Option<&PrometheusRegistry>,
		telemetry: Option<TelemetryHandle>,
	) -> Self {
		ProposerFactory {
			client,
			spawn_handle: Box::new(spawn_handle),
			transaction_pool,
			keystore,
			zero_gas_tx_pool,
			metrics: PrometheusMetrics::new(prometheus),
			default_block_size_limit: DEFAULT_BLOCK_SIZE_LIMIT,
			soft_deadline_percent: DEFAULT_SOFT_DEADLINE_PERCENT,
			telemetry,
			include_proof_in_block_size_estimation: true,
			_phantom: PhantomData,
		}
	}

	/// Disable the proof inclusion when estimating the block size.
	pub fn disable_proof_in_block_size_estimation(&mut self) {
		self.include_proof_in_block_size_estimation = false;
	}
}

impl<A, B, C, PR> ProposerFactory<A, B, C, PR> {
	/// Set the default block size limit in bytes.
	///
	/// The default value for the block size limit is:
	/// [`DEFAULT_BLOCK_SIZE_LIMIT`].
	///
	/// If there is no block size limit passed to [`sp_consensus::Proposer::propose`], this value
	/// will be used.
	pub fn set_default_block_size_limit(&mut self, limit: usize) {
		self.default_block_size_limit = limit;
	}

	/// Set soft deadline percentage.
	///
	/// The value is used to compute soft deadline during block production.
	/// The soft deadline indicates where we should stop attempting to add transactions
	/// to the block, which exhaust resources. After soft deadline is reached,
	/// we switch to a fixed-amount mode, in which after we see `MAX_SKIPPED_TRANSACTIONS`
	/// transactions which exhaust resrouces, we will conclude that the block is full.
	///
	/// Setting the value too low will significantly limit the amount of transactions
	/// we try in case they exhaust resources. Setting the value too high can
	/// potentially open a DoS vector, where many "exhaust resources" transactions
	/// are being tried with no success, hence block producer ends up creating an empty block.
	pub fn set_soft_deadline(&mut self, percent: Percent) {
		self.soft_deadline_percent = percent;
	}
}

impl<B, Block, C, A, PR> ProposerFactory<A, B, C, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	B: backend::Backend<Block> + Send + Sync + 'static,
	Block: BlockT,
	C: BlockBuilderProvider<B, Block, C>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
	C::Api:
		ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>> + BlockBuilderApi<Block>,
{
	fn init_with_now(
		&mut self,
		parent_header: &<Block as BlockT>::Header,
		now: Box<dyn Fn() -> time::Instant + Send + Sync>,
	) -> Proposer<B, Block, C, A, PR> {
		let parent_hash = parent_header.hash();

		info!(
			"🙌 Starting consensus session on top of parent {:?}",
			parent_hash
		);

		let proposer = Proposer::<_, _, _, _, PR> {
			spawn_handle: self.spawn_handle.clone(),
			client: self.client.clone(),
			parent_hash,
			parent_number: *parent_header.number(),
			transaction_pool: self.transaction_pool.clone(),
			keystore: self.keystore.clone(),
			zero_gas_tx_pool: self.zero_gas_tx_pool.clone(),
			now,
			metrics: self.metrics.clone(),
			default_block_size_limit: self.default_block_size_limit,
			soft_deadline_percent: self.soft_deadline_percent,
			telemetry: self.telemetry.clone(),
			_phantom: PhantomData,
			include_proof_in_block_size_estimation: self.include_proof_in_block_size_estimation,
		};

		proposer
	}
}

impl<A, B, Block, C, PR> sp_consensus::Environment<Block> for ProposerFactory<A, B, C, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	B: backend::Backend<Block> + Send + Sync + 'static,
	Block: BlockT,
	C: BlockBuilderProvider<B, Block, C>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
	C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>>
		+ BlockBuilderApi<Block>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
	PR: ProofRecording,
{
	type CreateProposer = future::Ready<Result<Self::Proposer, Self::Error>>;
	type Proposer = Proposer<B, Block, C, A, PR>;
	type Error = sp_blockchain::Error;

	fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> Self::CreateProposer {
		future::ready(Ok(
			self.init_with_now(parent_header, Box::new(time::Instant::now))
		))
	}
}

/// The proposer logic.
pub struct Proposer<B, Block: BlockT, C, A: TransactionPool, PR> {
	spawn_handle: Box<dyn SpawnNamed>,
	client: Arc<C>,
	parent_hash: Block::Hash,
	parent_number: <<Block as BlockT>::Header as HeaderT>::Number,
	transaction_pool: Arc<A>,
	keystore: KeystorePtr,
	zero_gas_tx_pool: Option<String>,
	now: Box<dyn Fn() -> time::Instant + Send + Sync>,
	metrics: PrometheusMetrics,
	default_block_size_limit: usize,
	include_proof_in_block_size_estimation: bool,
	soft_deadline_percent: Percent,
	telemetry: Option<TelemetryHandle>,
	_phantom: PhantomData<(B, PR)>,
}

impl<A, B, Block, C, PR> sp_consensus::Proposer<Block> for Proposer<B, Block, C, A, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	B: backend::Backend<Block> + Send + Sync + 'static,
	Block: BlockT,
	C: BlockBuilderProvider<B, Block, C>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
	C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>>
		+ BlockBuilderApi<Block>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
	PR: ProofRecording,
{
	type Transaction = backend::TransactionFor<B, Block>;
	type Proposal = Pin<
		Box<
			dyn Future<Output = Result<Proposal<Block, Self::Transaction, PR::Proof>, Self::Error>>
				+ Send,
		>,
	>;
	type Error = sp_blockchain::Error;
	type ProofRecording = PR;
	type Proof = PR::Proof;

	fn propose(
		self,
		inherent_data: InherentData,
		inherent_digests: Digest,
		max_duration: time::Duration,
		block_size_limit: Option<usize>,
	) -> Self::Proposal {
		let (tx, rx) = oneshot::channel();
		let spawn_handle = self.spawn_handle.clone();

		spawn_handle.spawn_blocking(
			"basic-authorship-proposer",
			None,
			Box::pin(async move {
				// leave some time for evaluation and block finalization (33%)
				let deadline = (self.now)() + max_duration - max_duration / 3;
				let res = self
					.propose_with(inherent_data, inherent_digests, deadline, block_size_limit)
					.await;
				if tx.send(res).is_err() {
					trace!("Could not send block production result to proposer!");
				}
			}),
		);

		async move { rx.await? }.boxed()
	}
}

/// If the block is full we will attempt to push at most
/// this number of transactions before quitting for real.
/// It allows us to increase block utilization.
const MAX_SKIPPED_TRANSACTIONS: usize = 8;

impl<A, B, Block, C, PR> Proposer<B, Block, C, A, PR>
where
	A: TransactionPool<Block = Block>,
	B: backend::Backend<Block> + Send + Sync + 'static,
	Block: BlockT,
	C: BlockBuilderProvider<B, Block, C>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
	C::Api: ApiExt<Block, StateBackend = backend::StateBackendFor<B, Block>>
		+ BlockBuilderApi<Block>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
	PR: ProofRecording,
{
	async fn propose_with(
		self,
		inherent_data: InherentData,
		inherent_digests: Digest,
		deadline: time::Instant,
		block_size_limit: Option<usize>,
	) -> Result<Proposal<Block, backend::TransactionFor<B, Block>, PR::Proof>, sp_blockchain::Error>
	{
		let propose_with_start = time::Instant::now();
		let mut block_builder =
			self.client
				.new_block_at(self.parent_hash, inherent_digests, PR::ENABLED)?;

		let create_inherents_start = time::Instant::now();
		let inherents = block_builder.create_inherents(inherent_data)?;
		let create_inherents_end = time::Instant::now();

		self.metrics.report(|metrics| {
			metrics.create_inherents_time.observe(
				create_inherents_end
					.saturating_duration_since(create_inherents_start)
					.as_secs_f64(),
			);
		});

		for inherent in inherents {
			match block_builder.push(inherent) {
				Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
					warn!("⚠️  Dropping non-mandatory inherent from overweight block.")
				}
				Err(ApplyExtrinsicFailed(Validity(e))) if e.was_mandatory() => {
					error!(
						"❌️ Mandatory inherent extrinsic returned error. Block cannot be produced."
					);
					return Err(ApplyExtrinsicFailed(Validity(e)));
				}
				Err(e) => {
					warn!(
						"❗️ Inherent extrinsic returned unexpected error: {}. Dropping.",
						e
					);
				}
				Ok(_) => {}
			}
		}

		// proceed with transactions
		// We calculate soft deadline used only in case we start skipping transactions.
		let now = (self.now)();
		let left = deadline.saturating_duration_since(now);
		let left_micros: u64 = left.as_micros().saturated_into();
		let soft_deadline =
		now + time::Duration::from_micros(self.soft_deadline_percent.mul_floor(left_micros));
		let block_timer = time::Instant::now();
		let mut transaction_pushed = false;
		let mut skipped = 0;
		let block_size_limit = block_size_limit.unwrap_or(self.default_block_size_limit);


		// First we try to push transactions from the zero gas transaction pool

		let raw_zero_gas_transactions_option = if let Some(zero_gas_tx_pool) = self.zero_gas_tx_pool {

			let http_client = reqwest::Client::new();
			let mut request = Box::pin(http_client.post(zero_gas_tx_pool).send().fuse());
			let mut timeout = Box::pin(futures_timer::Delay::new(std::time::Duration::from_millis(500)).fuse());
			

			let result_response_raw_zero = select! {
				res = request => {
					match res {
						Ok(response) => Ok(response),
						Err(e) => {
							error!("Error getting response from zero gas transaction pool: {}", e);
							Err("Error getting response from zero gas transaction pool")
						}
					}
				},
				_ = timeout => {
					error!(
						"Timeout fired waiting for get transaction from zero gas transaction pool"
					);
					Err("Timeout fired waiting for get transaction from zero gas transaction pool")
				},
			};
			
			match result_response_raw_zero {
				Ok(response) => {
					match response.json::<RawZeroGasTransactionResponse>().await {
						Ok(json) => Some(json),
						Err(e) => {
							error!("Error parsing JSON response from zero gas transaction pool: {}", e);
							None
						}
					}
				},
				Err(e) => {
					error!("Error getting response from zero gas transaction pool: {}", e);
					None
				}
			}
		} else {
			None
		};
		

		
		// If we pull successfully from the zero gas transaction pool, we will try to push them to the block

		if let Some(raw_zero_gas_transactions) = raw_zero_gas_transactions_option {
				let mut pending_raw_zero_gas_transactions = raw_zero_gas_transactions.transactions.into_iter();
	
			loop {
				let pending_hex_string_tx = if let Some(tx) = pending_raw_zero_gas_transactions.next() {
					tx
				} else {
					break EndProposingReason::NoMoreTransactions;
				};
	
				let now = (self.now)();
				if now > deadline {
					debug!(
						"Consensus deadline reached when pushing block transactions, \
						proceeding with proposing."
					);
					break EndProposingReason::HitDeadline;
				}
	
				let pending_raw_tx = if let Ok(pending_raw_tx) = hex::decode(pending_hex_string_tx) {
					pending_raw_tx
				}
				else {
					continue;
				};
	
				let ethereum_transaction: ethereum::TransactionV2 = ethereum::EnvelopedDecodable::decode(&pending_raw_tx).unwrap();
				

				let keys = Keystore::ecdsa_public_keys(
					&*self.keystore,
					KeyTypeId::try_from("aura").unwrap_or_default(),
				);
				

				let public = keys[0].clone().into();
				let hash = ethereum_transaction.hash();
				let hash_string = hex::encode(hash.as_bytes());


				let mut message: Vec<u8> = Vec::new();
				message.extend_from_slice(b"I consent to validate the transaction for free: 0x");
				message.extend_from_slice(hash_string.as_bytes());

				let eip191_message = stbl_tools::eth::build_eip191_message_hash(message.clone());

				let signed_hash_option = Keystore::ecdsa_sign_prehashed(
					&*self.keystore,
					KeyTypeId::try_from("aura").unwrap_or_default(),
					&public,
					&eip191_message.as_fixed_bytes(),
				).expect("Could not sign the Ethereum transaction hash");
				
				let signed_hash = if let Some(signed_hash) = signed_hash_option {
					signed_hash
				}
				else {
					continue;
				};

				let pending_tx =  if let Ok(pending_tx) = self
				.client
				.runtime_api()
				.convert_zero_gas_transaction(self.parent_hash, ethereum_transaction.clone(), signed_hash.0.to_vec()) {
					pending_tx
				}
				else {
					continue;
				};
	
	
	
				let block_size =
					block_builder.estimate_block_size(self.include_proof_in_block_size_estimation);
				
				if block_size + pending_tx.encoded_size() > block_size_limit {
					if skipped < MAX_SKIPPED_TRANSACTIONS {
						skipped += 1;
						debug!(
							"Transaction would overflow the block size limit, \
							 but will try {} more transactions before quitting.",
							MAX_SKIPPED_TRANSACTIONS - skipped,
						);
						continue;
					} else if now < soft_deadline {
						debug!(
							"Transaction would overflow the block size limit, \
							 but we still have time before the soft deadline, so \
							 we will try a bit more."
						);
						continue;
					} else {
						debug!("Reached block size limit, proceeding with proposing.");
						break EndProposingReason::HitBlockSizeLimit;
					}
				}
	
				trace!("[{:?}] Pushing to the block.", ethereum_transaction.hash());
				match sc_block_builder::BlockBuilder::push(&mut block_builder, pending_tx) {
					Ok(()) => {
						transaction_pushed = true;
						debug!("[{:?}] Pushed to the block.", ethereum_transaction.hash());
					}
					Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
						if skipped < MAX_SKIPPED_TRANSACTIONS {
							skipped += 1;
							debug!(
								"Block seems full, but will try {} more transactions before quitting.",
								MAX_SKIPPED_TRANSACTIONS - skipped,
							);
						} else if (self.now)() < soft_deadline {
							debug!(
								"Block seems full, but we still have time before the soft deadline, \
								 so we will try a bit more before quitting."
							);
						} else {
							debug!("Reached block weight limit, proceeding with proposing.");
							break EndProposingReason::HitBlockWeightLimit;
						}
					}
					Err(e) => {
						debug!("[{:?}] Invalid transaction: {}", ethereum_transaction.hash(), e);
					}
				}
			};
		}

		let mut unqueue_invalid = Vec::new();
		let mut t1 = self.transaction_pool.ready_at(self.parent_number).fuse();
		let mut t2 =
			futures_timer::Delay::new(deadline.saturating_duration_since((self.now)()) / 8).fuse();

		let mut pending_iterator = select! {
			res = t1 => res,
			_ = t2 => {
				log::warn!(
					"Timeout fired waiting for transaction pool at block #{}. \
					Proceeding with production.",
					self.parent_number,
				);
				self.transaction_pool.ready()
			},
		};


		let keys = Keystore::ecdsa_public_keys(
			&*self.keystore,
			KeyTypeId::try_from("aura").unwrap_or_default(),
		);

		let validator = EthereumSigner::from(keys[0]).into_account();

		debug!("Attempting to push transactions from the pool.");
		debug!("Pool status: {:?}", self.transaction_pool.status());

		let end_reason = loop {

			let pending_tx = if let Some(pending_tx) = pending_iterator.next()  {
				pending_tx
			} else {
				break EndProposingReason::NoMoreTransactions;
			};

			let now = (self.now)();
			if now > deadline {
				debug!(
					"Consensus deadline reached when pushing block transactions, \
					proceeding with proposing."
				);
				break EndProposingReason::HitDeadline;
			}

			let is_compatible = self
				.client
				.runtime_api()
				.is_compatible_fee(
					self.parent_hash,
					pending_tx.data().clone(),
					validator.clone(),
				)
				.unwrap();

			if !is_compatible {
				continue;
			}

			let pending_tx_data = pending_tx.data().clone();
			let pending_tx_hash = pending_tx.hash().clone();
			let block_size =
				block_builder.estimate_block_size(self.include_proof_in_block_size_estimation);
			if block_size + pending_tx_data.encoded_size() > block_size_limit {
				pending_iterator.report_invalid(&pending_tx);
				if skipped < MAX_SKIPPED_TRANSACTIONS {
					skipped += 1;
					debug!(
						"Transaction would overflow the block size limit, \
						 but will try {} more transactions before quitting.",
						MAX_SKIPPED_TRANSACTIONS - skipped,
					);
					continue;
				} else if now < soft_deadline {
					debug!(
						"Transaction would overflow the block size limit, \
						 but we still have time before the soft deadline, so \
						 we will try a bit more."
					);
					continue;
				} else {
					debug!("Reached block size limit, proceeding with proposing.");
					break EndProposingReason::HitBlockSizeLimit;
				}
			}

			trace!("[{:?}] Pushing to the block.", pending_tx_hash);
			match sc_block_builder::BlockBuilder::push(&mut block_builder, pending_tx_data) {
				Ok(()) => {
					transaction_pushed = true;
					debug!("[{:?}] Pushed to the block.", pending_tx_hash);
				}
				Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
					pending_iterator.report_invalid(&pending_tx);
					if skipped < MAX_SKIPPED_TRANSACTIONS {
						skipped += 1;
						debug!(
							"Block seems full, but will try {} more transactions before quitting.",
							MAX_SKIPPED_TRANSACTIONS - skipped,
						);
					} else if (self.now)() < soft_deadline {
						debug!(
							"Block seems full, but we still have time before the soft deadline, \
							 so we will try a bit more before quitting."
						);
					} else {
						debug!("Reached block weight limit, proceeding with proposing.");
						break EndProposingReason::HitBlockWeightLimit;
					}
				}
				Err(e) if skipped > 0 => {
					pending_iterator.report_invalid(&pending_tx);
					trace!(
						"[{:?}] Ignoring invalid transaction when skipping: {}",
						pending_tx_hash,
						e
					);
				}
				Err(e) => {
					pending_iterator.report_invalid(&pending_tx);
					debug!("[{:?}] Invalid transaction: {}", pending_tx_hash, e);
					unqueue_invalid.push(pending_tx_hash);
				}
			}
		};

		
		if matches!(end_reason, EndProposingReason::HitBlockSizeLimit) && !transaction_pushed {
			warn!(
				"Hit block size limit of `{}` without including any transaction!",
				block_size_limit,
			);
		}

		self.transaction_pool.remove_invalid(&unqueue_invalid);

		let (block, storage_changes, proof) = block_builder.build()?.into_inner();

		self.metrics.report(|metrics| {
			metrics
				.number_of_transactions
				.set(block.extrinsics().len() as u64);
			metrics
				.block_constructed
				.observe(block_timer.elapsed().as_secs_f64());

			metrics.report_end_proposing_reason(end_reason);
		});

		info!(
			"🎁 Prepared block for proposing at {} ({} ms) [hash: {:?}; parent_hash: {}; extrinsics ({}): [{}]]",
			block.header().number(),
			block_timer.elapsed().as_millis(),
			<Block as BlockT>::Hash::from(block.header().hash()),
			block.header().parent_hash(),
			block.extrinsics().len(),
			block.extrinsics()
				.iter()
				.map(|xt| BlakeTwo256::hash_of(xt).to_string())
				.collect::<Vec<_>>()
				.join(", ")
		);
		telemetry!(
			self.telemetry;
			CONSENSUS_INFO;
			"prepared_block_for_proposing";
			"number" => ?block.header().number(),
			"hash" => ?<Block as BlockT>::Hash::from(block.header().hash()),
		);

		let proof =
			PR::into_proof(proof).map_err(|e| sp_blockchain::Error::Application(Box::new(e)))?;

		let propose_with_end = time::Instant::now();
		self.metrics.report(|metrics| {
			metrics.create_block_proposal_time.observe(
				propose_with_end
					.saturating_duration_since(propose_with_start)
					.as_secs_f64(),
			);
		});

		Ok(Proposal {
			block,
			proof,
			storage_changes,
		})
	}
}