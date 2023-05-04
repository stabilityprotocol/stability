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
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED: [u8; 32] =
	keccak256!("ValidatorTokenAcceptanceChanged(address,address,bool)");

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_VALIDATOR_TOKEN_RATE_CHANGED: [u8; 32] =
	keccak256!("ValidatorTokenRateChanged(address,address,uint256,uint256)");

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
	pub DefaultAcceptance:bool = false;
	pub DefaultConversionRate:(U256, U256) = (U256::from(1), U256::from(1));
}

/// Precompile for manage tokens in which validators would accept fees to be paid on
/// Also the validators will set their conversion rate for those tokens
pub struct ValidatorFeeManagerPrecompile<
	Runtime,
	ValidatorFeeTokenController,
	Instance: 'static = (),
>(PhantomData<(Runtime, ValidatorFeeTokenController, Instance)>);

#[precompile_utils::precompile]
impl<Runtime, ValidatorFeeTokenController, Instance>
	ValidatorFeeManagerPrecompile<Runtime, ValidatorFeeTokenController, Instance>
where
	ValidatorFeeTokenController: pallet_validator_fee_selector::ValidatorFeeTokenController,
	Instance: 'static,
	Runtime: pallet_evm::Config + pallet_timestamp::Config + pallet_validator_set::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
{
	#[precompile::public("setTokenAcceptance(address,bool)")]
	fn set_token_acceptance(
		handle: &mut impl PrecompileHandle,
		token_address: Address,
		acceptance_value: bool,
	) -> EvmResult {
		let msg_sender = handle.context().caller;
		handle.record_log_costs_manual(3, 32)?;

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let validators = pallet_validator_set::Pallet::<Runtime>::approved_validators();

		if !validators.contains(&msg_sender.into()) {
			return Err(revert(
				"ValidatorFeeTokenController: sender is not an approved validator",
			));
		}

		match ValidatorFeeTokenController::update_fee_token_acceptance(
			msg_sender,
			token_address.into(),
			acceptance_value,
		) {
			Ok(_) => {}
			Err(_) => {
				return Err(revert("ValidatorFeeTokenController: token not supported"));
			}
		};

		log3(
			handle.context().address,
			SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED,
			msg_sender,
			Into::<H160>::into(token_address),
			EvmDataWriter::new().write(acceptance_value).build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("validatorSupportsToken(address,address)")]
	#[precompile::view]
	fn validator_supports_token(
		handle: &mut impl PrecompileHandle,
		validator: Address,
		token_address: Address,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ValidatorFeeTokenController::validator_supports_fee_token(
			validator.into(),
			token_address.into(),
		))
	}

	#[precompile::public("setTokenConversionRate(address,uint256,uint256)")]
	fn set_token_conversion_rate(
		handle: &mut impl PrecompileHandle,
		token_address: Address,
		numerator: U256,
		denominator: U256,
	) -> EvmResult {
		let msg_sender = handle.context().caller;
		handle.record_log_costs_manual(3, 64)?;

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let validators = pallet_validator_set::Pallet::<Runtime>::approved_validators();

		if !validators.contains(&msg_sender.into()) {
			return Err(revert(
				"ValidatorFeeTokenController: sender is not an approved validator",
			));
		}

		match ValidatorFeeTokenController::update_conversion_rate(
			msg_sender,
			token_address.into(),
			(numerator, denominator),
		) {
			Err(_) => {
				return Err(revert(
					b"ValidatorFeeTokenController: default token conversion rate cannot be updated",
				))
			}
			_ => {}
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_VALIDATOR_TOKEN_RATE_CHANGED,
			msg_sender,
			Into::<H160>::into(token_address),
			EvmDataWriter::new()
				.write(numerator)
				.write(denominator)
				.build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("tokenConversionRate(address,address)")]
	#[precompile::view]
	fn token_conversion_rate(
		handle: &mut impl PrecompileHandle,
		validator: Address,
		token_address: Address,
	) -> EvmResult<(U256, U256)> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ValidatorFeeTokenController::conversion_rate(
			validator.into(),
			token_address.into(),
		))
	}
}
