use sp_core::H256;
use sp_runtime::traits::Keccak256;
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

	hash.as_bytes().iter().for_each(|byte| {
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
