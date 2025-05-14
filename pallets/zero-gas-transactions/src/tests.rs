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

use crate::mock::{legacy_erc20_creation_transaction, new_test_ext, ChainId, Runtime, System};
use frame_system::RawOrigin;
use pallet_ethereum::Transaction;
use sp_core::{ecdsa, hexdisplay::AsBytesRef, Pair, H256};

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
