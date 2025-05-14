// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use crate::{mock::*, *};

use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};
use substrate_test_runtime_client::runtime::BlockNumber;

fn precompiles() -> Precompiles<Test> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::owner_selectors().contains(&0x8da5cb5b));
	assert!(PCall::pending_owner_selectors().contains(&0xe30c3978));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf2fde38b));
	assert!(PCall::claim_ownership_selectors().contains(&0x79ba5097));
	assert!(PCall::set_application_block_selectors().contains(&0x9b31c472));
	assert!(PCall::reject_proposed_code_selectors().contains(&0xec1820fd));

	assert_eq!(
		crate::SELECTOR_LOG_NEW_OWNER,
		&Keccak256::digest(b"NewOwner(address)")[..]
	);

	assert_eq!(
		crate::SELECTOR_SETTED__APPLICATION_BLOCK,
		&Keccak256::digest(b"SettedApplicationBlock(uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_CODE_PROPOSED_REJECTED,
		&Keccak256::digest(b"CodeProposedRejected()")[..]
	);
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), CryptoAlith, Precompile1);

		tester.test_view_modifier(PCall::owner_selectors());
		tester.test_view_modifier(PCall::pending_owner_selectors());
		tester.test_default_modifier(PCall::transfer_ownership_selectors());
		tester.test_default_modifier(PCall::claim_ownership_selectors());
		tester.test_default_modifier(PCall::set_application_block_selectors());
		tester.test_default_modifier(PCall::reject_proposed_code_selectors());
	});
}

#[test]
fn owner_correctly_init() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::owner {})
			.execute_returns(Into::<H256>::into(DefaultOwner::get()));
	})
}

parameter_types! {
	pub UnpermissionedAccount:H160 = H160::from_str("0x1000000000000000000000000000000000000000").expect("invalid address");
	pub UnpermissionedAccount2:H160 = H160::from_str("0x2000000000000000000000000000000000000000").expect("invalid address");
	pub NewMember:H160 = H160::from_str("0xaafB45fB0581FC07C0F07b04b730a303469548Dc").expect("invalid address");
	pub NotOwner:H160 = H160::from_str("0x9A34381dd72d7F7ED4be83Bf31593243196f5464").expect("invalid address");
}

#[test]

fn transfer_ownership_set_target_if_owner_twice() {
	ExtBuilder::default().build().execute_with(|| {
		let new_owner = UnpermissionedAccount::get();
		let other_owner = UnpermissionedAccount2::get();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns(Into::<H256>::into(new_owner));

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(other_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns(Into::<H256>::into(other_owner));
	})
}

#[test]
fn fail_transfer_ownership_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		let new_owner = UnpermissionedAccount::get();

		precompiles()
			.prepare_test(
				new_owner,
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	})
}

#[test]
fn fail_claim_ownership_if_not_claimable() {
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"target owner is not the claimer"))
	});
}

#[test]
fn claim_ownership_if_claimable() {
	let owner = DefaultOwner::get();
	let new_owner = UnpermissionedAccount::get();
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				owner,
				Precompile1,
				PCall::transfer_ownership {
					new_owner: solidity::codec::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.expect_log(log1(
				Precompile1,
				SELECTOR_LOG_NEW_OWNER,
				solidity::encode_event_data(Into::<H256>::into(new_owner)),
			))
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::owner {})
			.execute_returns(Into::<H256>::into(new_owner));
	});
}

#[test]
fn test_set_block_application() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec(),
		)
		.unwrap();

		let block_number = 100u32;
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_application_block {
					block_number: block_number,
				},
			)
			.expect_log(log1(
				Precompile1,
				SELECTOR_SETTED__APPLICATION_BLOCK,
				solidity::encode_event_data(Into::<U256>::into(block_number)),
			))
			.execute_some();

		assert_eq!(
			UpgradeRuntimeProposal::get_application_block_number().unwrap(),
			BlockNumber::from(block_number)
		);
	});
}

#[test]
fn test_fail_set_block_application_if_not_owner() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec(),
		)
		.unwrap();

		let block_number = 100u32;
		precompiles()
			.prepare_test(
				UnpermissionedAccount::get(),
				Precompile1,
				PCall::set_application_block {
					block_number: block_number,
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

#[test]
fn test_reject_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec(),
		)
		.unwrap();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::reject_proposed_code {},
			)
			.expect_log(log0(Precompile1, SELECTOR_CODE_PROPOSED_REJECTED))
			.execute_some();

		assert!(UpgradeRuntimeProposal::get_proposed_code().is_none());
		assert!(UpgradeRuntimeProposal::get_application_block_number().is_none());
	});
}

#[test]
fn test_fail_reject_proposed_code_if_not_owner() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec(),
		)
		.unwrap();

		precompiles()
			.prepare_test(
				UnpermissionedAccount::get(),
				Precompile1,
				PCall::reject_proposed_code {},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

#[test]
fn test_add_member_to_council() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.expect_log(log1(
				Precompile1,
				SELECTOR_MEMBER_ADDED,
				solidity::encode_event_data(Into::<Address>::into(NewMember::get())),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::get_technical_committee_members {},
			)
			.execute_returns(vec![Into::<Address>::into(NewMember::get())]);
	});
}

#[test]
fn test_remove_member_from_council() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::get_technical_committee_members {},
			)
			.execute_returns(vec![Into::<Address>::into(NewMember::get())]);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::remove_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.expect_log(log1(
				Precompile1,
				SELECTOR_MEMBER_REMOVED,
				solidity::encode_event_data(Into::<Address>::into(NewMember::get())),
			))
			.execute_some();

		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::get_technical_committee_members {},
			)
			.execute_returns(Vec::<Address>::new());
	});
}

#[test]
fn test_add_member_to_council_fails_if_not_owner() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

#[test]
fn test_remove_member_from_council_fails_if_not_owner() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::remove_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

#[test]
fn test_add_member_to_council_fails_if_already_member() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::get_technical_committee_members {},
			)
			.execute_returns(vec![Into::<Address>::into(NewMember::get())]);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"already a member"));
	});
}

#[test]
fn test_remove_member_from_council_fails_if_not_member() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::add_member_to_technical_committee {
					member: NewMember::get().into(),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::get_technical_committee_members {},
			)
			.execute_returns(vec![Into::<Address>::into(NewMember::get())]);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::remove_member_to_technical_committee {
					member: NotOwner::get().into(),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"not a member"));
	});
}

#[test]
fn test_get_hash_of_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = ExtBuilder::default().build();

	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));
	ext.execute_with(|| {
		UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec(),
		)
		.unwrap();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::get_hash_of_proposed_code {},
			)
			.execute_some();
	});
}
