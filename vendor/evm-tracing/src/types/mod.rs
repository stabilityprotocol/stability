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

//! Runtime API allowing to debug/trace Ethereum

extern crate alloc;

use ethereum_types::{H160, H256};
use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;

pub mod block;
pub mod serialization;
pub mod single;

use serde::Serialize;
use serialization::*;

pub const MANUAL_BLOCK_INITIALIZATION_RUNTIME_VERSION: u32 = 159;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CallResult {
	Output(#[serde(serialize_with = "bytes_0x_serialize")] Vec<u8>),
	// field "error"
	Error(#[serde(serialize_with = "string_serialize")] Vec<u8>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum CreateResult {
	Error {
		#[serde(serialize_with = "string_serialize")]
		error: Vec<u8>,
	},
	Success {
		#[serde(rename = "createdContractAddressHash")]
		created_contract_address_hash: H160,
		#[serde(serialize_with = "bytes_0x_serialize", rename = "createdContractCode")]
		created_contract_code: Vec<u8>,
	},
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CallType {
	Call,
	CallCode,
	DelegateCall,
	StaticCall,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CreateType {
	Create,
}

#[derive(Debug)]
pub enum ContextType {
	Call(CallType),
	Create,
}

impl ContextType {
	pub fn from(opcode: Vec<u8>) -> Option<Self> {
		let opcode = match alloc::str::from_utf8(&opcode[..]) {
			Ok(op) => op.to_uppercase(),
			_ => return None,
		};
		match &opcode[..] {
			"CREATE" | "CREATE2" => Some(ContextType::Create),
			"CALL" => Some(ContextType::Call(CallType::Call)),
			"CALLCODE" => Some(ContextType::Call(CallType::CallCode)),
			"DELEGATECALL" => Some(ContextType::Call(CallType::DelegateCall)),
			"STATICCALL" => Some(ContextType::Call(CallType::StaticCall)),
			_ => None,
		}
	}
}

pub fn convert_memory(memory: Vec<u8>) -> Vec<H256> {
	let size = 32;
	memory
		.chunks(size)
		.map(|c| {
			let mut msg = [0u8; 32];
			let chunk = c.len();
			if chunk < size {
				let left = size - chunk;
				let remainder = vec![0; left];
				msg[0..left].copy_from_slice(&remainder[..]);
				msg[left..size].copy_from_slice(c);
			} else {
				msg[0..size].copy_from_slice(c)
			}
			H256::from_slice(&msg[..])
		})
		.collect()
}
