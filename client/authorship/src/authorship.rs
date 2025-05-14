// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

// FIXME #1021 move this into sp-consensus

use account::EthereumSigner;
use fp_rpc::EthereumRuntimeRPCApi;
use futures::{
	channel::oneshot,
	future,
	future::{Future, FutureExt},
	select,
};
use log::{debug, error, info, trace, warn};
use parity_scale_codec::Encode;
use prometheus_endpoint::Registry as PrometheusRegistry;
use sc_block_builder::{BlockBuilderApi, BlockBuilderBuilder};
use sc_telemetry::{telemetry, TelemetryHandle, CONSENSUS_INFO};
use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
use sp_api::{ApiExt, CallApiAt, ProvideRuntimeApi};
use sp_blockchain::{ApplyExtrinsicFailed::Validity, Error::ApplyExtrinsicFailed, HeaderBackend};
use sp_consensus::{DisableProofRecording, EnableProofRecording, ProofRecording, Proposal};
use sp_core::crypto::KeyTypeId;
use sp_core::traits::SpawnNamed;
use sp_inherents::InherentData;
use sp_keystore::{Keystore, KeystorePtr};
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::{
	traits::{BlakeTwo256, Block as BlockT, Hash as HashT, Header as HeaderT},
	Digest, ExtrinsicInclusionMode, Percent, SaturatedConversion,
};
use stability_runtime::AccountId;
use stbl_primitives_fee_compatible_api::CompatibleFeeApi;
use std::{marker::PhantomData, pin::Pin, sync::Arc, time};
// use stbl_primitives_fee_compatible_api::CompatibleFeeApi;
use stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi;
use stbl_proposer_metrics::{EndProposingReason, MetricsLink as PrometheusMetrics};

/// Default block size limit in bytes used by [`Proposer`].
///
/// Can be overwritten by [`ProposerFactory::set_default_block_size_limit`].
///
/// Be aware that there is also an upper packet size on what the networking code
/// will accept. If the block doesn't fit in such a package, it can not be
/// transferred to other nodes.
pub const DEFAULT_BLOCK_SIZE_LIMIT: usize = 4 * 1024 * 1024 + 512;

const DEFAULT_SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(50);

const LOG_TARGET: &'static str = "stble-authorship";

#[derive(serde::Deserialize)]
pub struct RawZeroGasTransactionResponse {
	transactions: Vec<String>,
}

/// [`Proposer`] factory.
pub struct ProposerFactory<A, C, PR> {
	spawn_handle: Box<dyn SpawnNamed>,
	/// The client instance.
	client: Arc<C>,
	/// The transaction pool.
	transaction_pool: Arc<A>,

	/// Reference to Keystore
	keystore: KeystorePtr,

	/// HTTP URL of the private pool from which the node will retrieve zero-gas transactions
	zero_gas_tx_pool: Option<String>,

	/// Timeout in milliseconds for the zero-gas transaction pool
	/// (default: 1000)
	zero_gas_tx_pool_timeout: u64,

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
	/// transactions which exhaust resources, we will conclude that the block is full.
	soft_deadline_percent: Percent,
	telemetry: Option<TelemetryHandle>,
	/// When estimating the block size, should the proof be included?
	include_proof_in_block_size_estimation: bool,
	/// phantom member to pin the `ProofRecording` type.
	_phantom: PhantomData<PR>,
}

impl<A, C, PR> Clone for ProposerFactory<A, C, PR> {
	fn clone(&self) -> Self {
		Self {
			spawn_handle: self.spawn_handle.clone(),
			client: self.client.clone(),
			transaction_pool: self.transaction_pool.clone(),
			keystore: self.keystore.clone(),
			zero_gas_tx_pool: self.zero_gas_tx_pool.clone(),
			zero_gas_tx_pool_timeout: self.zero_gas_tx_pool_timeout,
			metrics: self.metrics.clone(),
			default_block_size_limit: self.default_block_size_limit,
			soft_deadline_percent: self.soft_deadline_percent,
			telemetry: self.telemetry.clone(),
			include_proof_in_block_size_estimation: self.include_proof_in_block_size_estimation,
			_phantom: self._phantom,
		}
	}
}

impl<A, C> ProposerFactory<A, C, DisableProofRecording> {
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
		zero_gas_tx_pool_timeout: u64,
		prometheus: Option<&PrometheusRegistry>,
		telemetry: Option<TelemetryHandle>,
	) -> Self {
		ProposerFactory {
			spawn_handle: Box::new(spawn_handle),
			transaction_pool,
			keystore,
			zero_gas_tx_pool,
			zero_gas_tx_pool_timeout,
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

impl<A, C> ProposerFactory<A, C, EnableProofRecording> {
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
		zero_gas_tx_pool_timeout: u64,
		prometheus: Option<&PrometheusRegistry>,
		telemetry: Option<TelemetryHandle>,
	) -> Self {
		ProposerFactory {
			client,
			spawn_handle: Box::new(spawn_handle),
			transaction_pool,
			keystore,
			zero_gas_tx_pool,
			zero_gas_tx_pool_timeout,
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

impl<A, C, PR> ProposerFactory<A, C, PR> {
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
	/// transactions which exhaust resources, we will conclude that the block is full.
	///
	/// Setting the value too low will significantly limit the amount of transactions
	/// we try in case they exhaust resources. Setting the value too high can
	/// potentially open a DoS vector, where many "exhaust resources" transactions
	/// are being tried with no success, hence block producer ends up creating an empty block.
	pub fn set_soft_deadline(&mut self, percent: Percent) {
		self.soft_deadline_percent = percent;
	}
}

impl<Block, C, A, PR> ProposerFactory<A, C, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: ApiExt<Block> + BlockBuilderApi<Block>,
{
	fn init_with_now(
		&mut self,
		parent_header: &<Block as BlockT>::Header,
		now: Box<dyn Fn() -> time::Instant + Send + Sync>,
	) -> Proposer<Block, C, A, PR> {
		let parent_hash = parent_header.hash();

		info!(
			"🙌 Starting consensus session on top of parent {:?} (#{})",
			parent_hash,
			parent_header.number()
		);

		let proposer = Proposer::<_, _, _, PR> {
			spawn_handle: self.spawn_handle.clone(),
			client: self.client.clone(),
			parent_hash,
			parent_number: *parent_header.number(),
			transaction_pool: self.transaction_pool.clone(),
			keystore: self.keystore.clone(),
			zero_gas_tx_pool: self.zero_gas_tx_pool.clone(),
			zero_gas_tx_pool_timeout: self.zero_gas_tx_pool_timeout,
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

impl<A, Block, C, PR> sp_consensus::Environment<Block> for ProposerFactory<A, C, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + CallApiAt<Block> + Send + Sync + 'static,
	C::Api: ApiExt<Block>
		+ BlockBuilderApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
	PR: ProofRecording,
{
	type CreateProposer = future::Ready<Result<Self::Proposer, Self::Error>>;
	type Proposer = Proposer<Block, C, A, PR>;
	type Error = sp_blockchain::Error;

	fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> Self::CreateProposer {
		future::ready(Ok(
			self.init_with_now(parent_header, Box::new(time::Instant::now))
		))
	}
}

/// The proposer logic.
pub struct Proposer<Block: BlockT, C, A: TransactionPool, PR> {
	spawn_handle: Box<dyn SpawnNamed>,
	client: Arc<C>,
	parent_hash: Block::Hash,
	parent_number: <<Block as BlockT>::Header as HeaderT>::Number,
	transaction_pool: Arc<A>,
	keystore: KeystorePtr,
	zero_gas_tx_pool: Option<String>,
	zero_gas_tx_pool_timeout: u64,
	now: Box<dyn Fn() -> time::Instant + Send + Sync>,
	metrics: PrometheusMetrics,
	default_block_size_limit: usize,
	include_proof_in_block_size_estimation: bool,
	soft_deadline_percent: Percent,
	telemetry: Option<TelemetryHandle>,
	_phantom: PhantomData<PR>,
}

impl<A, Block, C, PR> sp_consensus::Proposer<Block> for Proposer<Block, C, A, PR>
where
	A: TransactionPool<Block = Block> + 'static,
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + CallApiAt<Block> + Send + Sync + 'static,
	C::Api: ApiExt<Block>
		+ BlockBuilderApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
	PR: ProofRecording,
{
	type Proposal =
		Pin<Box<dyn Future<Output = Result<Proposal<Block, PR::Proof>, Self::Error>> + Send>>;
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
					trace!(
						target: LOG_TARGET,
						"Could not send block production result to proposer!"
					);
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

impl<A, Block, C, PR> Proposer<Block, C, A, PR>
where
	A: TransactionPool<Block = Block>,
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + CallApiAt<Block> + Send + Sync + 'static,
	C::Api: ApiExt<Block>
		+ BlockBuilderApi<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
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
	) -> Result<Proposal<Block, PR::Proof>, sp_blockchain::Error> {
		let block_timer = time::Instant::now();
		let mut block_builder = BlockBuilderBuilder::new(&*self.client)
			.on_parent_block(self.parent_hash)
			.with_parent_block_number(self.parent_number)
			.with_proof_recording(PR::ENABLED)
			.with_inherent_digests(inherent_digests)
			.build()?;

		self.apply_inherents(&mut block_builder, inherent_data)?;

		let mode = block_builder.extrinsic_inclusion_mode();
		let end_reason = match mode {
			ExtrinsicInclusionMode::AllExtrinsics => {
				self.apply_extrinsics(&mut block_builder, deadline, block_size_limit)
					.await?
			}
			ExtrinsicInclusionMode::OnlyInherents => EndProposingReason::NoMoreTransactions,
		};
		let (block, storage_changes, proof) = block_builder.build()?.into_inner();
		let block_took = block_timer.elapsed();

		let proof =
			PR::into_proof(proof).map_err(|e| sp_blockchain::Error::Application(Box::new(e)))?;

		self.print_summary(&block, end_reason, block_took, block_timer.elapsed());
		Ok(Proposal {
			block,
			proof,
			storage_changes,
		})
	}

	/// Apply all inherents to the block.
	fn apply_inherents(
		&self,
		block_builder: &mut sc_block_builder::BlockBuilder<'_, Block, C>,
		inherent_data: InherentData,
	) -> Result<(), sp_blockchain::Error> {
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
					warn!(
						target: LOG_TARGET,
						"⚠️  Dropping non-mandatory inherent from overweight block."
					)
				}
				Err(ApplyExtrinsicFailed(Validity(e))) if e.was_mandatory() => {
					error!(
						"❌️ Mandatory inherent extrinsic returned error. Block cannot be produced."
					);
					return Err(ApplyExtrinsicFailed(Validity(e)));
				}
				Err(e) => {
					warn!(
						target: LOG_TARGET,
						"❗️ Inherent extrinsic returned unexpected error: {}. Dropping.", e
					);
				}
				Ok(_) => {}
			}
		}
		Ok(())
	}

	/// Apply as many extrinsics as possible to the block.
	async fn apply_extrinsics(
		&self,
		block_builder: &mut sc_block_builder::BlockBuilder<'_, Block, C>,
		deadline: time::Instant,
		block_size_limit: Option<usize>,
	) -> Result<EndProposingReason, sp_blockchain::Error> {
		// proceed with transactions
		// We calculate soft deadline used only in case we start skipping transactions.
		let now = (self.now)();
		let left = deadline.saturating_duration_since(now);
		let left_micros: u64 = left.as_micros().saturated_into();
		let soft_deadline =
			now + time::Duration::from_micros(self.soft_deadline_percent.mul_floor(left_micros));
		let mut skipped = 0;
		let mut unqueue_invalid = Vec::new();

		// START STABILITY ZGT LOGIC
		let mut transaction_pushed = false;
		let block_size_limit = block_size_limit.unwrap_or(self.default_block_size_limit);

		// Get the current Validators public keys
		let keys = Keystore::ecdsa_public_keys(
			&*self.keystore,
			KeyTypeId::try_from("aura").unwrap_or_default(),
		);

		// Fetch pending transactions from the Zero Gas Transactions pool
		let raw_zero_gas_transactions_option =
			if let Some(zero_gas_tx_pool) = self.zero_gas_tx_pool.clone() {
				let http_client = reqwest::Client::new();
				let mut request = Box::pin(http_client.post(zero_gas_tx_pool).send().fuse());
				let mut timeout = Box::pin(
					futures_timer::Delay::new(std::time::Duration::from_millis(
						self.zero_gas_tx_pool_timeout,
					))
					.fuse(),
				);

				let zgt_response_start = time::Instant::now();

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

				let zgt_response_end = time::Instant::now();
				let zgt_total_time = zgt_response_end.saturating_duration_since(zgt_response_start);
				self.metrics.report(|metrics| {
					metrics
						.zgt_response_time
						.observe(zgt_total_time.clone().as_secs_f64());
				});

				match result_response_raw_zero {
					Ok(response) => {
						match response.json::<RawZeroGasTransactionResponse>().await {
							Ok(json) => {
								info!(
									"📥 Fetched {:?} txns from zero-gas-transactions pool ({:?} ms)",
									json.transactions.len(),
									zgt_total_time.as_millis()
								);
								Some(json)
							}
							Err(e) => {
								error!("Error parsing JSON response from zero gas transaction pool: {}", e);
								None
							}
						}
					}
					Err(e) => {
						error!(
							"Error getting response from zero gas transaction pool: {}",
							e
						);
						None
					}
				}
			} else {
				None
			};

		// INSERT ZGTs INTO THE BLOCK
		// If we pull successfully from the zero gas transaction pool, we will try to push them to the block
		if let Some(raw_zero_gas_transactions) = raw_zero_gas_transactions_option {
			let zgt_inclusion_in_block_start = time::Instant::now();

			if raw_zero_gas_transactions.transactions.len() > 0 {
				let mut pending_raw_zero_gas_transactions =
					raw_zero_gas_transactions.transactions.iter();

				let chain_id = self
					.client
					.runtime_api()
					.chain_id(self.parent_hash)
					.expect("Could not get chain id");

				let current_block = self.parent_number.saturated_into::<u32>().saturating_add(1);

				let message: Vec<u8> = b"I consent to validate zero gas transactions in block "
					.iter()
					.chain(current_block.to_string().as_bytes().iter())
					.chain(b" on chain ")
					.chain(chain_id.to_string().as_bytes().iter())
					.cloned()
					.collect();

				let public = keys[0].clone().into();

				let eip191_message = stbl_tools::eth::build_eip191_message_hash(message.clone());

				let signed_hash = Keystore::ecdsa_sign_prehashed(
					&*self.keystore,
					KeyTypeId::try_from("aura").unwrap_or_default(),
					&public,
					&eip191_message.as_fixed_bytes(),
				)
				.expect("Could not sign the Ethereum transaction hash");

				let zgt_end_reason = loop {
					let pending_hex_string_tx = match pending_raw_zero_gas_transactions.next() {
						Some(tx) => tx,
						_ => break EndProposingReason::NoMoreTransactions,
					};

					let now = (self.now)();
					if now > deadline {
						debug!(
							"Consensus deadline reached when pushing block transactions, \
							proceeding with proposing."
						);
						break EndProposingReason::HitDeadline;
					}

					let pending_raw_tx = match hex::decode(pending_hex_string_tx) {
						Ok(tx) => tx,
						Err(_) => continue,
					};

					let ethereum_transaction: ethereum::TransactionV2 =
						ethereum::EnvelopedDecodable::decode(&pending_raw_tx).unwrap();

					let pending_tx = match self.client.runtime_api().convert_zero_gas_transaction(
						self.parent_hash,
						ethereum_transaction.clone(),
						signed_hash.unwrap().0.to_vec(),
					) {
						Ok(pending_tx) => pending_tx,
						Err(_) => continue,
					};

					let block_size = block_builder
						.estimate_block_size(self.include_proof_in_block_size_estimation);

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
					match sc_block_builder::BlockBuilder::push(block_builder, pending_tx) {
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
							debug!(
								"[{:?}] Invalid transaction: {}",
								ethereum_transaction.hash(),
								e
							);
						}
					}
				};

				if matches!(zgt_end_reason, EndProposingReason::HitBlockSizeLimit)
					&& !transaction_pushed
				{
					warn!(
						target: LOG_TARGET,
						"Hit block size limit of `{}` without including any ZGT transaction!",
						block_size_limit,
					);
				}

				let zgt_inclusion_in_block_end = time::Instant::now();

				self.metrics.report(|metrics| {
					metrics.zgt_inclusion_in_block_time.observe(
						zgt_inclusion_in_block_end
							.saturating_duration_since(zgt_inclusion_in_block_start)
							.as_secs_f64(),
					);
				});

				// STOP PROPOSING LOGIC ON REASON WHERE IS NOT NO MORE TRANSACTIONS
				// SUCH A HIT DEADLINE OR HIT BLOCK SIZE LIMIT
				if zgt_end_reason != EndProposingReason::NoMoreTransactions {
					return Ok(zgt_end_reason);
				}
			}
		};
		// END STABILITY ZGT LOGIC

		let mut t1 = self.transaction_pool.ready_at(self.parent_number).fuse();
		let mut t2 =
			futures_timer::Delay::new(deadline.saturating_duration_since((self.now)()) / 8).fuse();

		let mut pending_iterator = select! {
			res = t1 => res,
			_ = t2 => {
				warn!(target: LOG_TARGET,
					"Timeout fired waiting for transaction pool at block #{}. \
					Proceeding with production.",
					self.parent_number,
				);
				self.transaction_pool.ready()
			},
		};

		debug!(
			target: LOG_TARGET,
			"Attempting to push transactions from the pool."
		);
		debug!(
			target: LOG_TARGET,
			"Pool status: {:?}",
			self.transaction_pool.status()
		);

		// Get the current Validators public keys
		let validator = EthereumSigner::from(keys[0]).into_account();

		let end_reason =
			loop {
				let pending_tx = if let Some(pending_tx) = pending_iterator.next() {
					pending_tx
				} else {
					debug!(
						target: LOG_TARGET,
						"No more transactions, proceeding with proposing."
					);

					break EndProposingReason::NoMoreTransactions;
				};

				let now = (self.now)();
				if now > deadline {
					debug!(
						target: LOG_TARGET,
						"Consensus deadline reached when pushing block transactions, \
				proceeding with proposing."
					);
					break EndProposingReason::HitDeadline;
				}

				// Check if the transaction is compatible with the current fee
				// and the current validator
				let is_compatible = self
					.client
					.runtime_api()
					.is_compatible_fee(
						self.parent_hash,
						pending_tx.data().clone(),
						validator.clone(),
					)
					.unwrap();
				// If the transaction is not compatible, we skip it
				if !is_compatible {
					debug!(
						target: LOG_TARGET,
						"Transaction is not compatible with the current fee or fee is low, skipping."
					);
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
							target: LOG_TARGET,
							"Transaction would overflow the block size limit, \
					 but will try {} more transactions before quitting.",
							MAX_SKIPPED_TRANSACTIONS - skipped,
						);
						continue;
					} else if now < soft_deadline {
						debug!(
							target: LOG_TARGET,
							"Transaction would overflow the block size limit, \
					 but we still have time before the soft deadline, so \
					 we will try a bit more."
						);
						continue;
					} else {
						debug!(
							target: LOG_TARGET,
							"Reached block size limit, proceeding with proposing."
						);
						break EndProposingReason::HitBlockSizeLimit;
					}
				}

				trace!(
					target: LOG_TARGET,
					"[{:?}] Pushing to the block.",
					pending_tx_hash
				);
				match sc_block_builder::BlockBuilder::push(block_builder, pending_tx_data) {
					Ok(()) => {
						transaction_pushed = true;
						debug!(
							target: LOG_TARGET,
							"[{:?}] Pushed to the block.", pending_tx_hash
						);
					}
					Err(ApplyExtrinsicFailed(Validity(e))) if e.exhausted_resources() => {
						pending_iterator.report_invalid(&pending_tx);
						if skipped < MAX_SKIPPED_TRANSACTIONS {
							skipped += 1;
							debug!(target: LOG_TARGET,
							"Block seems full, but will try {} more transactions before quitting.",
							MAX_SKIPPED_TRANSACTIONS - skipped,
						);
						} else if (self.now)() < soft_deadline {
							debug!(target: LOG_TARGET,
							"Block seems full, but we still have time before the soft deadline, \
							 so we will try a bit more before quitting."
						);
						} else {
							debug!(
								target: LOG_TARGET,
								"Reached block weight limit, proceeding with proposing."
							);
							break EndProposingReason::HitBlockWeightLimit;
						}
					}
					Err(e) => {
						pending_iterator.report_invalid(&pending_tx);
						debug!(
							target: LOG_TARGET,
							"[{:?}] Invalid transaction: {}", pending_tx_hash, e
						);
						unqueue_invalid.push(pending_tx_hash);
					}
				}
			};

		if matches!(end_reason, EndProposingReason::HitBlockSizeLimit) && !transaction_pushed {
			warn!(
				target: LOG_TARGET,
				"Hit block size limit of `{}` without including any transaction!", block_size_limit,
			);
		}

		self.transaction_pool.remove_invalid(&unqueue_invalid);
		Ok(end_reason)
	}

	/// Prints a summary and does telemetry + metrics.
	///
	/// - `block`: The block that was build.
	/// - `end_reason`: Why did we stop producing the block?
	/// - `block_took`: How long did it took to produce the actual block?
	/// - `propose_took`: How long did the entire proposing took?
	fn print_summary(
		&self,
		block: &Block,
		end_reason: EndProposingReason,
		block_took: time::Duration,
		propose_took: time::Duration,
	) {
		let extrinsics = block.extrinsics();
		self.metrics.report(|metrics| {
			metrics.number_of_transactions.set(extrinsics.len() as u64);
			metrics.block_constructed.observe(block_took.as_secs_f64());
			metrics.report_end_proposing_reason(end_reason);
			metrics
				.create_block_proposal_time
				.observe(propose_took.as_secs_f64());
		});

		let extrinsics_summary = if extrinsics.is_empty() {
			"no extrinsics".to_string()
		} else {
			format!(
				"extrinsics ({}): [{}]",
				extrinsics.len(),
				extrinsics
					.iter()
					.map(|xt| BlakeTwo256::hash_of(xt).to_string())
					.collect::<Vec<_>>()
					.join(", ")
			)
		};

		info!(
			"🎁 Prepared block for proposing at {} ({} ms) [hash: {:?}; parent_hash: {}; {extrinsics_summary}",
			block.header().number(),
			block_took.as_millis(),
			<Block as BlockT>::Hash::from(block.header().hash()),
			block.header().parent_hash(),
		);
		telemetry!(
			self.telemetry;
			CONSENSUS_INFO;
			"prepared_block_for_proposing";
			"number" => ?block.header().number(),
			"hash" => ?<Block as BlockT>::Hash::from(block.header().hash()),
		);
	}
}
