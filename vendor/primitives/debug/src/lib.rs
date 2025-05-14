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

#![cfg_attr(not(feature = "std"), no_std)]

use ethereum::{TransactionV0 as LegacyTransaction, TransactionV2 as Transaction};
use ethereum_types::{H160, H256, U256};
use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	// Api version is virtually 5.
	//
	// We realized that even using runtime overrides, using the ApiExt interface reads the api
	// versions from the state runtime, meaning we cannot just reset the versioning as we see fit.
	//
	// In order to be able to use ApiExt as part of the RPC handler logic we need to be always
	// above the version that exists on chain for this Api, even if this Api is only meant
	// to be used overridden.
	#[api_version(6)]
	pub trait DebugRuntimeApi {
		#[changed_in(5)]
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
		) -> Result<(), sp_runtime::DispatchError>;

		#[changed_in(4)]
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &LegacyTransaction,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		#[changed_in(5)]
		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_block(
			extrinsics: Vec<Block::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &Block::Header,
		) -> Result<(), sp_runtime::DispatchError>;

		fn trace_call(
			header: &Block::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TracerInput {
	None,
	Blockscout,
	CallTracer,
}

/// DebugRuntimeApi V2 result. Trace response is stored in client and runtime api call response is
/// empty.
#[derive(Debug)]
pub enum Response {
	Single,
	Block,
}
