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

//! Tests for the Validator Set pallet.

#![cfg(test)]

use frame_system::RawOrigin;
use sp_runtime::testing::{TestSignature, UintAuthorityId};

use crate::mock::{new_test_ext, Test};

use super::*;

#[test]
fn fail_validator_set_update_with_invalid_signature() {
	new_test_ext().execute_with(|| {
		let call = crate::Call::<Test>::publish_keys {
			keys: PublishingKeys {
				aura: UintAuthorityId(1),
				grandpa: UintAuthorityId(1),
				block_number: 1u64.into(),
			},
			signature: TestSignature(5, vec![]),
		};

		let err = crate::Pallet::<Test>::validate_unsigned(TransactionSource::Local, &call).err();

		assert_eq!(err, Some(InvalidTransaction::BadProof.into()));
	});
}

#[test]
fn fail_validator_set_update_with_not_approved_validator() {
	new_test_ext().execute_with(|| {
		let keys: PublishingKeys<UintAuthorityId, UintAuthorityId, u64> = PublishingKeys {
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
		let keys: PublishingKeys<UintAuthorityId, UintAuthorityId, u64> = PublishingKeys {
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
