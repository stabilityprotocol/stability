use account::EthereumSigner;
use futures::TryFutureExt;
use sc_client_api::HeaderBackend;
use sc_service::error::Error as ServiceError;
use sc_transaction_pool_api::TransactionSource;
use sp_api::ProvideRuntimeApi;
use sp_application_crypto::ecdsa::Public;
use sp_application_crypto::RuntimePublic;
use sp_core::crypto::KeyTypeId;
use sp_keystore::SyncCryptoStore;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::traits::IdentifyAccount;
use stability_runtime::AccountId;
use stbl_primitives_validator_health::ValidatorHealth;
use std::sync::Arc;

pub struct KeepAlive<A, TP, Block> {
	client: Arc<A>,
	pool: Arc<TP>,
	keystore: SyncCryptoStorePtr,
	_marker: std::marker::PhantomData<Block>,
}

impl<A, TP, Block> KeepAlive<A, TP, Block> {
	pub fn new(client: Arc<A>, pool: Arc<TP>, keystore: SyncCryptoStorePtr) -> Self {
		Self {
			client,
			pool,
			keystore,
			_marker: Default::default(),
		}
	}
}

#[async_trait::async_trait]
pub trait KeepAliveActions<BlockHash> {
	async fn do_validator_available_again(&self) -> Result<(), ServiceError>;
}

#[async_trait::async_trait]
impl<A, TP, Block> KeepAliveActions<<Block as BlockT>::Hash> for KeepAlive<A, TP, Block>
where
	Block: BlockT,
	A: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	A::Api: stbl_primitives_validator_health::ValidatorHealth<Block, AccountId>,
	TP: sc_transaction_pool_api::TransactionPool<Block = Block> + 'static,
{
	async fn do_validator_available_again(&self) -> Result<(), ServiceError> {
		let api = self.client.runtime_api();
		// Validator keys
		let keys = SyncCryptoStore::ecdsa_public_keys(
			&*self.keystore,
			KeyTypeId::try_from("aura").unwrap_or_default(),
		);

		// Retrieve validator keys
		let signer = EthereumSigner::from(keys[0]);
		let val_account_id = signer.into_account();
		let keypair: Public = keys[0].clone();

		// Get message for the node validator account
		let block_hash = BlockId::hash(self.client.info().best_hash);
		let message = api
			.generate_validator_message(&block_hash, val_account_id)
			.unwrap();
		let encoded_message = stbl_tools::misc::kecckak256(&message);
		// Generate signature for the message
		let signature = keypair.sign(
			KeyTypeId::try_from("aura").unwrap_or_default(),
			&encoded_message.as_fixed_bytes(),
		);

		match signature {
			Some(sig) => {
				let extrinsic = api
					.convert_add_validator_again_transaction(
						&block_hash,
						val_account_id,
						sig.0.into(),
					)
					.unwrap();
				// Submit transaction
				let submitted_transaction = self
					.pool
					.submit_one(&block_hash, TransactionSource::Local, extrinsic)
					.map_ok(move |_| {
						log::info!("Transaction submitted successfully");
					})
					.map_err(|e| {
						log::error!("Error submitting transaction: {:?}", e);
						ServiceError::Other("Error submitting transaction".into())
					});
				return submitted_transaction.await;
			}
			_ => Ok(()),
		}
	}
}
