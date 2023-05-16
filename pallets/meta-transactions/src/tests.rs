use ethereum::TransactionRecoveryId;
use sp_core::{H160, H256};
use sp_runtime::{transaction_validity::TransactionValidity, DispatchError};
use stbl_tools::eth::recover_signer;

use crate::mock::{
	new_test_ext, MetaTransaction0Signature, MetaTransaction1Signature, RawTransaction0,
	RawTransaction1, Runtime, Sponsor,
};

#[test]
fn fail_to_execute_meta_transaction_twice() {
	new_test_ext().execute_with(|| {
		let trx0 = get_transaction_from_bytes(RawTransaction0::get());

		let from = recover_signer(&trx0).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			0u64,
			MetaTransaction0Signature::get(),
		)
		.unwrap();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			0u64,
			MetaTransaction0Signature::get(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Invalid metatransaction nonce")
		));
	});
}

#[test]
fn fail_to_execute_meta_transaction_with_wrong_meta_nonce() {
	new_test_ext().execute_with(|| {
		let trx1 = get_transaction_from_bytes(RawTransaction1::get());

		let from = recover_signer(&trx1).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx1.clone(),
			Sponsor::get(),
			1u64,
			MetaTransaction1Signature::get(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Invalid metatransaction nonce")
		));
	});
}

#[test]
fn fail_to_execute_meta_transaction_twice_with_invalid_trx() {
	new_test_ext().execute_with(|| {
		let trx0 = get_transaction_from_bytes(RawTransaction0::get());

		let from = recover_signer(&trx0).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		pallet_ethereum::Pallet::<Runtime>::transact(origin.clone(), trx0.clone()).unwrap();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			0u64,
			MetaTransaction0Signature::get(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Transaction object is invalid")
		));
	});
}

#[test]
fn fail_to_execute_meta_transaction_twice_with_invalid_trx_signature() {
	new_test_ext().execute_with(|| {
		let trx0 = ethereum::TransactionV2::Legacy(ethereum::LegacyTransaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas_limit: 0.into(),
			action: ethereum::TransactionAction::Call(H160::zero()),
			value: 0.into(),
			input: Vec::new(),
			signature: ethereum::TransactionSignature::new(
				27,
				H256::from_low_u64_be(10),
				H256::from_low_u64_be(10),
			)
			.unwrap(),
		});

		let from = H160::zero();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		pallet_ethereum::Pallet::<Runtime>::transact(origin.clone(), trx0.clone()).unwrap();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			0u64,
			MetaTransaction0Signature::get(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Invalid transaction signature")
		));
	});
}

#[test]
fn fail_to_execute_meta_transaction_with_wrong_meta_signature() {
	new_test_ext().execute_with(|| {
		let trx1 = get_transaction_from_bytes(RawTransaction1::get());

		let from = recover_signer(&trx1).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx1.clone(),
			Sponsor::get(),
			0u64,
			MetaTransaction1Signature::get(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Invalid metatransaction signature")
		));
	});
}

fn get_transaction_from_bytes(trx_bytes: Vec<u8>) -> pallet_ethereum::Transaction {
	ethereum::EnvelopedDecodable::decode(trx_bytes.as_slice()).unwrap()
}
