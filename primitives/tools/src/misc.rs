use sp_core::{H256, U256};
use sp_std::vec::Vec;

#[macro_export]
macro_rules! map_err {
	($result:expr, $func:expr) => {
		match $result {
			Err(e) => return Err($func(e)),
			Ok(item) => item,
		}
	};
}

#[macro_export]
macro_rules! some_or_err {
	($result:expr, $func:expr) => {
		match $result {
			None => return Err($func()),
			Some(item) => item,
		}
	};
}

#[macro_export]
macro_rules! none_or_err {
	($result:expr, $func:expr) => {
		match $result {
			None => (),
			Some(e) => return Err($func(e)),
		}
	};
}

#[macro_export]
macro_rules! tertiary_operator {
	($condition:expr, $true:expr, $false:expr) => {
		if $condition {
			$true
		} else {
			$false
		}
	};
}

pub fn u256_to_h256(value: U256) -> H256 {
	let mut tmp = [0u8; 32];
	value.to_big_endian(&mut tmp);
	H256::from(tmp)
}

pub fn bool_to_vec_u8(value: bool) -> Vec<u8> {
	let mut vec = Vec::new();
	vec.push(value as u8);
	vec
}

pub fn u256_to_vec_u8(value: U256) -> Vec<u8> {
	let mut bytes = [0u8; 32];
	let bytes_slice = bytes.as_mut_slice();

	value.to_big_endian(bytes_slice);

	bytes_slice.to_vec()
}
