 use jsonrpsee::{core::RpcResult, proc_macros::rpc};
 use std::sync::Arc;
 use sc_transaction_pool_api::{InPoolTransaction, TransactionPool};
 use delegate_transaction::{pallet};

 #[rpc(client, server)]
 pub trait DelegatedTransactionRpc {
	#[method(name = "delegate_transaction")]
	fn delegate_transaction(
		&self, 
		to: H160,
		input: Vec<u8>,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
	) -> RpcResult<u64>;
	
	
	#[method(name = "execute_delegated_transaction")]
	fn execute_delegated(&self, nonce: u64) -> RpcResult<u64>;
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
	fn delegate_transaction(
		&self,
		to: H160,
		input: Vec<u8>,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
	) -> RpcResult<u64> {
		let outcome = pallet::Pallet::add_transaction(to, input, gas_limit, max_fee_per_gas, max_priority_fee_per_gas);
	}

	fn execute_delegated_transaction(&self, nonce: u64) -> RpcResult<u64> {
		let outcome = pallet::Pallet::execute_transaction(nonce);
	}
 }