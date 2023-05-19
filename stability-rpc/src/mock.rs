use sc_transaction_pool_api::InPoolTransaction;
use sp_api::{ApiError, ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::H160;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, NumberFor, Zero},
};
use substrate_test_runtime_client::runtime::{Block, Extrinsic, Hash};

pub struct TestApi {}

pub struct TestRuntimeApi {}

impl ProvideRuntimeApi<Block> for TestApi {
	type Api = TestRuntimeApi;

	fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
		TestRuntimeApi {}.into()
	}
}

pub struct MockedMempool;
impl Default for MockedMempool {
	fn default() -> Self {
		Self {}
	}
}

pub struct MockTransaction;
impl InPoolTransaction for MockTransaction {
	type Transaction = Extrinsic;

	type Hash = Hash;

	fn data(&self) -> &Self::Transaction {
		todo!()
	}

	fn hash(&self) -> &Self::Hash {
		todo!()
	}

	fn priority(&self) -> &sc_transaction_pool_api::TransactionPriority {
		todo!()
	}

	fn longevity(&self) -> &sc_transaction_pool_api::TransactionLongevity {
		todo!()
	}

	fn requires(&self) -> &[sc_transaction_pool_api::TransactionTag] {
		todo!()
	}

	fn provides(&self) -> &[sc_transaction_pool_api::TransactionTag] {
		todo!()
	}

	fn is_propagable(&self) -> bool {
		todo!()
	}
}

impl sc_service::TransactionPool for MockedMempool {
	type Block = Block;

	type Hash = Hash;

	type InPoolTransaction = MockTransaction;

	type Error = sc_transaction_pool_api::error::Error;

	fn submit_at(
		&self,
		at: &BlockId<Self::Block>,
		source: sc_transaction_pool_api::TransactionSource,
		xts: Vec<sc_transaction_pool_api::TransactionFor<Self>>,
	) -> sc_transaction_pool_api::PoolFuture<
		Vec<Result<sc_transaction_pool_api::TxHash<Self>, Self::Error>>,
		Self::Error,
	> {
		todo!()
	}

	fn submit_one(
		&self,
		at: &BlockId<Self::Block>,
		source: sc_transaction_pool_api::TransactionSource,
		xt: sc_transaction_pool_api::TransactionFor<Self>,
	) -> sc_transaction_pool_api::PoolFuture<sc_transaction_pool_api::TxHash<Self>, Self::Error> {
		todo!()
	}

	fn submit_and_watch(
		&self,
		at: &BlockId<Self::Block>,
		source: sc_transaction_pool_api::TransactionSource,
		xt: sc_transaction_pool_api::TransactionFor<Self>,
	) -> sc_transaction_pool_api::PoolFuture<
		std::pin::Pin<Box<sc_transaction_pool_api::TransactionStatusStreamFor<Self>>>,
		Self::Error,
	> {
		todo!()
	}

	fn ready_at(
		&self,
		at: NumberFor<Self::Block>,
	) -> std::pin::Pin<
		Box<
			dyn futures_util::Future<
					Output = Box<
						dyn sc_transaction_pool_api::ReadyTransactions<
								Item = sc_service::Arc<Self::InPoolTransaction>,
							> + Send,
					>,
				> + Send,
		>,
	> {
		todo!()
	}

	fn ready(
		&self,
	) -> Box<
		dyn sc_transaction_pool_api::ReadyTransactions<
				Item = sc_service::Arc<Self::InPoolTransaction>,
			> + Send,
	> {
		todo!()
	}

	fn remove_invalid(
		&self,
		hashes: &[sc_transaction_pool_api::TxHash<Self>],
	) -> Vec<sc_service::Arc<Self::InPoolTransaction>> {
		todo!()
	}

	fn status(&self) -> sc_transaction_pool_api::PoolStatus {
		sc_transaction_pool_api::PoolStatus {
			ready: 0,
			ready_bytes: 0,
			future: 0,
			future_bytes: 0,
		}
	}

	fn import_notification_stream(
		&self,
	) -> sc_transaction_pool_api::ImportNotificationStream<sc_transaction_pool_api::TxHash<Self>> {
		todo!()
	}

	fn on_broadcasted(
		&self,
		propagations: std::collections::HashMap<sc_transaction_pool_api::TxHash<Self>, Vec<String>>,
	) {
		// std::collections::HashMap::<sc_transaction_pool_api::TxHash<Self>, Vec<String>>::new();
		todo!()
	}

	fn hash_of(
		&self,
		xt: &sc_transaction_pool_api::TransactionFor<Self>,
	) -> sc_transaction_pool_api::TxHash<Self> {
		todo!()
	}

	fn ready_transaction(
		&self,
		hash: &sc_transaction_pool_api::TxHash<Self>,
	) -> Option<sc_service::Arc<Self::InPoolTransaction>> {
		None
	}
}

/// Blockchain database header backend. Does not perform any validation.
impl<Block: BlockT> HeaderBackend<Block> for TestApi {
	fn header(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<Option<Block::Header>, sp_blockchain::Error> {
		Ok(None)
	}

	fn info(&self) -> sc_client_api::blockchain::Info<Block> {
		sc_client_api::blockchain::Info {
			best_hash: Default::default(),
			best_number: Zero::zero(),
			finalized_hash: Default::default(),
			finalized_number: Zero::zero(),
			genesis_hash: Default::default(),
			number_leaves: Default::default(),
			finalized_state: None,
			block_gap: None,
		}
	}

	fn status(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
		Ok(sc_client_api::blockchain::BlockStatus::Unknown)
	}

	fn number(
		&self,
		_hash: Block::Hash,
	) -> std::result::Result<Option<NumberFor<Block>>, sp_blockchain::Error> {
		Ok(None)
	}

	fn hash(
		&self,
		_number: NumberFor<Block>,
	) -> std::result::Result<Option<Block::Hash>, sp_blockchain::Error> {
		Ok(None)
	}
}
