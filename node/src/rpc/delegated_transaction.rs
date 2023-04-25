 use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use sp_core::H160;
 use std::sync::Arc;
 use sc_transaction_pool_api::{TransactionPool};
 use pallet_delegated_transaction::{pallet};

 #[rpc(client, server)]
 pub trait DelegatedTransactionRpc {	
	#[method(name = "delegate_transaction")]
	fn delegate_transaction(&self) -> RpcResult<([u8; 32], u64)>;
	
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
	fn delegate_transaction(&self, to: H160, input: Vec<u8>) -> RpcResult<([u8; 32], u64)> {
		pallet::Pallet::<C,P>::delegate_transaction(to, input);
		
		Ok(([0; 32], 0))
	}

	fn execute_delegated_transaction(&self) -> RpcResult<u64> {
		Ok(0)
	}
 }