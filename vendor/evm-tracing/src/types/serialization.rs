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

//! Provide serialization functions for various types and formats.

use ethereum_types::{H256, U256};
use serde::{
	ser::{Error, SerializeSeq},
	Serializer,
};
use sp_runtime::traits::UniqueSaturatedInto;

pub fn seq_h256_serialize<S>(data: &Option<Vec<H256>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(vec) = data {
		let mut seq = serializer.serialize_seq(Some(vec.len()))?;
		for hash in vec {
			seq.serialize_element(&format!("{:x}", hash))?;
		}
		seq.end()
	} else {
		let seq = serializer.serialize_seq(Some(0))?;
		seq.end()
	}
}

pub fn bytes_0x_serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
}

pub fn option_bytes_0x_serialize<S>(
	bytes: &Option<Vec<u8>>,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(bytes) = bytes.as_ref() {
		return serializer.serialize_str(&format!("0x{}", hex::encode(&bytes[..])));
	}
	Err(S::Error::custom("String serialize error."))
}

pub fn opcode_serialize<S>(opcode: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(opcode)
		.map_err(|_| S::Error::custom("Opcode serialize error."))?
		.to_uppercase();
	serializer.serialize_str(&d)
}

pub fn string_serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let d = std::str::from_utf8(value)
		.map_err(|_| S::Error::custom("String serialize error."))?
		.to_string();
	serializer.serialize_str(&d)
}

pub fn option_string_serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	if let Some(value) = value.as_ref() {
		let d = std::str::from_utf8(&value[..])
			.map_err(|_| S::Error::custom("String serialize error."))?
			.to_string();
		return serializer.serialize_str(&d);
	}
	Err(S::Error::custom("String serialize error."))
}

pub fn u256_serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_u64(UniqueSaturatedInto::<u64>::unique_saturated_into(*data))
}

pub fn h256_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("{:x}", data))
}

pub fn h256_0x_serialize<S>(data: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&format!("0x{:x}", data))
}
