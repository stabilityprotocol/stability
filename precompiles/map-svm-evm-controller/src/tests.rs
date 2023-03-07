use crate::{mock::*, *};
use codec::Encode;
use once_cell::sync::Lazy;
use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};
use sp_core::H160;
use std::str::FromStr;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

static TEST_ACCOUNT_ALICE_SUBSTRATE: Lazy<AccountId> =
	Lazy::new(|| AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap());

static TEST_ACCOUNT_ALICE_EVM: Lazy<H160> =
	Lazy::new(|| H160::from_str("0x71A66452Ca097becB4a09e8Ec56F617cC8fc2860").unwrap());

#[test]
fn selectors() {
	assert!(PCall::link_of_selectors().contains(&0x529a5e26));
	assert!(PCall::un_link_selectors().contains(&0xf5b62a58));

	assert_eq!(
		crate::SELECTOR_LOG_ACCOUNT_UNLINKED,
		&Keccak256::digest(b"AccountUnlinked(address,bytes32)")[..]
	);
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), CryptoAlith, Precompile1);

		tester.test_view_modifier(PCall::link_of_selectors());
		tester.test_default_modifier(PCall::un_link_selectors());
	});
}

#[test]
fn test_link_of() {
	ExtBuilder::default()
		.with_linked_accounts(vec![(
			TEST_ACCOUNT_ALICE_SUBSTRATE.clone(),
			TEST_ACCOUNT_ALICE_EVM.clone(),
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					TEST_ACCOUNT_ALICE_EVM.clone(),
					Precompile1,
					PCall::link_of {
						address: TEST_ACCOUNT_ALICE_EVM.clone().into(),
					},
				)
				.execute_returns_encoded(H256::from_slice(
					TEST_ACCOUNT_ALICE_SUBSTRATE.encode().as_slice(),
				))
		});
}

#[test]
fn test_link_of_fails_if_account_is_not_linked() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				TEST_ACCOUNT_ALICE_EVM.clone(),
				Precompile1,
				PCall::link_of {
					address: TEST_ACCOUNT_ALICE_EVM.clone().into(),
				},
			)
			.execute_reverts(|output| output == b"EVM account not linked")
	});
}

#[test]
fn test_un_link() {
	ExtBuilder::default()
		.with_linked_accounts(vec![(
			TEST_ACCOUNT_ALICE_SUBSTRATE.clone(),
			TEST_ACCOUNT_ALICE_EVM.clone(),
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					TEST_ACCOUNT_ALICE_EVM.clone(),
					Precompile1,
					PCall::un_link {},
				)
				.expect_log(log2(
					Precompile1,
					SELECTOR_LOG_ACCOUNT_UNLINKED,
					TEST_ACCOUNT_ALICE_EVM.clone(),
					EvmDataWriter::new()
						.write(H256::from_slice(
							TEST_ACCOUNT_ALICE_SUBSTRATE.encode().as_slice(),
						))
						.build(),
				))
				.execute_returns_encoded(true);

			assert!(
				MapSvmEvm::get_linked_substrate_account(TEST_ACCOUNT_ALICE_EVM.clone()).is_none()
			)
		});
}

#[test]
fn test_un_link_reverts_if_sender_is_not_linked() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				TEST_ACCOUNT_ALICE_EVM.clone(),
				Precompile1,
				PCall::un_link {},
			)
			.execute_reverts(|output| output == b"Unlink failed");
	});
}
