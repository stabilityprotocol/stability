 use jsonrpsee::{core::RpcResult, proc_macros::rpc};
 use std::sync::Arc;
 use sc_transaction_pool_api::{TransactionPool};

 #[rpc(client, server)]
 pub trait DelegatedTransactionRpc {	
	#[method(name = "execute_delegated_transaction")]
	fn execute_delegated_transaction(&self) -> RpcResult<u64>;
 }

 pub struct DelegatedTransaction<C,P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>
 }

 impl<C,P> DelegatedTransaction<C,P> {
	pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
		Self {client, _marker: Default::default()}
	}
 }

 impl<C,P> DelegatedTransactionRpcServer for DelegatedTransaction<C,P>
 where
	 C: Send + Sync + 'static,
	 P: TransactionPool +  'static,
 {
	fn execute_delegated_transaction(&self) -> RpcResult<u64> {
		Ok(0)
	}
 }