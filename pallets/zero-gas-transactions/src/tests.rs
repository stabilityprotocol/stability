// Copyright © 2022 STABILITY SOLUTIONS, INC. ("STABILITY")
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the "Software").

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use crate::mock::{
	eth_address_from_private_key, legacy_erc20_creation_transaction, new_test_ext,
	sign_consent_message, ChainId, MockBlockAuthor, Runtime, System,
};
use frame_support::pallet_prelude::{TransactionSource, ValidateUnsigned};
use frame_system::RawOrigin;
use pallet_ethereum::Transaction;
use sp_core::{ecdsa, hexdisplay::AsBytesRef, Pair, H256};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

// ============================================================================
// Existing test (preserved)
// ============================================================================

#[test]
fn fail_to_execute_transaction_with_high_nonce() {
	new_test_ext().execute_with(|| {
		// Sign the transaction
		let private_key = H256::random();
		let trx1 = legacy_erc20_creation_transaction(100.into(), &private_key);

		let chain_id = ChainId::get();
		let current_block = System::block_number();

		let message: Vec<u8> = b"I consent to validate zero gas transactions in block "
			.iter()
			.chain(current_block.to_string().as_bytes().iter())
			.chain(b" on chain ")
			.chain(chain_id.to_string().as_bytes().iter())
			.cloned()
			.collect();

		let pair = ecdsa::Pair::from_seed_slice(private_key.as_bytes()).unwrap();
		let signature = pair.sign(message.as_bytes_ref());

		let error = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx1.clone()),
			signature.0.to_vec(),
		)
		.unwrap_err();

		assert!(matches!(
			error.error,
			sp_runtime::DispatchError::Other("Invalid transaction data")
		));
	})
}

// ============================================================================
// Group 1: validate_unsigned with TransactionSource-based branching
// ============================================================================

/// Test that `TransactionSource::Local` (pool submission) skips the validator
/// consent signature check. This is the core enabler for the mempool enqueue path:
/// the background fetcher submits ZGTs with TransactionSource::Local, and pool
/// validation should pass even without a valid consent signature for the current block.
#[test]
fn validate_unsigned_local_source_skips_consent_check() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(1);
		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);

		// Use a garbage consent signature — Local source should skip the check
		let garbage_signature = vec![0u8; 65];

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: garbage_signature,
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::Local, &call);

		assert!(
			result.is_ok(),
			"TransactionSource::Local should skip consent check, but got: {:?}",
			result.err()
		);
	})
}

/// Test that `TransactionSource::External` (pool submission from gossip)
/// also skips the consent check, same as Local.
#[test]
fn validate_unsigned_external_source_skips_consent_check() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(2);
		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);

		let garbage_signature = vec![0u8; 65];

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: garbage_signature,
		};

		let result =
			crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::External, &call);

		assert!(
			result.is_ok(),
			"TransactionSource::External should skip consent check, but got: {:?}",
			result.err()
		);
	})
}

/// Test that `TransactionSource::InBlock` (block execution) REQUIRES a valid
/// consent signature. With a wrong signature, it should fail with BadProof.
#[test]
fn validate_unsigned_inblock_source_requires_valid_consent() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(3);
		let validator_key = H256::from_low_u64_be(100);
		let validator_address = eth_address_from_private_key(&validator_key);

		// Set the block author so find_author() returns our validator
		MockBlockAuthor::put(validator_address);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);

		// Use a garbage consent signature — InBlock source should reject it
		let garbage_signature = vec![0u8; 65];

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: garbage_signature,
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::InBlock, &call);

		assert!(
			result.is_err(),
			"TransactionSource::InBlock should require valid consent signature"
		);
		assert_eq!(
			result.unwrap_err(),
			TransactionValidityError::Invalid(InvalidTransaction::BadProof)
		);
	})
}

/// Test that `TransactionSource::InBlock` succeeds when given a properly
/// signed consent message matching the current block and validator.
#[test]
fn validate_unsigned_inblock_source_succeeds_with_valid_consent() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(4);
		let validator_key = H256::from_low_u64_be(200);
		let validator_address = eth_address_from_private_key(&validator_key);

		// Set the block author and block number
		MockBlockAuthor::put(validator_address);
		let block_number: u64 = 5;
		frame_system::Pallet::<Runtime>::set_block_number(block_number);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, block_number, ChainId::get());

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: consent_sig,
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::InBlock, &call);

		assert!(
			result.is_ok(),
			"InBlock with valid consent should succeed, but got: {:?}",
			result.err()
		);
	})
}

/// Test that even with `TransactionSource::Local`, a transaction with an
/// invalid Ethereum signature (unrecoverable) is still rejected.
/// The pool skip only applies to the *validator consent* check, NOT the
/// Ethereum transaction signature check.
#[test]
fn validate_unsigned_local_source_still_checks_tx_signature() {
	new_test_ext().execute_with(|| {
		// Create a legacy transaction with an irrecoverable signature.
		// We set both r and s to zero which makes secp256k1 recovery impossible.
		let trx = ethereum::LegacyTransaction {
			nonce: sp_core::U256::zero(),
			gas_price: sp_core::U256::zero(),
			gas_limit: sp_core::U256::from(0x100000),
			action: ethereum::TransactionAction::Call(sp_core::H160::zero()),
			value: sp_core::U256::zero(),
			input: vec![],
			signature: ethereum::TransactionSignature::new(
				// v = 27 (unprotected legacy), r and s = 1 (not a valid point on the curve
				// for this message, so recovery will fail or produce a garbage address
				// with no valid chain_id check)
				38, // chain_id * 2 + 35 = 20180428 * 2 + 35 = 40360891, but we need a simpler approach
				H256::from_low_u64_be(1), // r = 1 (invalid for secp256k1 recovery for most messages)
				H256::from_low_u64_be(1), // s = 1
			)
			.unwrap(),
		};

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: vec![0u8; 65],
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::Local, &call);

		// The transaction should fail at ensure_transaction_signature (BadProof)
		// or at pool_ensure_transaction_unicity (Call) due to invalid chain_id.
		// Either way, it must NOT succeed.
		assert!(
			result.is_err(),
			"Transaction with invalid/unrecoverable signature should be rejected even for Local source"
		);
	})
}

// ============================================================================
// Group 2: ±10 Block Window in ensure_zero_gas_transaction
// ============================================================================

/// Test that the consent signature is verified successfully when it matches
/// the exact current block number (fast path).
#[test]
fn consent_signature_exact_block_succeeds() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(10);
		let validator_key = H256::from_low_u64_be(300);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);
		let block_number: u64 = 42;
		frame_system::Pallet::<Runtime>::set_block_number(block_number);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, block_number, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		// The transaction should pass consent check. It may fail at EVM execution
		// (the mock doesn't have a real contract at Sponsor address) but it should
		// NOT fail with "Invalid zero gas transaction signature".
		if let Err(ref err) = result {
			assert!(
				!matches!(
					err.error,
					sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
				),
				"Consent signature for exact block should be valid"
			);
		}
	})
}

/// Test that the consent signature is verified successfully when signed for
/// a block within the ±10 window (the pool-based enqueue scenario).
/// The fetcher signs for best_block+1, but execution may happen several blocks later.
#[test]
fn consent_signature_within_window_succeeds() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(11);
		let validator_key = H256::from_low_u64_be(400);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);

		// The fetcher signed for block 50, but we're now executing at block 58
		// (within the ±10 window)
		let signing_block: u64 = 50;
		let executing_block: u64 = 58;
		frame_system::Pallet::<Runtime>::set_block_number(executing_block);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, signing_block, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		if let Err(ref err) = result {
			assert!(
				!matches!(
					err.error,
					sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
				),
				"Consent signature within +8 block window should be valid"
			);
		}
	})
}

/// Test the ±10 window in the negative direction: signed for block 50,
/// executing at block 42 (within window).
#[test]
fn consent_signature_within_negative_window_succeeds() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(12);
		let validator_key = H256::from_low_u64_be(500);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);

		let signing_block: u64 = 50;
		let executing_block: u64 = 42;
		frame_system::Pallet::<Runtime>::set_block_number(executing_block);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, signing_block, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		if let Err(ref err) = result {
			assert!(
				!matches!(
					err.error,
					sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
				),
				"Consent signature within -8 block window should be valid"
			);
		}
	})
}

/// Test that a consent signature signed for a block OUTSIDE the ±10 window
/// is rejected. Signed for block 20, but executing at block 35 (15 blocks away).
#[test]
fn consent_signature_outside_window_fails() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(13);
		let validator_key = H256::from_low_u64_be(600);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);

		let signing_block: u64 = 20;
		let executing_block: u64 = 35; // 15 blocks away, outside ±10 window
		frame_system::Pallet::<Runtime>::set_block_number(executing_block);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, signing_block, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		assert!(
			result.is_err(),
			"Consent signature outside ±10 window should be rejected"
		);

		let err = result.unwrap_err();
		assert!(
			matches!(
				err.error,
				sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
			),
			"Expected 'Invalid zero gas transaction signature' error, got: {:?}",
			err.error
		);
	})
}

/// Test the boundary: consent signed for block N, executing at exactly N+10
/// (the edge of the window). Should succeed.
#[test]
fn consent_signature_at_window_boundary_succeeds() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(14);
		let validator_key = H256::from_low_u64_be(700);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);

		let signing_block: u64 = 50;
		let executing_block: u64 = 60; // exactly +10 blocks = boundary
		frame_system::Pallet::<Runtime>::set_block_number(executing_block);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, signing_block, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		if let Err(ref err) = result {
			assert!(
				!matches!(
					err.error,
					sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
				),
				"Consent signature at exactly +10 boundary should be valid"
			);
		}
	})
}

/// Test the boundary: consent signed for block N, executing at N+11
/// (just past the window). Should fail.
#[test]
fn consent_signature_just_past_window_boundary_fails() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(15);
		let validator_key = H256::from_low_u64_be(800);
		let validator_address = eth_address_from_private_key(&validator_key);

		MockBlockAuthor::put(validator_address);

		let signing_block: u64 = 50;
		let executing_block: u64 = 61; // +11 blocks, just past the window
		frame_system::Pallet::<Runtime>::set_block_number(executing_block);

		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);
		let consent_sig = sign_consent_message(&validator_key, signing_block, ChainId::get());

		let result = crate::Pallet::<Runtime>::send_zero_gas_transaction(
			RawOrigin::None.into(),
			Transaction::Legacy(trx),
			consent_sig,
		);

		assert!(
			result.is_err(),
			"Consent signature at +11 (past window) should be rejected"
		);

		let err = result.unwrap_err();
		assert!(
			matches!(
				err.error,
				sp_runtime::DispatchError::Other("Invalid zero gas transaction signature")
			),
			"Expected 'Invalid zero gas transaction signature' error, got: {:?}",
			err.error
		);
	})
}

// ============================================================================
// Group 3: ValidTransaction properties (longevity, provides, priority)
// ============================================================================

/// Test that validate_unsigned returns a ValidTransaction with longevity=20.
/// This ensures ZGTs expire from the pool after ~20 blocks.
#[test]
fn validate_unsigned_returns_correct_longevity() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(20);
		let trx = legacy_erc20_creation_transaction(0.into(), &sender_key);

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: vec![0u8; 65], // Local source skips consent check
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::Local, &call);

		assert!(result.is_ok());
		let valid_tx = result.unwrap();
		assert_eq!(
			valid_tx.longevity, 20,
			"ZGT pool longevity should be 20 blocks, got: {}",
			valid_tx.longevity
		);
	})
}

/// Test that validate_unsigned returns the correct provides tag (sender, nonce)
/// and maximum priority.
#[test]
fn validate_unsigned_returns_correct_provides_and_priority() {
	new_test_ext().execute_with(|| {
		let sender_key = H256::from_low_u64_be(21);
		let sender_address = eth_address_from_private_key(&sender_key);
		let nonce = 0u64;
		let trx = legacy_erc20_creation_transaction(nonce.into(), &sender_key);

		let call = crate::Call::<Runtime>::send_zero_gas_transaction {
			transaction: Transaction::Legacy(trx),
			validator_signature: vec![0u8; 65],
		};

		let result = crate::Pallet::<Runtime>::validate_unsigned(TransactionSource::Local, &call);

		assert!(result.is_ok());
		let valid_tx = result.unwrap();

		// Priority should be u64::MAX for ZGTs
		assert_eq!(
			valid_tx.priority,
			u64::MAX,
			"ZGT priority should be u64::MAX"
		);

		// Provides should contain the (sender_address, nonce) tuple
		assert!(
			!valid_tx.provides.is_empty(),
			"ValidTransaction should have provides tags"
		);

		// The provides tag is encoded as (H160, U256), verify it's present
		use parity_scale_codec::Encode;
		let expected_tag = (sender_address, sp_core::U256::from(nonce)).encode();
		assert!(
			valid_tx.provides.contains(&expected_tag),
			"Provides should contain (sender_address, nonce). Expected: {:?}, Got: {:?}",
			expected_tag,
			valid_tx.provides
		);
	})
}
