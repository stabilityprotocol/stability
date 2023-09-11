//! Tests for the Validator Set pallet.

#![cfg(test)]

use frame_support::assert_noop;
use frame_system::RawOrigin;
use sp_runtime::testing::{TestSignature, UintAuthorityId};

use crate::mock::{new_test_ext, Test};

use super::*;

#[test]
fn fail_validator_set_update_with_invalid_signature() {
	new_test_ext().execute_with(|| {
		let err = crate::Pallet::<Test>::publish_keys(
			RawOrigin::None.into(),
			PublishingKeys {
				aura: UintAuthorityId(1),
				grandpa: UintAuthorityId(1),
				block_number: 1u64.into(),
			},
			TestSignature(2, vec![]),
		)
		.err();

		assert_eq!(
			err,
			Some(DispatchError::Other("Failed to verify signature"))
		);
	});
}

#[test]
fn fail_validator_set_update_with_not_approved_validator() {
	new_test_ext().execute_with(|| {
		let keys: PublishingKeys<
			UintAuthorityId,
			UintAuthorityId,
			<Test as frame_system::Config>::BlockNumber,
		> = PublishingKeys {
			aura: UintAuthorityId(5),
			grandpa: UintAuthorityId(5),
			block_number: 1u64.into(),
		};

		let signed_message = keys
			.clone()
			.using_encoded(|bytes| TestSignature(5, bytes.to_vec()));

		let err =
			crate::Pallet::<Test>::publish_keys(RawOrigin::None.into(), keys, signed_message).err();

		assert_eq!(err, Some(Error::<Test>::ValidatorNotApproved.into()));
	});
}

#[test]
fn validator_publish_keys() {
	new_test_ext().execute_with(|| {
		let keys: PublishingKeys<
			UintAuthorityId,
			UintAuthorityId,
			<Test as frame_system::Config>::BlockNumber,
		> = PublishingKeys {
			aura: UintAuthorityId(1),
			grandpa: UintAuthorityId(1),
			block_number: 1u64.into(),
		};

		let signed_message = keys
			.clone()
			.using_encoded(|bytes| TestSignature(1, bytes.to_vec()));

		let is_ok = crate::Pallet::<Test>::publish_keys(
			RawOrigin::None.into(),
			keys.clone(),
			signed_message,
		)
		.is_ok();

		assert_eq!(is_ok, true);

		assert!(pallet_session::NextKeys::<Test>::get(keys.aura.0).is_some())
	});
}
