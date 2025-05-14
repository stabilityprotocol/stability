// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

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

pub fn truncate_u256_to_u64(n: U256) -> u64 {
	if n > U256::from(u64::MAX) {
		u64::MAX
	} else {
		n.as_u64()
	}
}

pub fn kecckak256(bytes: &[u8]) -> H256 {
	use sha3::{Digest, Keccak256};
	let mut hasher = Keccak256::new();

	hasher.update(bytes);
	let result = hasher.finalize();

	H256::from_slice(result.as_slice())
}

pub fn u64_to_buffer_in_ascii(u: u64) -> Vec<u8> {
	let mut buffer = Vec::new();
	let mut u = u;

	if u == 0 {
		buffer.push(48);
		return buffer;
	}

	while u > 0 {
		let digit = u % 10;
		buffer.push((digit + 48) as u8);
		u /= 10;
	}
	buffer.reverse();
	buffer
}
