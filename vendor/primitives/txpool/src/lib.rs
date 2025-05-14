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
// These clippy lints are disabled because the macro-generated code triggers them.
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::too_many_arguments)]

pub use ethereum::{TransactionV0 as LegacyTransaction, TransactionV2 as Transaction};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::scale_info::TypeInfo;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct TxPoolResponseLegacy {
	pub ready: Vec<LegacyTransaction>,
	pub future: Vec<LegacyTransaction>,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct TxPoolResponse {
	pub ready: Vec<Transaction>,
	pub future: Vec<Transaction>,
}

sp_api::decl_runtime_apis! {
	#[api_version(2)]
	pub trait TxPoolRuntimeApi {
		#[changed_in(2)]
		fn extrinsic_filter(
			xt_ready: Vec<<Block as BlockT>::Extrinsic>,
			xt_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> TxPoolResponseLegacy;
		fn extrinsic_filter(
			xt_ready: Vec<<Block as BlockT>::Extrinsic>,
			xt_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> TxPoolResponse;
	}
}
