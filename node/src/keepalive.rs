use account::EthereumSigner;
use futures::prelude::*;
use log::info;
use sc_client_api::HeaderBackend;
use sc_service::error::Error as ServiceError;
use sc_transaction_pool_api::TransactionSource;
use sp_api::ProvideRuntimeApi;
use sp_core::crypto::KeyTypeId;
use sp_keystore::SyncCryptoStore;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::traits::IdentifyAccount;
use stability_runtime::AccountId;
use stbl_core_primitives::Block;
use stbl_primitives_validator_health::ValidatorHealth;
use std::sync::Arc;

pub struct KeepAlive<A, TP> {
	pub client: Arc<A>,
	pub pool: Arc<TP>,
	pub keystore: SyncCryptoStorePtr,
}

pub async fn do_validator_available_again<A, TP>(
	KeepAlive {
		client,
		pool,
		keystore,
	}: KeepAlive<A, TP>,
) where
	Block: BlockT,
	A: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
	A::Api: stbl_primitives_validator_health::ValidatorHealth<Block, AccountId>,
	TP: sc_transaction_pool_api::TransactionPool<Block = Block> + 'static,
{
	{
		info!("🥲 Starting KeepAlive");

		let api = client.runtime_api();
		let block_hash = BlockId::hash(client.info().best_hash);

		// Validator keys
		let keys =
			SyncCryptoStore::ecdsa_public_keys(&*keystore, KeyTypeId::try_from("aura").unwrap());

		// Retrieve validator keys
		let signer = EthereumSigner::from(keys[0]);
		let val_account_id = signer.into_account();

		info!("🥲 Signer: {:?}", val_account_id.clone());
		info!("🥲 KEys: {:?}", keys.len());
		// Get message for the node validator account
		let message = api
			.generate_validator_message(&block_hash, val_account_id)
			.unwrap();
		info!("🥲 Message: {:?}", message.clone());
		let encoded_message = stbl_tools::misc::kecckak256(&message);

		info!("🥲 tengo sida");

		let tmp_signature = SyncCryptoStore::ecdsa_sign_prehashed(
			&*keystore,
			KeyTypeId::try_from("aura").unwrap(),
			&keys[0],
			encoded_message.as_fixed_bytes(),
		)
		.unwrap();
		info!("🥲 signature: {:?}", tmp_signature.clone());

		let sig = tmp_signature.unwrap();
		let extrinsic = api
			.convert_add_validator_again_transaction(&block_hash, val_account_id, sig.0.into())
			.unwrap();

		info!("🥲 extrinsic {:?}", extrinsic.clone());
		// Submit transaction
		let _submitted_transaction = pool
			.submit_one(&block_hash, TransactionSource::Local, extrinsic)
			.map_ok(move |_| {
				info!("🥲 Transaction submitted successfully");
			})
			.map_err(|e| {
				info!("🥲🥲🥲 Error submitting transaction: {:?}", e);
				ServiceError::Other("Error submitting transaction".into())
			})
			.await;
	}
}
