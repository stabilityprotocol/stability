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

use super::*;

use crate::eth::EthConfiguration;
use fc_rpc_core::types::FilterPool;
use fc_storage::StorageOverride;
use fp_rpc::EthereumRuntimeRPCApi;
use moonbeam_rpc_debug::{DebugHandler, DebugRequester};
use moonbeam_rpc_trace::{CacheRequester as TraceFilterCacheRequester, CacheTask};
use prometheus_endpoint::Registry as PrometheusRegistry;
use sc_client_api::{
	Backend, BlockOf, BlockchainEvents, HeaderBackend, StateBackend, StorageProvider,
};
use sc_service::TaskManager;
use sp_block_builder::BlockBuilder;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use std::str::FromStr;
use tokio::sync::Semaphore;

#[derive(Debug, PartialEq, Clone)]
pub enum EthApi {
	Txpool,
	Debug,
	Trace,
}

impl FromStr for EthApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"txpool" => Self::Txpool,
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				log::error!("`{}` is not recognized as a supported Ethereum Api", s);
				Self::Txpool // Default to Txpool or any other default variant
			}
		})
	}
}

#[derive(Clone)]
pub struct RpcRequesters {
	pub debug: Option<DebugRequester>,
	pub trace: Option<TraceFilterCacheRequester>,
}

pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
	pub filter_pool: Option<FilterPool>,
	pub storage_override: Arc<dyn StorageOverride<B>>,
}

// Spawn the tasks that are required to run a Moonbeam tracing node.
pub fn spawn_tracing_tasks<B, C, BE>(
	rpc_config: &EthConfiguration,
	prometheus: Option<PrometheusRegistry>,
	params: SpawnTasksParams<B, C, BE>,
) -> RpcRequesters
where
	C: ProvideRuntimeApi<B> + BlockOf,
	C: StorageProvider<B, BE>,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: BlockchainEvents<B>,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<B> + moonbeam_rpc_primitives_debug::DebugRuntimeApi<B>,
	C::Api: BlockBuilder<B>,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let permit_pool = Arc::new(Semaphore::new(rpc_config.ethapi_max_permits as usize));

	let (trace_filter_task, trace_filter_requester) = if rpc_config.ethapi.contains(&EthApi::Trace)
	{
		let (trace_filter_task, trace_filter_requester) = CacheTask::create(
			Arc::clone(&params.client),
			Arc::clone(&params.substrate_backend),
			core::time::Duration::from_secs(rpc_config.ethapi_trace_cache_duration),
			Arc::clone(&permit_pool),
			Arc::clone(&params.storage_override),
			prometheus,
		);
		(Some(trace_filter_task), Some(trace_filter_requester))
	} else {
		(None, None)
	};

	let (debug_task, debug_requester) = if rpc_config.ethapi.contains(&EthApi::Debug) {
		let (debug_task, debug_requester) = DebugHandler::task(
			Arc::clone(&params.client),
			Arc::clone(&params.substrate_backend),
			Arc::clone(&params.frontier_backend),
			Arc::clone(&permit_pool),
			Arc::clone(&params.storage_override),
			rpc_config.tracing_raw_max_memory_usage,
		);
		(Some(debug_task), Some(debug_requester))
	} else {
		(None, None)
	};

	// `trace_filter` cache task. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(trace_filter_task) = trace_filter_task {
		params.task_manager.spawn_essential_handle().spawn(
			"trace-filter-cache",
			Some("eth-tracing"),
			trace_filter_task,
		);
	}

	// `debug` task if enabled. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(debug_task) = debug_task {
		params.task_manager.spawn_essential_handle().spawn(
			"ethapi-debug",
			Some("eth-tracing"),
			debug_task,
		);
	}

	RpcRequesters {
		debug: debug_requester,
		trace: trace_filter_requester,
	}
}
