// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

//! `trace_filter` RPC handler and its associated service task.
//! The RPC handler rely on `CacheTask` which provides a future that must be run inside a tokio
//! executor.
//!
//! The implementation is composed of multiple tasks :
//! - Many calls the the RPC handler `Trace::filter`, communicating with the main task.
//! - A main `CacheTask` managing the cache and the communication between tasks.
//! - For each traced block an async task responsible to wait for a permit, spawn a blocking
//!   task and waiting for the result, then send it to the main `CacheTask`.

use futures::{select, stream::FuturesUnordered, FutureExt, StreamExt};
use std::{collections::BTreeMap, future::Future, marker::PhantomData, sync::Arc, time::Duration};
use tokio::{
	sync::{mpsc, oneshot, Semaphore},
	time::sleep,
};
use tracing::{instrument, Instrument};

use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use substrate_prometheus_endpoint::{
	register, Counter, PrometheusError, Registry as PrometheusRegistry, U64,
};

use ethereum_types::H256;
use fc_storage::StorageOverride;
use fp_rpc::EthereumRuntimeRPCApi;

use moonbeam_client_evm_tracing::{
	formatters::ResponseFormatter,
	types::block::{self, TransactionTrace},
};
pub use moonbeam_rpc_core_trace::{FilterRequest, TraceServer};
use moonbeam_rpc_core_types::{RequestBlockId, RequestBlockTag};
use moonbeam_rpc_primitives_debug::DebugRuntimeApi;

type TxsTraceRes = Result<Vec<TransactionTrace>, String>;

/// RPC handler. Will communicate with a `CacheTask` through a `CacheRequester`.
pub struct Trace<B, C> {
	_phantom: PhantomData<B>,
	client: Arc<C>,
	requester: CacheRequester,
	max_count: u32,
}

impl<B, C> Clone for Trace<B, C> {
	fn clone(&self) -> Self {
		Self {
			_phantom: PhantomData,
			client: Arc::clone(&self.client),
			requester: self.requester.clone(),
			max_count: self.max_count,
		}
	}
}

impl<B, C> Trace<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	/// Create a new RPC handler.
	pub fn new(client: Arc<C>, requester: CacheRequester, max_count: u32) -> Self {
		Self {
			client,
			requester,
			max_count,
			_phantom: PhantomData,
		}
	}

	/// Convert an optional block ID (number or tag) to a block height.
	fn block_id(&self, id: Option<RequestBlockId>) -> Result<u32, &'static str> {
		match id {
			Some(RequestBlockId::Number(n)) => Ok(n),
			None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
				Ok(self.client.info().best_number)
			}
			Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
			Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
				Err("'pending' is not supported")
			}
			Some(RequestBlockId::Hash(_)) => Err("Block hash not supported"),
		}
	}

	/// `trace_filter` endpoint (wrapped in the trait implementation with futures compatibilty)
	async fn filter(self, req: FilterRequest) -> TxsTraceRes {
		let from_block = self.block_id(req.from_block)?;
		let to_block = self.block_id(req.to_block)?;
		let block_heights = from_block..=to_block;

		let count = req.count.unwrap_or(self.max_count);
		if count > self.max_count {
			return Err(format!(
				"count ({}) can't be greater than maximum ({})",
				count, self.max_count
			));
		}

		// Build a list of all the Substrate block hashes that need to be traced.
		let mut block_hashes = vec![];
		for block_height in block_heights {
			if block_height == 0 {
				continue; // no traces for genesis block.
			}

			let block_hash = self
				.client
				.hash(block_height)
				.map_err(|e| {
					format!(
						"Error when fetching block {} header : {:?}",
						block_height, e
					)
				})?
				.ok_or_else(|| format!("Block with height {} don't exist", block_height))?;

			block_hashes.push(block_hash);
		}

		// Start a batch with these blocks.
		let batch_id = self.requester.start_batch(block_hashes.clone()).await?;
		// Fetch all the traces. It is done in another function to simplify error handling and allow
		// to call the following `stop_batch` regardless of the result. This is important for the
		// cache cleanup to work properly.
		let res = self.fetch_traces(req, &block_hashes, count as usize).await;
		// Stop the batch, allowing the cache task to remove useless non-started block traces and
		// start the expiration delay.
		self.requester.stop_batch(batch_id).await;

		res
	}

	async fn fetch_traces(
		&self,
		req: FilterRequest,
		block_hashes: &[H256],
		count: usize,
	) -> TxsTraceRes {
		let from_address = req.from_address.unwrap_or_default();
		let to_address = req.to_address.unwrap_or_default();

		let mut traces_amount: i64 = -(req.after.unwrap_or(0) as i64);
		let mut traces = vec![];

		for &block_hash in block_hashes {
			// Request the traces of this block to the cache service.
			// This will resolve quickly if the block is already cached, or wait until the block
			// has finished tracing.
			let block_traces = self.requester.get_traces(block_hash).await?;

			// Filter addresses.
			let mut block_traces: Vec<_> = block_traces
				.iter()
				.filter(|trace| match trace.action {
					block::TransactionTraceAction::Call { from, to, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& (to_address.is_empty() || to_address.contains(&to))
					}
					block::TransactionTraceAction::Create { from, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& to_address.is_empty()
					}
					block::TransactionTraceAction::Suicide { address, .. } => {
						(from_address.is_empty() || from_address.contains(&address))
							&& to_address.is_empty()
					}
				})
				.cloned()
				.collect();

			// Don't insert anything if we're still before "after"
			traces_amount += block_traces.len() as i64;
			if traces_amount > 0 {
				let traces_amount = traces_amount as usize;
				// If the current Vec of traces is across the "after" marker,
				// we skip some elements of it.
				if traces_amount < block_traces.len() {
					let skip = block_traces.len() - traces_amount;
					block_traces = block_traces.into_iter().skip(skip).collect();
				}

				traces.append(&mut block_traces);

				// If we go over "count" (the limit), we trim and exit the loop,
				// unless we used the default maximum, in which case we return an error.
				if traces_amount >= count {
					if req.count.is_none() {
						return Err(format!(
							"the amount of traces goes over the maximum ({}), please use 'after' \
							and 'count' in your request",
							self.max_count
						));
					}

					traces = traces.into_iter().take(count).collect();
					break;
				}
			}
		}

		Ok(traces)
	}
}

#[jsonrpsee::core::async_trait]
impl<B, C> TraceServer for Trace<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	async fn filter(
		&self,
		filter: FilterRequest,
	) -> jsonrpsee::core::RpcResult<Vec<TransactionTrace>> {
		self.clone()
			.filter(filter)
			.await
			.map_err(|e| fc_rpc::internal_err(e))
	}
}

/// An opaque batch ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheBatchId(u64);

/// Requests the cache task can accept.
enum CacheRequest {
	/// Request to start caching the provided range of blocks.
	/// The task will add to blocks to its pool and immediately return a new batch ID.
	StartBatch {
		/// Returns the ID of the batch for cancellation.
		sender: oneshot::Sender<CacheBatchId>,
		/// List of block hash to trace.
		blocks: Vec<H256>,
	},
	/// Fetch the traces for given block hash.
	/// The task will answer only when it has processed this block.
	GetTraces {
		/// Returns the array of traces or an error.
		sender: oneshot::Sender<TxsTraceRes>,
		/// Hash of the block.
		block: H256,
	},
	/// Notify the cache that it can stop the batch with that ID. Any block contained only in
	/// this batch and still not started will be discarded.
	StopBatch { batch_id: CacheBatchId },
}

/// Allows to interact with the cache task.
#[derive(Clone)]
pub struct CacheRequester(TracingUnboundedSender<CacheRequest>);

impl CacheRequester {
	/// Request to start caching the provided range of blocks.
	/// The task will add to blocks to its pool and immediately return the batch ID.
	#[instrument(skip(self))]
	pub async fn start_batch(&self, blocks: Vec<H256>) -> Result<CacheBatchId, String> {
		let (response_tx, response_rx) = oneshot::channel();
		let sender = self.0.clone();

		sender
			.unbounded_send(CacheRequest::StartBatch {
				sender: response_tx,
				blocks,
			})
			.map_err(|e| {
				format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				)
			})?;

		response_rx.await.map_err(|e| {
			format!(
				"Trace cache task closed the response channel. Error : {:?}",
				e
			)
		})
	}

	/// Fetch the traces for given block hash.
	/// The task will answer only when it has processed this block.
	/// The block should be part of a batch first. If no batch has requested the block it will
	/// return an error.
	#[instrument(skip(self))]
	pub async fn get_traces(&self, block: H256) -> TxsTraceRes {
		let (response_tx, response_rx) = oneshot::channel();
		let sender = self.0.clone();

		sender
			.unbounded_send(CacheRequest::GetTraces {
				sender: response_tx,
				block,
			})
			.map_err(|e| {
				format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				)
			})?;

		response_rx
			.await
			.map_err(|e| {
				format!(
					"Trace cache task closed the response channel. Error : {:?}",
					e
				)
			})?
			.map_err(|e| format!("Failed to replay block. Error : {:?}", e))
	}

	/// Notify the cache that it can stop the batch with that ID. Any block contained only in
	/// this batch and still in the waiting pool will be discarded.
	#[instrument(skip(self))]
	pub async fn stop_batch(&self, batch_id: CacheBatchId) {
		let sender = self.0.clone();

		// Here we don't care if the request has been accepted or refused, the caller can't
		// do anything with it.
		let _ = sender
			.unbounded_send(CacheRequest::StopBatch { batch_id })
			.map_err(|e| {
				format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				)
			});
	}
}

/// Data stored for each block in the cache.
/// `active_batch_count` represents the number of batches using this
/// block. It will increase immediatly when a batch is created, but will be
/// decrease only after the batch ends and its expiration delay passes.
/// It allows to keep the data in the cache for following requests that would use
/// this block, which is important to handle pagination efficiently.
struct CacheBlock {
	active_batch_count: usize,
	state: CacheBlockState,
}

/// State of a cached block. It can either be polled to be traced or cached.
enum CacheBlockState {
	/// Block has been added to the pool blocks to be replayed.
	/// It may be currently waiting to be replayed or being replayed.
	Pooled {
		started: bool,
		/// Multiple requests might query the same block while it is pooled to be
		/// traced. They response channel is stored here, and the result will be
		/// sent in all of them when the tracing is finished.
		waiting_requests: Vec<oneshot::Sender<TxsTraceRes>>,
		/// Channel used to unqueue a tracing that has not yet started.
		/// A tracing will be unqueued if it has not yet been started and the last batch
		/// needing this block is ended (ignoring the expiration delay).
		/// It is not used directly, but dropping will wake up the receiver.
		#[allow(dead_code)]
		unqueue_sender: oneshot::Sender<()>,
	},
	/// Tracing has completed and the result is available. No Runtime API call
	/// will be needed until this block cache is removed.
	Cached { traces: TxsTraceRes },
}

/// Tracing a block is done in a separate tokio blocking task to avoid clogging the async threads.
/// For this reason a channel using this type is used by the blocking task to communicate with the
/// main cache task.
enum BlockingTaskMessage {
	/// Notify the tracing for this block has started as the blocking task got a permit from
	/// the semaphore. This is used to prevent the deletion of a cache entry for a block that has
	/// started being traced.
	Started { block_hash: H256 },
	/// The tracing is finished and the result is send to the main task.
	Finished {
		block_hash: H256,
		result: TxsTraceRes,
	},
}

/// Type wrapper for the cache task, generic over the Client, Block and Backend types.
pub struct CacheTask<B, C, BE> {
	client: Arc<C>,
	backend: Arc<BE>,
	blocking_permits: Arc<Semaphore>,
	cached_blocks: BTreeMap<H256, CacheBlock>,
	batches: BTreeMap<u64, Vec<H256>>,
	next_batch_id: u64,
	metrics: Option<Metrics>,
	_phantom: PhantomData<B>,
}

impl<B, C, BE> CacheTask<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<B>,
	C: StorageProvider<B, BE>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
	C::Api: ApiExt<B>,
{
	/// Create a new cache task.
	///
	/// Returns a Future that needs to be added to a tokio executor, and an handle allowing to
	/// send requests to the task.
	pub fn create(
		client: Arc<C>,
		backend: Arc<BE>,
		cache_duration: Duration,
		blocking_permits: Arc<Semaphore>,
		storage_override: Arc<dyn StorageOverride<B>>,
		prometheus: Option<PrometheusRegistry>,
	) -> (impl Future<Output = ()>, CacheRequester) {
		// Communication with the outside world :
		let (requester_tx, mut requester_rx) =
			sc_utils::mpsc::tracing_unbounded("trace-filter-cache", 100_000);

		// Task running in the service.
		let task = async move {
			// The following variables are polled by the select! macro, and thus cannot be
			// part of Self without introducing borrowing issues.
			let mut batch_expirations = FuturesUnordered::new();
			let (blocking_tx, mut blocking_rx) =
				mpsc::channel(blocking_permits.available_permits() * 2);
			let metrics = if let Some(registry) = prometheus {
				match Metrics::register(&registry) {
					Ok(metrics) => Some(metrics),
					Err(err) => {
						log::error!(target: "tracing", "Failed to register metrics {err:?}");
						None
					}
				}
			} else {
				None
			};
			// Contains the inner state of the cache task, excluding the pooled futures/channels.
			// Having this object allow to refactor each event into its own function, simplifying
			// the main loop.
			let mut inner = Self {
				client,
				backend,
				blocking_permits,
				cached_blocks: BTreeMap::new(),
				batches: BTreeMap::new(),
				next_batch_id: 0,
				metrics,
				_phantom: Default::default(),
			};

			// Main event loop. This loop must not contain any direct .await, as we want to
			// react to events as fast as possible.
			loop {
				select! {
					request = requester_rx.next() => {
						match request {
							None => break,
							Some(CacheRequest::StartBatch {sender, blocks})
								=> inner.request_start_batch(&blocking_tx, sender, blocks, storage_override.clone()),
							Some(CacheRequest::GetTraces {sender, block})
								=> inner.request_get_traces(sender, block),
							Some(CacheRequest::StopBatch {batch_id}) => {
								// Cannot be refactored inside `request_stop_batch` because
								// it has an unnamable type :C
								batch_expirations.push(async move {
									sleep(cache_duration).await;
									batch_id
								});

								inner.request_stop_batch(batch_id);
							},
						}
					},
					message = blocking_rx.recv().fuse() => {
						match message {
							None => (),
							Some(BlockingTaskMessage::Started { block_hash })
								=> inner.blocking_started(block_hash),
							Some(BlockingTaskMessage::Finished { block_hash, result })
								=> inner.blocking_finished(block_hash, result),
						}
					},
					batch_id = batch_expirations.next() => {
						match batch_id {
							None => (),
							Some(batch_id) => inner.expired_batch(batch_id),
						}
					}
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

		(task, CacheRequester(requester_tx))
	}

	/// Handle the creation of a batch.
	/// Will start the tracing process for blocks that are not already in the cache.
	#[instrument(skip(self, blocking_tx, sender, blocks, storage_override))]
	fn request_start_batch(
		&mut self,
		blocking_tx: &mpsc::Sender<BlockingTaskMessage>,
		sender: oneshot::Sender<CacheBatchId>,
		blocks: Vec<H256>,
		storage_override: Arc<dyn StorageOverride<B>>,
	) {
		tracing::trace!("Starting batch {}", self.next_batch_id);
		self.batches.insert(self.next_batch_id, blocks.clone());

		for block in blocks {
			// The block is already in the cache, awesome !
			if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
				block_cache.active_batch_count += 1;
				tracing::trace!(
					"Cache hit for block {}, now used by {} batches.",
					block,
					block_cache.active_batch_count
				);
			}
			// Otherwise we need to queue this block for tracing.
			else {
				tracing::trace!("Cache miss for block {}, pooling it for tracing.", block);

				let blocking_permits = Arc::clone(&self.blocking_permits);
				let (unqueue_sender, unqueue_receiver) = oneshot::channel();
				let client = Arc::clone(&self.client);
				let backend = Arc::clone(&self.backend);
				let blocking_tx = blocking_tx.clone();
				let storage_override = storage_override.clone();

				// Spawn all block caching asynchronously.
				// It will wait to obtain a permit, then spawn a blocking task.
				// When the blocking task returns its result, it is send
				// thought a channel to the main task loop.
				tokio::spawn(
					async move {
						tracing::trace!("Waiting for blocking permit or task cancellation");
						let _permit = select!(
							_ = unqueue_receiver.fuse() => {
							tracing::trace!("Tracing of the block has been cancelled.");
								return;
							},
							permit = blocking_permits.acquire().fuse() => permit,
						);

						// Warn the main task that block tracing as started, and
						// this block cache entry should not be removed.
						let _ = blocking_tx
							.send(BlockingTaskMessage::Started { block_hash: block })
							.await;

						tracing::trace!("Start block tracing in a blocking task.");

						// Perform block tracing in a tokio blocking task.
						let result = async {
							tokio::task::spawn_blocking(move || {
								Self::cache_block(client, backend, block, storage_override.clone())
							})
							.await
							.map_err(|e| {
								format!("Tracing Substrate block {} panicked : {:?}", block, e)
							})?
						}
						.await
						.map_err(|e| e.to_string());

						tracing::trace!("Block tracing finished, sending result to main task.");

						// Send response to main task.
						let _ = blocking_tx
							.send(BlockingTaskMessage::Finished {
								block_hash: block,
								result,
							})
							.await;
					}
					.instrument(tracing::trace_span!("Block tracing", block = %block)),
				);

				// Insert the block in the cache.
				self.cached_blocks.insert(
					block,
					CacheBlock {
						active_batch_count: 1,
						state: CacheBlockState::Pooled {
							started: false,
							waiting_requests: vec![],
							unqueue_sender,
						},
					},
				);
			}
		}

		// Respond with the batch ID.
		let _ = sender.send(CacheBatchId(self.next_batch_id));

		// Increase batch ID for next request.
		self.next_batch_id = self.next_batch_id.overflowing_add(1).0;
	}

	/// Handle a request to get the traces of the provided block.
	/// - If the result is stored in the cache, it sends it immediatly.
	/// - If the block is currently being pooled, it is added in this block cache waiting list,
	///   and all requests concerning this block will be satisfied when the tracing for this block
	///   is finished.
	/// - If this block is missing from the cache, it means no batch asked for it. All requested
	///   blocks should be contained in a batch beforehand, and thus an error is returned.
	#[instrument(skip(self))]
	fn request_get_traces(&mut self, sender: oneshot::Sender<TxsTraceRes>, block: H256) {
		if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
			match &mut block_cache.state {
				CacheBlockState::Pooled {
					ref mut waiting_requests,
					..
				} => {
					tracing::warn!(
						"A request asked a pooled block ({}), adding it to the list of \
						waiting requests.",
						block
					);
					waiting_requests.push(sender);
					if let Some(metrics) = &self.metrics {
						metrics.tracing_cache_misses.inc();
					}
				}
				CacheBlockState::Cached { traces, .. } => {
					tracing::warn!(
						"A request asked a cached block ({}), sending the traces directly.",
						block
					);
					let _ = sender.send(traces.clone());
					if let Some(metrics) = &self.metrics {
						metrics.tracing_cache_hits.inc();
					}
				}
			}
		} else {
			tracing::warn!(
				"An RPC request asked to get a block ({}) which was not batched.",
				block
			);
			let _ = sender.send(Err(format!(
				"RPC request asked a block ({}) that was not batched",
				block
			)));
		}
	}

	/// Handle a request to stop a batch.
	/// For all blocks that needed to be traced, are only in this batch and not yet started, their
	/// tracing is cancelled to save CPU-time and avoid attacks requesting large amount of blocks.
	/// This batch data is not yet removed however. Instead a expiration delay timer is started
	/// after which the data will indeed be cleared. (the code for that is in the main loop code
	/// as it involved an unnamable type :C)
	#[instrument(skip(self))]
	fn request_stop_batch(&mut self, batch_id: CacheBatchId) {
		tracing::trace!("Stopping batch {}", batch_id.0);
		if let Some(blocks) = self.batches.get(&batch_id.0) {
			for block in blocks {
				let mut remove = false;

				// We remove early the block cache if this batch is the last
				// pooling this block.
				if let Some(block_cache) = self.cached_blocks.get_mut(block) {
					if block_cache.active_batch_count == 1
						&& matches!(
							block_cache.state,
							CacheBlockState::Pooled { started: false, .. }
						) {
						remove = true;
					}
				}

				if remove {
					tracing::trace!("Pooled block {} is no longer requested.", block);
					// Remove block from the cache. Drops the value,
					// closing all the channels contained in it.
					let _ = self.cached_blocks.remove(&block);
				}
			}
		}
	}

	/// A tracing blocking task notifies it got a permit and is starting the tracing.
	/// This started status is stored to avoid removing this block entry.
	#[instrument(skip(self))]
	fn blocking_started(&mut self, block_hash: H256) {
		if let Some(block_cache) = self.cached_blocks.get_mut(&block_hash) {
			if let CacheBlockState::Pooled {
				ref mut started, ..
			} = block_cache.state
			{
				*started = true;
			}
		}
	}

	/// A tracing blocking task notifies it has finished the tracing and provide the result.
	#[instrument(skip(self, result))]
	fn blocking_finished(&mut self, block_hash: H256, result: TxsTraceRes) {
		// In some cases it might be possible to receive traces of a block
		// that has no entry in the cache because it was removed of the pool
		// and received a permit concurrently. We just ignore it.
		//
		// TODO : Should we add it back ? Should it have an active_batch_count
		// of 1 then ?
		if let Some(block_cache) = self.cached_blocks.get_mut(&block_hash) {
			if let CacheBlockState::Pooled {
				ref mut waiting_requests,
				..
			} = block_cache.state
			{
				tracing::trace!(
					"A new block ({}) has been traced, adding it to the cache and responding to \
					{} waiting requests.",
					block_hash,
					waiting_requests.len()
				);
				// Send result in waiting channels
				while let Some(channel) = waiting_requests.pop() {
					let _ = channel.send(result.clone());
				}

				// Update cache entry
				block_cache.state = CacheBlockState::Cached { traces: result };
			}
		}
	}

	/// A batch expiration delay timer has completed. It performs the cache cleaning for blocks
	/// not longer used by other batches.
	#[instrument(skip(self))]
	fn expired_batch(&mut self, batch_id: CacheBatchId) {
		if let Some(batch) = self.batches.remove(&batch_id.0) {
			for block in batch {
				// For each block of the batch, we remove it if it was the
				// last batch containing it.
				let mut remove = false;
				if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
					block_cache.active_batch_count -= 1;

					if block_cache.active_batch_count == 0 {
						remove = true;
					}
				}

				if remove {
					let _ = self.cached_blocks.remove(&block);
				}
			}
		}
	}

	/// (In blocking task) Use the Runtime API to trace the block.
	#[instrument(skip(client, backend, storage_override))]
	fn cache_block(
		client: Arc<C>,
		backend: Arc<BE>,
		substrate_hash: H256,
		storage_override: Arc<dyn StorageOverride<B>>,
	) -> TxsTraceRes {
		// Get Subtrate block data.
		let api = client.runtime_api();
		let block_header = client
			.header(substrate_hash)
			.map_err(|e| {
				format!(
					"Error when fetching substrate block {} header : {:?}",
					substrate_hash, e
				)
			})?
			.ok_or_else(|| format!("Subtrate block {} don't exist", substrate_hash))?;

		let height = *block_header.number();
		let substrate_parent_hash = *block_header.parent_hash();

		// Get Ethereum block data.
		let (eth_block, eth_transactions) = match (
			storage_override.current_block(substrate_hash),
			storage_override.current_transaction_statuses(substrate_hash),
		) {
			(Some(a), Some(b)) => (a, b),
			_ => {
				return Err(format!(
					"Failed to get Ethereum block data for Substrate block {}",
					substrate_hash
				))
			}
		};

		let eth_block_hash = eth_block.header.hash();
		let eth_tx_hashes = eth_transactions
			.iter()
			.map(|t| t.transaction_hash)
			.collect();

		// Get extrinsics (containing Ethereum ones)
		let extrinsics = backend
			.blockchain()
			.body(substrate_hash)
			.map_err(|e| {
				format!(
					"Blockchain error when fetching extrinsics of block {} : {:?}",
					height, e
				)
			})?
			.ok_or_else(|| format!("Could not find block {} when fetching extrinsics.", height))?;

		// Get DebugRuntimeApi version
		let trace_api_version = if let Ok(Some(api_version)) =
			api.api_version::<dyn DebugRuntimeApi<B>>(substrate_parent_hash)
		{
			api_version
		} else {
			return Err("Runtime api version call failed (trace)".to_string());
		};

		// Trace the block.
		let f = || -> Result<_, String> {
			let result = if trace_api_version >= 5 {
				// The block is initialized inside "trace_block"
				api.trace_block(
					substrate_parent_hash,
					extrinsics,
					eth_tx_hashes,
					&block_header,
				)
			} else {
				// Old "trace_block" api did not initialize block before applying transactions,
				// so we need to do it here before calling "trace_block".
				#[allow(deprecated)]
				api.initialize_block_before_version_5(substrate_parent_hash, &block_header)
					.map_err(|e| format!("Runtime api access error: {:?}", e))?;

				#[allow(deprecated)]
				api.trace_block_before_version_5(substrate_parent_hash, extrinsics, eth_tx_hashes)
			};

			result
				.map_err(|e| format!("Blockchain error when replaying block {} : {:?}", height, e))?
				.map_err(|e| {
					tracing::warn!(
						"Internal runtime error when replaying block {} : {:?}",
						height,
						e
					);
					format!(
						"Internal runtime error when replaying block {} : {:?}",
						height, e
					)
				})?;

			Ok(moonbeam_rpc_primitives_debug::Response::Block)
		};

		let eth_transactions_by_index: BTreeMap<u32, H256> = eth_transactions
			.iter()
			.map(|t| (t.transaction_index, t.transaction_hash))
			.collect();

		let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
		proxy.using(f)?;

		let traces: Vec<TransactionTrace> =
			moonbeam_client_evm_tracing::formatters::TraceFilter::format(proxy)
				.ok_or("Fail to format proxy")?
				.into_iter()
				.filter_map(|mut trace| {
					match eth_transactions_by_index.get(&trace.transaction_position) {
						Some(transaction_hash) => {
							trace.block_hash = eth_block_hash;
							trace.block_number = height;
							trace.transaction_hash = *transaction_hash;

							// Reformat error messages.
							if let block::TransactionTraceOutput::Error(ref mut error) =
								trace.output
							{
								if error.as_slice() == b"execution reverted" {
									*error = b"Reverted".to_vec();
								}
							}

							Some(trace)
						}
						None => {
							log::warn!(
								"A trace in block {} does not map to any known ethereum transaction. Trace: {:?}",
								height,
								trace,
							);
							None
						}
					}
				})
				.collect();

		Ok(traces)
	}
}

/// Prometheus metrics for tracing.
#[derive(Clone)]
pub(crate) struct Metrics {
	tracing_cache_hits: Counter<U64>,
	tracing_cache_misses: Counter<U64>,
}

impl Metrics {
	pub(crate) fn register(registry: &PrometheusRegistry) -> Result<Self, PrometheusError> {
		Ok(Self {
			tracing_cache_hits: register(
				Counter::new("tracing_cache_hits", "Number of tracing cache hits.")?,
				registry,
			)?,
			tracing_cache_misses: register(
				Counter::new("tracing_cache_misses", "Number of tracing cache misses.")?,
				registry,
			)?,
		})
	}
}
