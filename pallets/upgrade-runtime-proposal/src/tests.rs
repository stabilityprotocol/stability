use crate::mock::UpgradeRuntimeProposal;

use super::*;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use mock::{
	new_test_ext, Test, assert_runtime_updated_digest
};
use sp_core::keccak_256;
use frame_support::traits::Hooks;

#[test]
fn test_setup_works() {
	new_test_ext().execute_with(|| {
		assert!(UpgradeRuntimeProposal::get_proposed_code().is_none());
	});
}

#[test]
fn test_propose_code_works() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		let proposed_code = substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec();
		assert!(UpgradeRuntimeProposal::get_proposed_code().is_none());
		assert_ok!(UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			proposed_code.clone()
		));
		let saved_code = UpgradeRuntimeProposal::get_proposed_code();
		assert!(saved_code.is_some());
		assert_eq!(
			keccak_256(proposed_code.as_slice()),
			keccak_256(saved_code.unwrap().as_slice())
		);
	});
}

#[test]
fn test_propose_code_fails_if_bad_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::propose_code(
				frame_system::RawOrigin::Signed(1).into(),
				vec![1, 2, 3]
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_propose_code_fails_if_there_are_already_a_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_ok!(UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
		));
		assert_noop!(
			UpgradeRuntimeProposal::propose_code(
				frame_system::RawOrigin::Root.into(),
				substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
			),
			Error::<Test>::ProposalInProgress
		);
	});
}

#[test]
fn test_propose_code_fails_invalid_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::propose_code(
				frame_system::RawOrigin::Root.into(),
				vec![1, 2, 3]
			),
			Error::<Test>::InvalidCode
		);
	});
}

#[test]
fn test_set_block_application() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_ok!(UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
		));
		assert_ok!(UpgradeRuntimeProposal::set_block_application(
			frame_system::RawOrigin::Root.into(),
			1
		));
		assert_eq!(UpgradeRuntimeProposal::get_application_block_number().unwrap(), 1);
	});
}

#[test]
fn test_set_block_application_fails_if_bad_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::set_block_application(
				frame_system::RawOrigin::Signed(1).into(),
				1
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_set_block_application_fails_block_is_older() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_ok!(UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
		));
		assert_noop!(
			UpgradeRuntimeProposal::set_block_application(frame_system::RawOrigin::Root.into(), 0),
			Error::<Test>::BlockNumberMustBeGreaterThanCurrentBlockNumber
		);
	});
}

#[test]
fn test_set_block_application_fails_if_not_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::set_block_application(frame_system::RawOrigin::Root.into(), 1),
			Error::<Test>::NoProposedCode
		);
	});
}

#[test]
fn test_reject_proposed_code() {
	let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
	let mut ext = new_test_ext();
	ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

	ext.execute_with(|| {
		assert_ok!(UpgradeRuntimeProposal::propose_code(
			frame_system::RawOrigin::Root.into(),
			substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
		));
		assert_ok!(UpgradeRuntimeProposal::set_block_application(
			frame_system::RawOrigin::Root.into(),
			1
		));
		assert_ok!(UpgradeRuntimeProposal::reject_proposed_code(
			frame_system::RawOrigin::Root.into()
		));
		assert!(UpgradeRuntimeProposal::get_proposed_code().is_none());
		assert!(UpgradeRuntimeProposal::get_application_block_number().is_none());
	});
}

#[test]
fn test_fails_reject_proposed_code_if_bad_origin() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::reject_proposed_code(frame_system::RawOrigin::Signed(1).into()),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_fails_reject_proposed_code_if_no_proposed_code() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			UpgradeRuntimeProposal::reject_proposed_code(frame_system::RawOrigin::Root.into()),
			Error::<Test>::NoProposedCode
		);
	});
}

#[test]
fn test_scheduled_update_runtime() {
    let executor = substrate_test_runtime_client::new_native_or_wasm_executor();
    let mut ext = new_test_ext();
    ext.register_extension(sp_core::traits::ReadRuntimeVersionExt::new(executor));

    ext.execute_with(|| {
        assert_ok!(UpgradeRuntimeProposal::propose_code(
            frame_system::RawOrigin::Root.into(),
            substrate_test_runtime_client::runtime::wasm_binary_unwrap().to_vec()
        ));
        assert_ok!(UpgradeRuntimeProposal::set_block_application(
            frame_system::RawOrigin::Root.into(),
            1
        ));
        assert_eq!(UpgradeRuntimeProposal::get_application_block_number().unwrap(), 1);
        
        UpgradeRuntimeProposal::on_initialize(1);

        assert_runtime_updated_digest(1);
        assert!(UpgradeRuntimeProposal::get_proposed_code().is_none());
        assert!(UpgradeRuntimeProposal::get_application_block_number().is_none());
    });
}