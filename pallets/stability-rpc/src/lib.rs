use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::H160;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
pub use stability_rpc_api::StabilityRpcApi as StabilityRpcRuntimeApi;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct StabilityOutput<T> {
	code: u32,
	value: T,
}

#[rpc(server)]
pub trait StabilityRpc<BlockHash> {
	#[method(name = "stability_getSupportedTokens")]
	fn get_supported_tokens(&self, at: Option<BlockHash>) -> RpcResult<StabilityOutput<Vec<H160>>>;
}

pub struct StabilityRpcPallet<C, Block> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> StabilityRpcPallet<C, Block> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block> StabilityRpcServer<<Block as BlockT>::Hash> for StabilityRpcPallet<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: StabilityRpcRuntimeApi<Block>,
{
	fn get_supported_tokens(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let value = api
			.get_supported_tokens(&at)
			.map_err(runtime_error_into_rpc_err);
		Ok(StabilityOutput {
			code: 200,
			value: value.unwrap(),
		})
	}
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
