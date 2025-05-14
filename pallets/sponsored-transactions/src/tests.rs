// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use sp_core::{H160, H256, U256};
use sp_runtime::DispatchError;
use stbl_tools::eth::recover_signer;

use crate::mock::{
	new_test_ext, MetaDeploymentSignature, MetaDeploymentTransaction, MetaTransaction0Signature,
	RawTransaction0, Runtime, Sponsor, StorageCalledArguments,
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
			MetaTransaction0Signature::get(),
		)
		.unwrap();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
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
			gas_price: 1_000_000_000.into(),
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
		let trx1 = get_transaction_from_bytes(RawTransaction0::get());

		let from = recover_signer(&trx1).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		let error = crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx1.clone(),
			Sponsor::get(),
			hex::decode("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap(),
		)
		.unwrap_err();

		assert!(matches!(
			error,
			DispatchError::Other("Invalid metatransaction signature")
		));
	});
}

// fee checks

#[test]
fn fees_managed_correctly_native_token_transaction() {
	new_test_ext().execute_with(|| {
		let trx0 = get_transaction_from_bytes(RawTransaction0::get());

		let from = recover_signer(&trx0).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			MetaTransaction0Signature::get(),
		)
		.unwrap();

		let fee_called_arguments = StorageCalledArguments::get();

		check_correct_fee_management(fee_called_arguments);
	});
}

#[test]
fn fees_managed_correctly_deploy_contract() {
	new_test_ext().execute_with(|| {
		let trx0 = get_transaction_from_bytes(MetaDeploymentTransaction::get());

		let from = recover_signer(&trx0).unwrap();
		let origin: <Runtime as frame_system::Config>::RuntimeOrigin =
			pallet_ethereum::Origin::EthereumTransaction(from).into();

		crate::Pallet::<Runtime>::send_sponsored_transaction(
			origin.clone(),
			trx0.clone(),
			Sponsor::get(),
			MetaDeploymentSignature::get(),
		)
		.unwrap();

		let fee_called_arguments = StorageCalledArguments::get();

		check_correct_fee_management(fee_called_arguments);
	});
}

// Utils

fn get_transaction_from_bytes(trx_bytes: Vec<u8>) -> pallet_ethereum::Transaction {
	ethereum::EnvelopedDecodable::decode(trx_bytes.as_slice()).unwrap()
}

fn check_correct_fee_management(called_arguments: Vec<(bool, H160, H160, U256)>) {
	let mut total_deposited = U256::from(0);
	let mut total_withdrawn = U256::from(0);
	for (is_deposit, token, _, amount) in called_arguments.iter() {
		assert!(token.eq(&called_arguments[0].1));
		if *is_deposit {
			total_deposited = amount.checked_add(total_deposited).unwrap();
		} else {
			total_withdrawn = amount.checked_add(total_withdrawn).unwrap();
		}
	}
	assert_eq!(total_deposited, total_withdrawn);
}
