// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

use std::time::Duration;

use super::*;

use fc_db::Backend as BackendT;
use fc_rpc::OverrideHandle;
use fp_rpc::EthereumRuntimeRPCApi;
use moonbeam_rpc_debug::{DebugHandler, DebugRequester};
use moonbeam_rpc_trace::{CacheRequester as TraceFilterCacheRequester, CacheTask};
use sc_client_api::BlockOf;
use sc_service::TaskManager;
use sp_core::{traits::SpawnEssentialNamed, H256};
use sp_runtime::traits::Header as HeaderT;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct RpcRequesters {
	pub debug: Option<DebugRequester>,
	pub trace: Option<TraceFilterCacheRequester>,
}

// Spawn the tasks that are required to run a Moonbeam tracing node.
pub fn spawn_tracing_tasks<B, C, BE>(
	task_manager: &TaskManager,
	client: Arc<C>,
	backend: Arc<BE>,
	frontier_backend: Arc<BackendT<B>>,
	overrides: Arc<OverrideHandle<B>>,
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
	let ethapi_max_permits = 1000;
	let trace_enabled = true;
	let debug_enabled = true;
	let permit_pool = Arc::new(Semaphore::new(ethapi_max_permits as usize));
	let tracing_raw_max_memory_usage = 1024 * 1024;
	let ethapi_trace_cache_duration = 3600;

	let (trace_filter_task, trace_filter_requester) = if trace_enabled {
		let (trace_filter_task, trace_filter_requester) = CacheTask::create(
			Arc::clone(&client),
			Arc::clone(&backend),
			Duration::from_secs(ethapi_trace_cache_duration),
			Arc::clone(&permit_pool),
			Arc::clone(&overrides),
		);
		(Some(trace_filter_task), Some(trace_filter_requester))
	} else {
		(None, None)
	};

	let (debug_task, debug_requester) = if debug_enabled {
		let (debug_task, debug_requester) = DebugHandler::task(
			Arc::clone(&client),
			Arc::clone(&backend),
			Arc::clone(&frontier_backend),
			Arc::clone(&permit_pool),
			Arc::clone(&overrides),
			tracing_raw_max_memory_usage,
		);
		(Some(debug_task), Some(debug_requester))
	} else {
		(None, None)
	};

	// `trace_filter` cache task. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(trace_filter_task) = trace_filter_task {
		task_manager.spawn_essential_handle().spawn(
			"trace-filter-cache",
			Some("eth-tracing"),
			trace_filter_task,
		);
	}

	// `debug` task if enabled. Essential.
	// Proxies rpc requests to it's handler.
	if let Some(debug_task) = debug_task {
		task_manager.spawn_essential_handle().spawn(
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
