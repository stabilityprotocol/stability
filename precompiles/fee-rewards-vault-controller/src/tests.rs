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

use precompile_utils::prelude::*;

use precompile_utils::testing::*;
use sha3::{Digest, Keccak256};

fn precompiles() -> Precompiles<Test> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::owner_selectors().contains(&0x8da5cb5b));
	assert!(PCall::pending_owner_selectors().contains(&0xe30c3978));
	assert!(PCall::transfer_ownership_selectors().contains(&0xf2fde38b));
	assert!(PCall::claim_ownership_selectors().contains(&0x79ba5097));
	assert!(PCall::claim_reward_selectors().contains(&0x4953c782));
	assert!(PCall::set_whitelist_selectors().contains(&0x9281aa0b));
	assert!(PCall::can_claim_reward_selectors().contains(&0xa4630c85));
	assert!(PCall::get_claimable_reward_selectors().contains(&0x21e91dea));
	assert!(PCall::is_whitelisted_selectors().contains(&0x3af32abf));

	assert_eq!(
		crate::SELECTOR_LOG_NEW_OWNER,
		&Keccak256::digest(b"NewOwner(address)")[..]
	);

	assert_eq!(
		crate::SELECTOR_REWARD_CLAIMED,
		&Keccak256::digest(b"RewardClaimed(address,address,address)")[..]
	);

	assert_eq!(
		crate::SELECTOR_WHITELIST_STATUS_UPDATED,
		&Keccak256::digest(b"WhitelistStatusUpdated(address,bool)")[..]
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
		tester.test_default_modifier(PCall::claim_reward_selectors());
		tester.test_default_modifier(PCall::set_whitelist_selectors());
		tester.test_view_modifier(PCall::can_claim_reward_selectors());
		tester.test_view_modifier(PCall::can_claim_reward_selectors());
		tester.test_view_modifier(PCall::get_claimable_reward_selectors());
		tester.test_view_modifier(PCall::is_whitelisted_selectors());
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
fn test_set_whitelisted() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_whitelisted {
					holder: SmartContractWithoutOwner::get().into(),
				},
			)
			.execute_returns(false);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithoutOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_whitelisted {
					holder: SmartContractWithoutOwner::get().into(),
				},
			)
			.execute_returns(true);
	});
}

#[test]
fn test_set_whitelisted_should_fail_if_address_is_not_smartcontract() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::is_whitelisted {
					holder: NotOwner::get().into(),
				},
			)
			.execute_returns(false);

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: NotOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"address is not a smartcontract"));
	});
}

#[test]
fn test_set_whitelisted_fails_if_sender_is_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				NotOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithoutOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

parameter_types! {
	pub Dapp1:H160 = H160::from_str("0x3000000000000000000000000000000000000000").expect("invalid address");
	pub Dapp2:H160 = H160::from_str("0x4000000000000000000000000000000000000000").expect("invalid address");
}

#[test]
fn test_can_claim_reward_returns_true_if_holder_and_claimant_are_equal() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithoutOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);
		precompiles()
			.prepare_test(
				SmartContractWithoutOwner::get(),
				Precompile1,
				PCall::can_claim_reward {
					claimant: SmartContractWithoutOwner::get().into(),
					holder: SmartContractWithoutOwner::get().into(),
				},
			)
			.execute_returns(true);
	});
}

#[test]
fn test_can_claim_reward_should_return_false_if_not_whitelisted() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Dapp1::get(),
				Precompile1,
				PCall::can_claim_reward {
					claimant: Dapp1::get().into(),
					holder: Dapp1::get().into(),
				},
			)
			.execute_returns(false);
	});
}

#[test]
fn test_can_claim_reward_should_return_true_if_claimant_is_validator() {
	ExtBuilder::default().build().execute_with(|| {
		let claimant = Validators::get()[0].into();
		precompiles()
			.prepare_test(
				claimant,
				Precompile1,
				PCall::can_claim_reward {
					claimant,
					holder: claimant,
				},
			)
			.execute_returns(true);
	})
}

#[test]
fn test_can_claim_reward_should_return_true_if_claimant_are_the_owner_of_the_dapp() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);
		precompiles()
			.prepare_test(
				OwnerOfDapp::get(),
				Precompile1,
				PCall::can_claim_reward {
					claimant: OwnerOfDapp::get().into(),
					holder: SmartContractWithOwner::get().into(),
				},
			)
			.with_subcall_handle(move |subcall| {
				let Subcall {
					address,
					transfer: _,
					input,
					target_gas: _,
					is_static,
					context,
				} = subcall;

				// Called on the behalf of the permit maker.
				assert_eq!(context.caller, Precompile1.into());
				assert_eq!(address, SmartContractWithOwner::get());
				assert_eq!(is_static, true);

				assert_eq!(input, vec![141, 165, 203, 91]);

				let mut output = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

				output.extend_from_slice(&OwnerOfDapp::get().as_bytes());

				SubcallOutput {
					output: output,
					cost: 1,
					..SubcallOutput::succeed()
				}
			})
			.execute_returns(true);
	});
}

#[test]
fn test_can_claim_reward_should_return_false_if_claimant_are_not_the_owner_of_the_dapp() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);
		precompiles()
			.prepare_test(
				OwnerOfDapp::get(),
				Precompile1,
				PCall::can_claim_reward {
					claimant: NotOwner::get().into(),
					holder: SmartContractWithOwner::get().into(),
				},
			)
			.with_subcall_handle(move |subcall| {
				let Subcall {
					address,
					transfer: _,
					input,
					target_gas: _,
					is_static,
					context,
				} = subcall;

				// Called on the behalf of the permit maker.
				assert_eq!(context.caller, Precompile1.into());
				assert_eq!(address, SmartContractWithOwner::get());
				assert_eq!(is_static, true);

				assert_eq!(input, vec![141, 165, 203, 91]);

				let mut output = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

				output.extend_from_slice(&OwnerOfDapp::get().as_bytes());

				SubcallOutput {
					output: output,
					cost: 1,
					..SubcallOutput::succeed()
				}
			})
			.execute_returns(false);
	});
}

#[test]
fn test_can_claim_reward_should_return_false_if_dapp_not_implement_owner_function() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithoutOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);
		precompiles()
			.prepare_test(
				OwnerOfDapp::get(),
				Precompile1,
				PCall::can_claim_reward {
					claimant: NotOwner::get().into(),
					holder: SmartContractWithoutOwner::get().into(),
				},
			)
			.with_subcall_handle(move |_| {
				panic!("should not be called");
			})
			.execute_returns(false);
	});
}

#[test]
fn test_get_claimable_reward() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(SmartContractWithoutOwner::get(), Token1::get()),
			sp_core::U256::from(0)
		);

		FeeRewardsVault::add_claimable_reward(
			SmartContractWithoutOwner::get(),
			Token1::get(),
			sp_core::U256::from(100),
		)
		.unwrap();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::get_claimable_reward {
					holder: SmartContractWithoutOwner::get().into(),
					token: Token1::get().into(),
				},
			)
			.execute_returns(sp_core::U256::from(100));
	});
}

#[test]
fn test_claim_reward() {
	ExtBuilder::default().build().execute_with(|| {
		let precompile_address: H160 = Precompile1.into();

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_whitelist {
					holder: SmartContractWithoutOwner::get().into(),
					is_whitelisted: true,
				},
			)
			.execute_returns(true);

		assert_eq!(
			FeeRewardsVault::get_claimable_reward(SmartContractWithoutOwner::get(), Token1::get()),
			sp_core::U256::from(0)
		);

		FeeRewardsVault::add_claimable_reward(
			SmartContractWithoutOwner::get(),
			Token1::get(),
			sp_core::U256::from(100),
		)
		.unwrap();
		FeeRewardsVault::add_claimable_reward(
			SmartContractWithoutOwner::get(),
			Token2::get(),
			sp_core::U256::from(100),
		)
		.unwrap();

		precompiles()
			.prepare_test(
				SmartContractWithoutOwner::get(),
				Precompile1,
				PCall::claim_reward {
					holder: SmartContractWithoutOwner::get().into(),
					token: Token1::get().into(),
				},
			)
			.with_subcall_handle(move |subcall| {
				let Subcall {
					address,
					transfer: _,
					input,
					target_gas: _,
					is_static,
					context,
				} = subcall;

				// Called on the behalf of the permit maker.
				assert_eq!(context.caller, precompile_address);
				assert_eq!(address, Token1::get());
				assert_eq!(is_static, false);

				assert_eq!(
					input,
					stbl_tools::eth::generate_calldata(
						&"transfer(address,uint256)",
						&vec![
							SmartContractWithoutOwner::get().into(),
							stbl_tools::misc::u256_to_h256(sp_core::U256::from(100))
						]
					)
				);

				let output = vec![];

				SubcallOutput {
					output: output,
					cost: 1,
					..SubcallOutput::succeed()
				}
			})
			.expect_log(log3(
				precompile_address,
				SELECTOR_REWARD_CLAIMED,
				SmartContractWithoutOwner::get(),
				SmartContractWithoutOwner::get(),
				Vec::from(Token1::get().to_fixed_bytes()),
			))
			.execute_some();

		assert_eq!(
			FeeRewardsVault::get_claimable_reward(SmartContractWithoutOwner::get(), Token1::get()),
			sp_core::U256::from(0)
		);
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(SmartContractWithoutOwner::get(), Token2::get()),
			sp_core::U256::from(100)
		);
	});
}

#[test]
fn test_set_validator_percentage() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::get_validator_percentage {},
			)
			.execute_returns(sp_core::U256::from(0));
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_validator_percentage {
					percentage: sp_core::U256::from(10),
				},
			)
			.execute_some();
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::get_validator_percentage {},
			)
			.execute_returns(sp_core::U256::from(10));
	});
}

#[test]
fn test_set_validator_percentage_fails_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				SmartContractWithoutOwner::get(),
				Precompile1,
				PCall::set_validator_percentage {
					percentage: sp_core::U256::from(10),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"sender is not owner"));
	});
}

#[test]
fn test_set_validator_percentage_fails_if_greater() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::set_validator_percentage {
					percentage: sp_core::U256::from(110),
				},
			)
			.execute_reverts(|x| x.eq_ignore_ascii_case(b"percentage is too high"));
	});
}
