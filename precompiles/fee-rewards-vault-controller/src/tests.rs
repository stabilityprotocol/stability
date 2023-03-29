use crate::{mock::*, *};

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
			.execute_returns_encoded(Into::<H256>::into(DefaultOwner::get()));
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
					new_owner: precompile_utils::data::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns_encoded(Into::<H256>::into(new_owner));

		precompiles()
			.prepare_test(
				DefaultOwner::get(),
				Precompile1,
				PCall::transfer_ownership {
					new_owner: precompile_utils::data::Address(other_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(DefaultOwner::get(), Precompile1, PCall::pending_owner {})
			.execute_returns_encoded(Into::<H256>::into(other_owner));
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
					new_owner: precompile_utils::data::Address(new_owner),
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
					new_owner: precompile_utils::data::Address(new_owner),
				},
			)
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::claim_ownership {})
			.expect_log(log1(
				Precompile1,
				SELECTOR_LOG_NEW_OWNER,
				EvmDataWriter::new()
					.write(Into::<H256>::into(new_owner))
					.build(),
			))
			.execute_some();

		precompiles()
			.prepare_test(new_owner, Precompile1, PCall::owner {})
			.execute_returns_encoded(Into::<H256>::into(new_owner));
	});
}

parameter_types! {
	pub Dapp1:H160 = H160::from_str("0x3000000000000000000000000000000000000000").expect("invalid address");
	pub Dapp2:H160 = H160::from_str("0x4000000000000000000000000000000000000000").expect("invalid address");
}

#[test]
fn  test_can_claim_reward_returns_true_if_dapp_and_claimant_are_equal() {
    ExtBuilder::default().build().execute_with(|| {
        precompiles()
            .prepare_test(DefaultOwner::get(), Precompile1, PCall::set_whitelist { dapp: SmartContratWithoutOwner::get().into(), is_whitelisted: true } )
            .execute_returns_encoded(true);
        precompiles()
            .prepare_test(SmartContratWithoutOwner::get(), Precompile1, PCall::can_claim_reward {
                claimant: SmartContratWithoutOwner::get().into(),
                dapp: SmartContratWithoutOwner::get().into(),
            })
            .execute_returns_encoded(true);
    });
}


#[test]
fn  test_can_claim_reward_should_return_false_if_not_whitelisted() {
    ExtBuilder::default().build().execute_with(|| {
        precompiles()
            .prepare_test(Dapp1::get(), Precompile1, PCall::can_claim_reward {
                claimant: Dapp1::get().into(),
                dapp: Dapp1::get().into(),
            })
            .execute_returns_encoded(false);
    });
}

#[test]
fn test_can_claim_reward_should_return_true_if_claimant_are_the_owner_of_the_dapp() {
    ExtBuilder::default().build().execute_with(|| {
        precompiles()
            .prepare_test(DefaultOwner::get(), Precompile1, PCall::set_whitelist { dapp: SmartContractWithOwner::get().into(), is_whitelisted: true } )
            .execute_returns_encoded(true);
        precompiles()
            .prepare_test(OwnerOfDapp::get(), Precompile1, PCall::can_claim_reward {
                claimant: OwnerOfDapp::get().into(),
                dapp: SmartContractWithOwner::get().into(),
            }).with_subcall_handle(move |subcall| {
				let Subcall {
					address,
					transfer,
					input,
					target_gas,
					is_static,
					context,
				} = subcall;

				// Called on the behalf of the permit maker.
				assert_eq!(context.caller, Precompile1.into());
				assert_eq!(address, SmartContractWithOwner::get());
				assert_eq!(is_static, true);
				
				assert_eq!(input, vec![141, 165, 203, 91]);
		
				let mut output =  vec![0,0,0,0,0,0,0,0,0,0,0,0];

				output.extend_from_slice(&OwnerOfDapp::get().as_bytes());

				SubcallOutput {
					reason: ExitReason::Succeed(ExitSucceed::Returned),
					output: output,
					cost: 1,
					logs: vec![],
				}
			}).execute_returns_encoded(true);
    });
}


#[test]
fn test_can_claim_reward_should_return_false_if_claimant_are_not_the_owner_of_the_dapp() {
    ExtBuilder::default().build().execute_with(|| {
        precompiles()
            .prepare_test(DefaultOwner::get(), Precompile1, PCall::set_whitelist { dapp: SmartContractWithOwner::get().into(), is_whitelisted: true } )
            .execute_returns_encoded(true);
        precompiles()
            .prepare_test(OwnerOfDapp::get(), Precompile1, PCall::can_claim_reward {
                claimant: NotOwner::get().into(),
                dapp: SmartContractWithOwner::get().into(),
            }).with_subcall_handle(move |subcall| {
				let Subcall {
					address,
					transfer,
					input,
					target_gas,
					is_static,
					context,
				} = subcall;

				// Called on the behalf of the permit maker.
				assert_eq!(context.caller, Precompile1.into());
				assert_eq!(address, SmartContractWithOwner::get());
				assert_eq!(is_static, true);
				
				assert_eq!(input, vec![141, 165, 203, 91]);
		
				let mut output =  vec![0,0,0,0,0,0,0,0,0,0,0,0];

				output.extend_from_slice(&OwnerOfDapp::get().as_bytes());

				SubcallOutput {
					reason: ExitReason::Succeed(ExitSucceed::Returned),
					output: output,
					cost: 1,
					logs: vec![],
				}
			}).execute_returns_encoded(false);
    });
}


#[test]
fn test_can_claim_reward_should_return_false_if_dapp_not_implement_owner_function() {
	ExtBuilder::default().build().execute_with(|| {
	precompiles()
		.prepare_test(DefaultOwner::get(), Precompile1, PCall::set_whitelist { dapp: SmartContratWithoutOwner::get().into(), is_whitelisted: true } )
		.execute_returns_encoded(true);
    precompiles()
            .prepare_test(OwnerOfDapp::get(), Precompile1, PCall::can_claim_reward {
                claimant: NotOwner::get().into(),
                dapp: SmartContratWithoutOwner::get().into(),
            }).with_subcall_handle(move |_| {
				
				panic!("should not be called");

			}).execute_returns_encoded(false);
	});
}

#[test]
fn test_can_claim_reward_should_return_false_if_dapp_is_not_smart_contract() {
	ExtBuilder::default().build().execute_with(|| {
	precompiles()
		.prepare_test(DefaultOwner::get(), Precompile1, PCall::set_whitelist { dapp: NotOwner::get().into(), is_whitelisted: true } )
		.execute_returns_encoded(true);
    precompiles()
            .prepare_test(OwnerOfDapp::get(), Precompile1, PCall::can_claim_reward {
                claimant: NotOwner::get().into(),
                dapp: NotOwner::get().into(),
            }).with_subcall_handle(move |_| {
				
				panic!("should not be called");

			}).execute_returns_encoded(false);
	});
}
