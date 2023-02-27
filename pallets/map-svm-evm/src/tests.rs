use core::str::FromStr;

use super::*;
use hex::FromHex;
use mock::{new_test_ext, MapSvmEvm, RuntimeOrigin, Test};
use once_cell::sync::Lazy;

const TEST_ACCOUNT_ALICE_SUBSTRATE: u64 = 1;

static TEST_ACCOUNT_ALICE_EVM: Lazy<H160> =
	Lazy::new(|| H160::from_str("0x71A66452Ca097becB4a09e8Ec56F617cC8fc2860").unwrap());

static ALICE_LINK_MESSAGE_NONCE_0: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("647e3b223a551b583170defc55e6904115a74a9aa6a9ca2a90499ed8e9107e12667ff322473ca09cef7ea946104abf2818075e827b102a274c8ff059ad3caf291b").unwrap()
});
static ALICE_LINK_MESSAGE_NONCE_1: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("0e8132d50b5a5e242fddedf5c28234c761f8a19682f7191ef194b0b6e708451f4f67fafab2586d75690482dc6fc47365ef61289187dc7854073412eff2eeb2281b").unwrap()
});

const TEST_ACCOUNT_BOB_SUBSTRATE: u64 = 2;

static TEST_ACCOUNT_BOB_EVM: Lazy<H160> =
	Lazy::new(|| H160::from_str("0x6716c927e927f1c258E82ff30aEc98E0EdC51994").unwrap());

static BOB_LINK_MESSAGE_NONCE_0: Lazy<Vec<u8>> = Lazy::new(|| {
	Vec::<u8>::from_hex("3045bd99debf25cbcf9d7a6aef7e7f1ca808cfce3d1f188e56c52260169a9f83564841dd126636a0dac7cb775e54a467cd7295ba4178b83c3a8d94c75930f9b81b").unwrap()
});

#[test]
fn test_setup_works() {
	new_test_ext(vec![]).execute_with(|| {
		assert!(MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE).is_none())
	});
}

#[test]
fn test_genesis_config() {
	new_test_ext(vec![(
		TEST_ACCOUNT_ALICE_SUBSTRATE,
		TEST_ACCOUNT_ALICE_EVM.clone(),
	)])
	.execute_with(|| {
		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		);

		assert_eq!(MapSvmEvm::evm_link_nonce(TEST_ACCOUNT_ALICE_EVM.clone()), 1);
	});
}

#[test]
fn test_link_evm_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		)
	})
}

#[test]
fn fails_if_substrate_account_is_already_linked() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
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
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		let err = MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_BOB_SUBSTRATE),
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
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
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
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
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
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE)).unwrap();

		assert!(MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE).is_none())
	})
}

#[test]
fn fails_if_substrate_account_is_not_linked() {
	new_test_ext(vec![]).execute_with(|| {
		let err =
			MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE))
				.unwrap_err();

		assert_eq!(err, Error::<Test>::AccountNotLinked.into());
	})
}

#[test]
fn unlink_evm_account_and_then_link_to_another_evm_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE)).unwrap();

		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_BOB_EVM.clone(),
			BOB_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_ALICE_SUBSTRATE),
			Some(TEST_ACCOUNT_BOB_EVM.clone())
		)
	})
}

#[test]
fn unlink_substrate_account_and_then_link_to_another_substrate_account() {
	new_test_ext(vec![]).execute_with(|| {
		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_0.clone(),
		)
		.unwrap();

		MapSvmEvm::unlink_evm_account(RuntimeOrigin::signed(TEST_ACCOUNT_ALICE_SUBSTRATE)).unwrap();

		MapSvmEvm::link_evm_account(
			RuntimeOrigin::signed(TEST_ACCOUNT_BOB_SUBSTRATE),
			TEST_ACCOUNT_ALICE_EVM.clone(),
			ALICE_LINK_MESSAGE_NONCE_1.clone(),
		)
		.unwrap();

		assert_eq!(
			MapSvmEvm::get_linked_evm_account(TEST_ACCOUNT_BOB_SUBSTRATE),
			Some(TEST_ACCOUNT_ALICE_EVM.clone())
		)
	})
}
