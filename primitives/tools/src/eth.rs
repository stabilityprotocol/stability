use fp_ethereum::{Transaction, TransactionData};
use sha3::Digest;
use sp_core::U256;
use sp_core::{Encode, H160, H256};
use sp_runtime::traits::Keccak256;
use sp_std::prelude::*;
use sp_std::vec;
use sp_std::vec::Vec;

use crate::some_or_err;

pub fn generate_calldata(signature: &str, args: &Vec<H256>) -> Vec<u8> {
	let hash = <Keccak256 as sp_core::Hasher>::hash(signature.as_bytes());

	let mut u8_array: Vec<u8> = Vec::default();

	hash.as_bytes().split_at(4).0.iter().for_each(|byte| {
		u8_array.push(*byte);
	});

	args.iter().for_each(|arg_value| {
		arg_value.as_bytes().iter().for_each(|byte| {
			u8_array.push(*byte);
		});
	});

	u8_array
}

pub fn generate_calldata_from_encoded_args(signature: &str, args: &Vec<u8>) -> Vec<u8> {
	let hash = <Keccak256 as sp_core::Hasher>::hash(signature.as_bytes());

	let mut u8_array: Vec<u8> = Vec::default();

	hash.as_bytes().split_at(4).0.iter().for_each(|byte| {
		u8_array.push(*byte);
	});

	u8_array.extend(args);

	u8_array
}

pub fn read_bytes32_from_output_into(
	output_bytes: &[u8],
	number_of_chunks: usize,
	result: &mut Vec<[u8; 32]>,
) -> Result<(), ()> {
	let chunks = &mut output_bytes.chunks(32);

	for _ in 0..number_of_chunks {
		let chunk = some_or_err!(chunks.next(), || ());

		result.push(*H256::from_slice(chunk).as_fixed_bytes());
	}

	Ok(())
}

pub fn recover_signer(transaction: &Transaction) -> Option<H160> {
	let mut sig = [0u8; 65];
	let mut msg = [0u8; 32];
	match transaction {
		Transaction::Legacy(t) => {
			sig[0..32].copy_from_slice(&t.signature.r()[..]);
			sig[32..64].copy_from_slice(&t.signature.s()[..]);
			sig[64] = t.signature.standard_v();
			msg.copy_from_slice(&ethereum::LegacyTransactionMessage::from(t.clone()).hash()[..]);
		}
		Transaction::EIP2930(t) => {
			sig[0..32].copy_from_slice(&t.r[..]);
			sig[32..64].copy_from_slice(&t.s[..]);
			sig[64] = t.odd_y_parity as u8;
			msg.copy_from_slice(&ethereum::EIP2930TransactionMessage::from(t.clone()).hash()[..]);
		}
		Transaction::EIP1559(t) => {
			sig[0..32].copy_from_slice(&t.r[..]);
			sig[32..64].copy_from_slice(&t.s[..]);
			sig[64] = t.odd_y_parity as u8;
			msg.copy_from_slice(&ethereum::EIP1559TransactionMessage::from(t.clone()).hash()[..]);
		}
	}
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
	Some(H160::from(H256::from(sp_io::hashing::keccak_256(&pubkey))))
}

pub fn code_implements_function(code: &[u8], function: &str) -> bool {
	let hash = <Keccak256 as sp_core::Hasher>::hash(function.as_bytes());
	let selector = &hash.as_bytes()[0..4];

	let mut encoded_byte_code_function = vec![99]; // PUSH4 OP_CODE
	encoded_byte_code_function.extend_from_slice(selector);

	let encoded_byte_code_function_slice = encoded_byte_code_function.as_slice();

	code.windows(encoded_byte_code_function_slice.len())
		.any(|window| window == encoded_byte_code_function_slice)
}

pub fn args_to_bytes(args: sp_std::vec::Vec<H256>) -> Vec<u8> {
	let mut u8_array: Vec<u8> = Vec::default();

	args.iter().for_each(|arg_value| {
		arg_value.as_bytes().iter().for_each(|byte| {
			u8_array.push(*byte);
		});
	});

	u8_array
}

pub fn get_storage_address_for_mapping(address: H160, var_slot: H256) -> H256 {
	let u256_address = H256::from(address);
	let address_bytes = u256_address.as_bytes();
	let balance_slot_bytes = var_slot.as_bytes();

	let input = &[&address_bytes[..], &balance_slot_bytes[..]].concat();

	sha3::Keccak256::new()
		.chain_update(input)
		.finalize()
		.using_encoded(|x| H256::from_slice(&x[1..]))
}

pub fn transaction_gas_price(
	base_fee: U256,
	transaction: &Transaction,
	is_transactional: bool,
) -> Result<U256, ()> {
	let data: TransactionData = TransactionData::from(transaction);

	let max_fee_per_gas = crate::custom_fee::compute_fee_details(
		base_fee,
		data.max_fee_per_gas.or(data.gas_price),
		data.max_priority_fee_per_gas,
	)
	.actual_fee;

	if max_fee_per_gas < base_fee && is_transactional {
		return Err(());
	} else {
		Ok(max_fee_per_gas)
	}
}

pub fn build_eip191_message_hash(message: Vec<u8>) -> H256 {
	let result = b"\x19Ethereum Signed Message:\n"
		.iter()
		.chain(crate::misc::u64_to_buffer_in_ascii(message.len().try_into().unwrap()).iter())
		.chain(message.iter())
		.cloned()
		.collect::<Vec<u8>>();

	crate::misc::kecckak256(&result)
}
