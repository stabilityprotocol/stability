use futures_util::TryFutureExt;
use jsonrpsee::{
	core::{error, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use rpc_eth_extension_api::EthExtensionRpcApi;
use sc_transaction_pool_api::TransactionSource;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{Bytes, H160, H256};
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
pub use stability_rpc_api::StabilityRpcApi as StabilityRpcRuntimeApi;
use std::{
	str::{self},
	sync::Arc,
};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

#[derive(serde::Deserialize, serde::Serialize)]
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

	#[method(name = "stability_sendSponsoredTransaction")]
	async fn send_sponsored_transaction(
		&self,
		transaction_req: Bytes,
		meta_trx_nonce: u64,
		meta_trx_sponsor: H160,
		meta_trx_sponsor_signature: Bytes,
	) -> RpcResult<H256>;

	#[method(name = "stability_getSponsorNonce")]
	async fn get_sponsor_nonce(&self, address: H160) -> RpcResult<u64>;
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
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block> + rpc_eth_extension_api::EthExtensionRpcApi<Block>,
	Pool: sc_transaction_pool_api::TransactionPool<Block = Block> + Send + Sync + 'static,
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

	fn get_validator_list(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<StabilityOutput<Vec<H160>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let value = api
			.get_validator_list(&at)
			.map_err(runtime_error_into_rpc_err);
		Ok(StabilityOutput {
			code: 200,
			value: value.unwrap(),
		})
	}

	async fn send_sponsored_transaction(
		&self,
		transaction: Bytes,
		meta_trx_nonce: u64,
		meta_trx_sponsor: H160,
		meta_trx_sponsor_signature: Bytes,
	) -> RpcResult<H256> {
		let block_hash = BlockId::hash(self.client.info().best_hash);

		let slice = &transaction.0[..];
		if slice.is_empty() {
			return Err(error::Error::Custom("Invalid raw transaction".into()));
		}

		let transaction: ethereum::TransactionV2 = match ethereum::EnvelopedDecodable::decode(slice)
		{
			Ok(transaction) => transaction,
			Err(_) => return Err(error::Error::Custom("Invalid raw transaction".into())),
		};

		let extrinsic = self
			.client
			.runtime_api()
			.convert_sponsored_transaction(
				&block_hash,
				transaction.clone(),
				meta_trx_nonce,
				meta_trx_sponsor,
				meta_trx_sponsor_signature.to_vec(),
			)
			.unwrap();

		let transaction_hash = transaction.hash();

		self.pool
			.submit_one(&block_hash, TransactionSource::Local, extrinsic)
			.map_ok(move |_| transaction_hash)
			.map_err(|e| {
				error::Error::Custom(format!("Unable to submit transaction: {:?}", e).into())
			})
			.await
	}

	async fn get_sponsor_nonce(&self, address: H160) -> RpcResult<u64> {
		let block_hash = BlockId::hash(self.client.info().best_hash);
		self.client
			.runtime_api()
			.get_sponsor_nonce(&block_hash, address)
			.map_err(|e| {
				error::Error::Custom(format!("Unable to get sponsor nonce: {:?}", e).into())
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
