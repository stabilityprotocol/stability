// Copyright © 2022 STABILITY SOLUTIONS, INC. ("STABILITY")
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the "Software").

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

//! Background worker that fetches zero-gas transactions from an external HTTP pool
//! and submits them into the Substrate transaction pool (mempool).
//!
//! This decouples the HTTP fetch from the block proposal hot path, allowing the
//! proposer to pick up ZGTs from `pool.ready()` like any other transaction.

use fp_rpc::EthereumRuntimeRPCApi;
use futures::{future::FutureExt, select};
use log::{debug, error, info, warn};
use prometheus_endpoint::{register, Counter, Histogram, HistogramOpts, Opts, Registry, U64};
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::crypto::KeyTypeId;
use sp_keystore::{Keystore, KeystorePtr};
use sp_runtime::{
	traits::{Block as BlockT, NumberFor},
	SaturatedConversion,
};
use stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi;
use std::{
	collections::HashMap,
	sync::Arc,
	time,
};

const LOG_TARGET: &str = "zero-gas-fetcher";

/// Maximum number of transaction hashes to keep in the dedup set.
/// Entries older than this many blocks are evicted.
const DEDUP_BLOCK_TTL: u32 = 25;

#[derive(serde::Deserialize)]
struct RawZeroGasTransactionResponse {
	transactions: Vec<String>,
}

/// Prometheus metrics for the ZGT fetcher.
#[derive(Clone)]
pub struct FetcherMetrics {
	pub fetch_time: Histogram,
	pub submit_success: Counter<U64>,
	pub submit_failure: Counter<U64>,
	pub fetch_count: Counter<U64>,
	pub dedup_skipped: Counter<U64>,
}

impl FetcherMetrics {
	pub fn register(registry: &Registry) -> Result<Self, prometheus_endpoint::PrometheusError> {
		Ok(Self {
			fetch_time: register(
				Histogram::with_opts(HistogramOpts::new(
					"stability_zgt_fetcher_fetch_time",
					"Histogram of time taken to fetch from the ZGT HTTP pool",
				))?,
				registry,
			)?,
			submit_success: register(
				Counter::with_opts(Opts::new(
					"stability_zgt_fetcher_submit_success",
					"Number of ZGTs successfully submitted to the Substrate pool",
				))?,
				registry,
			)?,
			submit_failure: register(
				Counter::with_opts(Opts::new(
					"stability_zgt_fetcher_submit_failure",
					"Number of ZGTs that failed to submit to the Substrate pool",
				))?,
				registry,
			)?,
			fetch_count: register(
				Counter::with_opts(Opts::new(
					"stability_zgt_fetcher_fetch_count",
					"Total number of ZGTs fetched from the HTTP pool",
				))?,
				registry,
			)?,
			dedup_skipped: register(
				Counter::with_opts(Opts::new(
					"stability_zgt_fetcher_dedup_skipped",
					"Number of ZGTs skipped due to deduplication",
				))?,
				registry,
			)?,
		})
	}
}

/// Spawns a background worker task that periodically polls the external ZGT HTTP pool
/// and submits fetched transactions into the Substrate transaction pool.
///
/// # Arguments
///
/// * `spawn_handle` - Handle to spawn the background task
/// * `client` - Substrate client for runtime API calls
/// * `pool` - Substrate transaction pool for submission
/// * `keystore` - Keystore for signing consent messages
/// * `url` - HTTP URL of the external ZGT pool
/// * `timeout_ms` - Timeout in milliseconds for the HTTP request
/// * `poll_interval_ms` - Interval in milliseconds between poll cycles
/// * `prometheus` - Optional Prometheus registry for metrics
pub fn spawn_zero_gas_fetcher<Block, C, P>(
	spawn_handle: impl sp_core::traits::SpawnNamed + 'static,
	client: Arc<C>,
	pool: Arc<P>,
	keystore: KeystorePtr,
	url: String,
	timeout_ms: u64,
	poll_interval_ms: u64,
	prometheus: Option<&Registry>,
) where
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<Block>
		+ ZeroGasTransactionApi<Block>,
	P: TransactionPool<Block = Block> + Send + Sync + 'static,
	NumberFor<Block>: Into<u64>,
{
	let metrics = prometheus.and_then(|registry| {
		FetcherMetrics::register(registry)
			.map_err(|err| {
				warn!(
					target: LOG_TARGET,
					"Failed to register ZGT fetcher prometheus metrics: {}", err
				)
			})
			.ok()
	});

	info!(
		target: LOG_TARGET,
		"🚀 Starting zero-gas transaction fetcher (poll interval: {}ms, timeout: {}ms, url: {})",
		poll_interval_ms,
		timeout_ms,
		url,
	);

	spawn_handle.spawn(
		"zero-gas-fetcher",
		Some("zero-gas-transactions"),
		Box::pin(zero_gas_fetcher_loop::<Block, C, P>(
			client,
			pool,
			keystore,
			url,
			timeout_ms,
			poll_interval_ms,
			metrics,
		)),
	);
}

async fn zero_gas_fetcher_loop<Block, C, P>(
	client: Arc<C>,
	pool: Arc<P>,
	keystore: KeystorePtr,
	url: String,
	timeout_ms: u64,
	poll_interval_ms: u64,
	metrics: Option<FetcherMetrics>,
) where
	Block: BlockT,
	C: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<Block>
		+ ZeroGasTransactionApi<Block>,
	P: TransactionPool<Block = Block> + Send + Sync + 'static,
	NumberFor<Block>: Into<u64>,
{
	// Dedup map: ethereum tx hash -> block number when first seen
	let mut seen_txs: HashMap<sp_core::H256, u32> = HashMap::new();
	let http_client = reqwest::Client::new();

	loop {
		// Wait for the poll interval
		futures_timer::Delay::new(std::time::Duration::from_millis(poll_interval_ms)).await;

		let best_hash = client.info().best_hash;
		let best_number: u64 = client.info().best_number.into();
		let current_block_u32 = best_number.saturated_into::<u32>();

		// Evict old entries from the dedup set
		seen_txs.retain(|_, block| {
			current_block_u32.saturating_sub(*block) < DEDUP_BLOCK_TTL
		});

		// Fetch from the external ZGT pool
		let fetch_start = time::Instant::now();

		let raw_txs = match fetch_zero_gas_transactions(
			&http_client,
			&url,
			timeout_ms,
		)
		.await
		{
			Ok(txs) => txs,
			Err(e) => {
				error!(
					target: LOG_TARGET,
					"Failed to fetch from ZGT pool: {}", e
				);
				continue;
			}
		};

		let fetch_duration = fetch_start.elapsed();
		if let Some(ref m) = metrics {
			m.fetch_time.observe(fetch_duration.as_secs_f64());
		}

		if raw_txs.transactions.is_empty() {
			debug!(
				target: LOG_TARGET,
				"No transactions from ZGT pool (fetched in {:?})", fetch_duration
			);
			continue;
		}

		info!(
			target: LOG_TARGET,
			"📥 Fetched {} txns from ZGT enqueue pool ({:?}ms)",
			raw_txs.transactions.len(),
			fetch_duration.as_millis(),
		);

		if let Some(ref m) = metrics {
			m.fetch_count.inc_by(raw_txs.transactions.len() as u64);
		}

		// Get validator keys for signing
		let keys = Keystore::ecdsa_public_keys(
			&*keystore,
			KeyTypeId::try_from("aura").unwrap_or_default(),
		);

		if keys.is_empty() {
			warn!(
				target: LOG_TARGET,
				"No ECDSA keys found in keystore for 'aura'. Cannot sign consent messages."
			);
			continue;
		}

		// Sign the consent message for best_number + 1
		let signing_block = best_number.saturating_add(1);

		let chain_id = match client.runtime_api().chain_id(best_hash) {
			Ok(id) => id,
			Err(e) => {
				error!(
					target: LOG_TARGET,
					"Failed to get chain_id from runtime API: {}", e
				);
				continue;
			}
		};

		let message: Vec<u8> = b"I consent to validate zero gas transactions in block "
			.iter()
			.chain(signing_block.to_string().as_bytes().iter())
			.chain(b" on chain ")
			.chain(chain_id.to_string().as_bytes().iter())
			.cloned()
			.collect();

		let public = keys[0].clone().into();
		let eip191_message = stbl_tools::eth::build_eip191_message_hash(message);

		let validator_signature = match Keystore::ecdsa_sign_prehashed(
			&*keystore,
			KeyTypeId::try_from("aura").unwrap_or_default(),
			&public,
			&eip191_message.as_fixed_bytes(),
		) {
			Ok(Some(sig)) => sig.0.to_vec(),
			Ok(None) => {
				error!(
					target: LOG_TARGET,
					"Keystore returned None for ECDSA signature"
				);
				continue;
			}
			Err(e) => {
				error!(
					target: LOG_TARGET,
					"Failed to sign consent message: {}", e
				);
				continue;
			}
		};

		// Process each transaction
		for hex_tx in &raw_txs.transactions {
			let raw_tx = match hex::decode(hex_tx) {
				Ok(bytes) => bytes,
				Err(e) => {
					debug!(
						target: LOG_TARGET,
						"Failed to decode hex transaction: {}", e
					);
					continue;
				}
			};

			let ethereum_tx: ethereum::TransactionV2 =
				match ethereum::EnvelopedDecodable::decode(&raw_tx) {
					Ok(tx) => tx,
					Err(e) => {
						debug!(
							target: LOG_TARGET,
							"Failed to RLP-decode Ethereum transaction: {:?}", e
						);
						continue;
					}
				};

			let tx_hash = ethereum_tx.hash();

			// Skip if we've already seen this transaction recently
			if seen_txs.contains_key(&tx_hash) {
				if let Some(ref m) = metrics {
					m.dedup_skipped.inc();
				}
				continue;
			}

			// Convert to unsigned extrinsic via runtime API
			let extrinsic = match client.runtime_api().convert_zero_gas_transaction(
				best_hash,
				ethereum_tx.clone(),
				validator_signature.clone(),
			) {
				Ok(ext) => ext,
				Err(e) => {
					debug!(
						target: LOG_TARGET,
						"[{:?}] Failed to convert ZGT via runtime API: {}", tx_hash, e
					);
					continue;
				}
			};

			// Submit to the Substrate transaction pool with TransactionSource::Local
			// so that validate_unsigned skips the consent signature check (which would
			// fail because block_number() and find_author() are incorrect during pool validation)
			match pool
				.submit_one(best_hash, TransactionSource::Local, extrinsic)
				.await
			{
				Ok(_) => {
					debug!(
						target: LOG_TARGET,
						"[{:?}] Successfully submitted ZGT to pool", tx_hash
					);
					seen_txs.insert(tx_hash, current_block_u32);
					if let Some(ref m) = metrics {
						m.submit_success.inc();
					}
				}
				Err(e) => {
					debug!(
						target: LOG_TARGET,
						"[{:?}] Failed to submit ZGT to pool: {}", tx_hash, e
					);
					if let Some(ref m) = metrics {
						m.submit_failure.inc();
					}
				}
			}
		}
	}
}

/// Fetch zero-gas transactions from the external HTTP pool with a timeout.
async fn fetch_zero_gas_transactions(
	client: &reqwest::Client,
	url: &str,
	timeout_ms: u64,
) -> Result<RawZeroGasTransactionResponse, String> {
	let mut request = Box::pin(client.post(url).send().fuse());
	let mut timeout = Box::pin(
		futures_timer::Delay::new(std::time::Duration::from_millis(timeout_ms)).fuse(),
	);

	let response = select! {
		res = request => {
			res.map_err(|e| format!("HTTP request error: {}", e))?
		},
		_ = timeout => {
			return Err("HTTP request timed out".to_string());
		},
	};

	response
		.json::<RawZeroGasTransactionResponse>()
		.await
		.map_err(|e| format!("JSON parse error: {}", e))
}
