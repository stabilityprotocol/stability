use futures_util::TryFutureExt;
use jsonrpsee::types::{ErrorObject};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use sc_transaction_pool_api::TransactionSource;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{Bytes, H160, H256};
use sp_runtime::traits::Block as BlockT;
pub use stability_rpc_api::StabilityRpcApi as StabilityRpcRuntimeApi;
use std::{
	str::{self},
	sync::Arc,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct StabilityOutput<T> {
	code: u32,
	value: T,
}

#[rpc(server)]
pub trait StabilityRpcEndpoints<BlockHash> {
	#[method(name = "stability_getSupportedTokens")]
	fn get_supported_tokens(&self, at: Option<BlockHash>) -> RpcResult<StabilityOutput<Vec<H160>>>;

	#[method(name = "stability_getValidatorList")]
	fn get_validator_list(&self, at: Option<BlockHash>) -> RpcResult<StabilityOutput<Vec<H160>>>;

	#[method(name = "stability_getActiveValidatorList")]
	fn get_active_validator_list(
		&self,
		at: Option<BlockHash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>>;

	#[method(name = "stability_sendSponsoredTransaction")]
	async fn send_sponsored_transaction(
		&self,
		transaction_req: Bytes,
		meta_trx_sponsor: H160,
		meta_trx_sponsor_signature: Bytes,
	) -> RpcResult<H256>;
}

pub struct StabilityRpc<C, P, Block> {
	client: Arc<C>,
	pool: Arc<P>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, P, Block> StabilityRpc<C, P, Block> {
	pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
		Self {
			client,
			pool,
			_marker: Default::default(),
		}
	}
}

#[async_trait::async_trait]
impl<C, Pool, Block> StabilityRpcEndpointsServer<<Block as BlockT>::Hash>
	for StabilityRpc<C, Pool, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: StabilityRpcRuntimeApi<Block>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	Pool: sc_transaction_pool_api::TransactionPool<Block = Block> + Send + Sync + 'static,
{
	fn get_supported_tokens(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		let value = api
			.get_supported_tokens(at)
			.map_err(runtime_error_into_rpc_err);
		Ok(StabilityOutput {
			code: 200,
			value: value.unwrap(),
		})
	}

	fn get_validator_list(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		let value = api
			.get_validator_list(at)
			.map_err(runtime_error_into_rpc_err);
		Ok(StabilityOutput {
			code: 200,
			value: value.unwrap(),
		})
	}

	fn get_active_validator_list(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		let value = api
			.get_active_validator_list(at)
			.map_err(runtime_error_into_rpc_err);
		Ok(StabilityOutput {
			code: 200,
			value: value.unwrap(),
		})
	}

	async fn send_sponsored_transaction(
		&self,
		transaction: Bytes,
		meta_trx_sponsor: H160,
		meta_trx_sponsor_signature: Bytes,
	) -> RpcResult<H256> {
		let block_hash = self.client.info().best_hash;

		let slice = &transaction.0[..];
		if slice.is_empty() {
			return Err(ErrorObject::owned(
				1,
				format!("Invalid raw transaction"),
				None::<()>,
			));
		}

		let transaction: ethereum::TransactionV2 = match ethereum::EnvelopedDecodable::decode(slice)
		{
			Ok(transaction) => transaction,
			Err(_) => {
				return Err(ErrorObject::owned(
					1,
					format!("Invalid raw transaction"),
					None::<()>,
				))
			}
		};

		let extrinsic = self
			.client
			.runtime_api()
			.convert_sponsored_transaction(
				block_hash,
				transaction.clone(),
				meta_trx_sponsor,
				meta_trx_sponsor_signature.to_vec(),
			)
			.unwrap();

		let transaction_hash = transaction.hash();

		self.pool
			.submit_one(block_hash, TransactionSource::Local, extrinsic)
			.map_ok(move |_| transaction_hash)
			.map_err(|e| {
				ErrorObject::owned(
					1,
					format!("Unable to submit transaction: {:?}", e),
					None::<()>,
				)
			})
			.await
	}
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcResult<String> {
	Err(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
}
