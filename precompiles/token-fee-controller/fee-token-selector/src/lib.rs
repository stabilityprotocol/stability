// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

//! Precompile to interact with pallet_balances instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use core::str::FromStr;

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::parameter_types;

use precompile_utils::prelude::*;
use sp_core::H256;
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_FEE_CHANGED: [u8; 32] = keccak256!("FeeTokenChanged(address,address)");

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct FeeTokenPrecompile<Runtime, UserFeeTokenController, Instance: 'static = ()>(
	PhantomData<(Runtime, UserFeeTokenController, Instance)>,
);

#[precompile_utils::precompile]
impl<Runtime, UserFeeTokenController, Instance>
	FeeTokenPrecompile<Runtime, UserFeeTokenController, Instance>
where
	UserFeeTokenController: pallet_user_fee_selector::UserFeeTokenController,
	Instance: 'static,
	Runtime: pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	#[precompile::public("setFeeToken(address)")]
	fn set_fee_token(handle: &mut impl PrecompileHandle, token_address: Address) -> EvmResult {
		let msg_sender = handle.context().caller;

		if token_address == H160::zero().into() {
			return Err(revert("UserFeeTokenController: zero address is invalid"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		match UserFeeTokenController::set_user_fee_token(msg_sender.into(), token_address.into()) {
			Err(_) => {
				return Err(revert("UserFeeTokenController: token not supported"));
			}
			_ => {}
		};

		handle.record_log_costs_manual(2, 32)?;
		log2(
			handle.context().address,
			SELECTOR_LOG_FEE_CHANGED,
			msg_sender,
			solidity::encode_event_data(token_address)
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("getFeeToken(address)")]
	#[precompile::view]
	fn get_fee_token(handle: &mut impl PrecompileHandle, address: Address) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(UserFeeTokenController::get_user_fee_token(address.into()).into())
	}
}
