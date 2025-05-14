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

use crate::listeners::call_list::Listener;
use crate::types::serialization::*;
use crate::types::{
	single::{Call, TransactionTrace},
	CallResult, CallType, CreateResult,
};
use ethereum_types::{H160, U256};
use parity_scale_codec::{Decode, Encode};
use serde::Serialize;

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn format(listener: Listener) -> Option<TransactionTrace> {
		if let Some(entry) = listener.entries.last() {
			return Some(TransactionTrace::CallList(
				entry
					.into_iter()
					.map(|(_, value)| Call::Blockscout(value.clone()))
					.collect(),
			));
		}
		None
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum BlockscoutCallInner {
	Call {
		#[serde(rename(serialize = "callType"))]
		/// Type of call.
		call_type: CallType,
		to: H160,
		#[serde(serialize_with = "bytes_0x_serialize")]
		input: Vec<u8>,
		/// "output" or "error" field
		#[serde(flatten)]
		res: CallResult,
	},
	Create {
		#[serde(serialize_with = "bytes_0x_serialize")]
		init: Vec<u8>,
		#[serde(flatten)]
		res: CreateResult,
	},
	SelfDestruct {
		#[serde(skip)]
		balance: U256,
		to: H160,
	},
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockscoutCall {
	pub from: H160,
	/// Indices of parent calls.
	pub trace_address: Vec<u32>,
	/// Number of children calls.
	/// Not needed for Blockscout, but needed for `crate::block`
	/// types that are build from this type.
	#[serde(skip)]
	pub subtraces: u32,
	/// Sends funds to the (payable) function
	pub value: U256,
	/// Remaining gas in the runtime.
	pub gas: U256,
	/// Gas used by this context.
	pub gas_used: U256,
	#[serde(flatten)]
	pub inner: BlockscoutCallInner,
	pub logs: Vec<crate::types::single::Log>,
}
