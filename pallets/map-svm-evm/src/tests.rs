use core::str::FromStr;

use crate::mock::{SmartcontractErc1271Fails, SmartcontractErc1271Success, SmartcontractWithoutErc721};

use super::*;
use hex::FromHex;
use mock::{new_test_ext, AccountId, MapSvmEvm, RuntimeOrigin, Test};
use once_cell::sync::Lazy;

const TEST_ACCOUNT_ALICE_SUBSTRATE: Lazy<AccountId> =
	Lazy::new(|| AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap());

static TEST_ACCOUNT_ALICE_EVM: Lazy<H160> =
	Lazy::new(|| H160::from_str("0xce165b22a1f815e8b703fcb25d188cd648d7e91e").unwrap());

static ALICE_LINK_MESSAGE_NONCE_0: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("779c35319c85884017413a8a4d3ccc0de35842699d3010c914af2fc1eb0e94223cdcad1e503ee31654da002eac14d8015c406c31cd8c29d24afaf28d9aebe77b1c").unwrap()
});
static ALICE_LINK_MESSAGE_NONCE_1: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("fbf77ed983e24d9e6f0af223bae81b915dc6eec215430113929f772f5bb74e4744fe1f0fddbf28ecdbbe03559ef81cdb2c31aa9702e819c4a47ad50d94b27f601b").unwrap()
});

const TEST_ACCOUNT_BOB_SUBSTRATE: Lazy<AccountId> =
	Lazy::new(|| AccountId::from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap());

static TEST_ACCOUNT_BOB_EVM: Lazy<H160> =
	Lazy::new(|| H160::from_str("0x8eb92711aa4fa5d54fd41db4a47c012bd66783cf").unwrap());

static BOB_LINK_MESSAGE_NONCE_0: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("b0e928803ee2c4ab63b28062b0015f74c812f3be878152437bf3425de372fd8549152776e7e0c0702ecbcb4c7c4e60cba40ece061e5ae13d2228c3cc285ef0551b").unwrap()
});

#[test]
fn test_setup_works() {
	new_test_ext(vec![]).execute_with(|| {
		assert!(MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()).is_none())
	});
}

#[test]
fn test_genesis_config() {
	new_test_ext(vec![(
		TEST_ACCOUNT_ALICE_SUBSTRATE.clone(),
		TEST_ACCOUNT_ALICE_EVM.clone(),
	)])
	.execute_with(|| {
		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		);

		assert_eq!(MapSvmEvm::evm_link_nonce(TEST_ACCOUNT_ALICE_EVM.clone()), 1);
	});
}

#[test]
fn test_link_evm_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		)
	})
}

#[test]
fn fails_if_received_zero_address() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			ZeroAddress::get(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap_err();


		assert_eq!(err, Error::<Test>::InvalidAddress.into());
	})
}

#[test]
fn fails_if_substrate_account_is_already_linked() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_BOB_EVM.clone(),
			BOB_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::SubstrateAlreadyLinked.into());
	})
}

#[test]
fn fails_if_evm_account_is_already_linked() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_BOB_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_1.clone(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::EvmAlreadyLinked.into());
	})
}

#[test]
fn fails_if_nonce_of_message_is_not_the_expected() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_1.clone(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::AddressNotMatch.into());
	})
}

#[test]
fn fails_if_signature_is_invalid() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			Vec::<u8>::from_hex("405045").unwrap(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::InvalidSignature.into());
	})
}

#[test]
fn unlink_evm_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()))
			.unwrap();

		assert!(MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()).is_none())
	})
}

#[test]
fn fails_if_substrate_account_is_not_linked() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(
			TEST_ACCOUNT_ALICE_SUBSTRATE.clone(),
		))
		.unwrap_err();

		assert_eq!(err, Error::<Test>::AccountNotLinked.into());
	})
}

#[test]
fn unlink_evm_account_and_then_link_to_another_evm_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()))
			.unwrap();

		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_BOB_EVM.clone(),
			BOB_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			Some(TEST_ACCOUNT_BOB_EVM.clone())
		)
	})
}

#[test]
fn unlink_substrate_account_and_then_link_to_another_substrate_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()))
			.unwrap();

		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_BOB_SUBSTRATE.clone()),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_1.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_BOB_SUBSTRATE.clone()),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		)
	})
}

#[test]
fn link_to_evm_contract() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			SmartcontractErc1271Success::get(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			Some(SmartcontractErc1271Success::get())
		)
	})
}

#[test]
fn link_to_evm_contract_fails() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			SmartcontractErc1271Fails::get(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::InvalidSignature.into());
	})
}

#[test]
fn link_to_evm_contract_fails_if_contract_doesnt_implement_erc1271() {
	new_test_ext(vec![]).execute_with(|| {
		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE.clone()),
			SmartcontractWithoutErc721::get(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap_err();

		assert_eq!(err, Error::<Test>::InvalidSignature.into());
	})
}
