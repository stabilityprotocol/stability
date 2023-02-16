// Copyright 2019-2022 Stability Solutions.
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
use frame_support::parameter_types;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	storage::types::{StorageDoubleMap, ValueQuery},
	traits::StorageInstance,
	Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};

use precompile_utils::prelude::*;
use sp_core::{Get, H256};
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

/// Associates pallet Instance to a prefix used for the Approves storage.
/// This trait is implemented for () and the 16 substrate Instance.
pub trait FeePrefixes {
	/// Prefix used for the Approves storage.
	type ValidatorTokenAcceptancePrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($instance:ident, $name:literal) => {
		// Using `paste!` we generate a dedicated module to avoid collisions
		// between each instance `Approves` struct.
		paste::paste! {
			mod [<_impl_prefix_ $instance:snake>] {
				use super::*;

				pub struct ValidatorTokenAcceptance;

				impl StorageInstance for ValidatorTokenAcceptance {
					const STORAGE_PREFIX: &'static str = "ValidatorTokenAcceptance";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}


				impl FeePrefixes for $instance {
					type ValidatorTokenAcceptancePrefix = ValidatorTokenAcceptance;
				}
			}
		}
	};
}

// Since the macro expect a `ident` to be used with `paste!` we cannot provide `()` directly.
type Instance0 = ();

impl_prefix!(Instance0, "Erc20Instance0Balances");
impl_prefix!(Instance1, "Erc20Instance1Balances");
impl_prefix!(Instance2, "Erc20Instance2Balances");
impl_prefix!(Instance3, "Erc20Instance3Balances");
impl_prefix!(Instance4, "Erc20Instance4Balances");
impl_prefix!(Instance5, "Erc20Instance5Balances");
impl_prefix!(Instance6, "Erc20Instance6Balances");
impl_prefix!(Instance7, "Erc20Instance7Balances");
impl_prefix!(Instance8, "Erc20Instance8Balances");
impl_prefix!(Instance9, "Erc20Instance9Balances");
impl_prefix!(Instance10, "Erc20Instance10Balances");
impl_prefix!(Instance11, "Erc20Instance11Balances");
impl_prefix!(Instance12, "Erc20Instance12Balances");
impl_prefix!(Instance13, "Erc20Instance13Balances");
impl_prefix!(Instance14, "Erc20Instance14Balances");
impl_prefix!(Instance15, "Erc20Instance15Balances");
impl_prefix!(Instance16, "Erc20Instance16Balances");

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
	pub DefaultAcceptance:bool = false;
	pub DefaultConversionRate:(U256, U256) = (U256::from(1), U256::from(1));
}

/// Storage double map type used to store acceptable tokens for validators.
pub type ValidatorTokenAcceptanceStorage<Instance> = StorageDoubleMap<
	<Instance as FeePrefixes>::ValidatorTokenAcceptancePrefix,
	// Validator
	Blake2_128Concat,
	H160,
	// Token
	Blake2_128Concat,
	H160,
	// Is accepted?
	bool,
	ValueQuery,
	DefaultAcceptance,
	// ^ false
>;

/// Storage double map type used to store conversion rates.
pub type ValidatorTokenConversionRateStorage<Instance> = StorageDoubleMap<
	<Instance as FeePrefixes>::ValidatorTokenAcceptancePrefix,
	// Owner
	Blake2_128Concat,
	H160,
	Blake2_128Concat,
	H160,
	// Nonce
	(U256, U256),
	ValueQuery,
	DefaultConversionRate,
>;

/// Precompile for manage tokens in which validators would accept fees to be paid on
/// Also the validators will set their conversion rate for those tokens
pub struct ValidatorFeeManagerPrecompile<
	Runtime,
	DefaultToken: Get<H160> + 'static,
	Instance: 'static = (),
>(PhantomData<(Runtime, DefaultToken, Instance)>);

#[precompile_utils::precompile]
impl<Runtime, DefaultFeeToken, Instance>
	ValidatorFeeManagerPrecompile<Runtime, DefaultFeeToken, Instance>
where
	DefaultFeeToken: Get<H160> + 'static,
	Instance: FeePrefixes + 'static,
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
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

		ValidatorTokenAcceptanceStorage::<Instance>::set::<H160, H160>(
			msg_sender,
			token_address.into(),
			acceptance_value,
		);

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
	fn validator_supports_token(
		handle: &mut impl PrecompileHandle,
		validator: Address,
		token_address: Address,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let acceptance = ValidatorTokenAcceptanceStorage::<Instance>::get::<H160, H160>(
			validator.into(),
			token_address.into(),
		);

		Ok(acceptance)
	}

	#[precompile::public("setTokenConversionRate(address,uint256,uint256)")]
	#[precompile::view]
	fn set_token_conversion_rate(
		handle: &mut impl PrecompileHandle,
		token_address: Address,
		numerator: U256,
		denominator: U256,
	) -> EvmResult {
		if DefaultFeeToken::get() == token_address.into() {
			return Err(revert(
				"VALIDATOR_FEE_MANAGER: default token has a fixed conversion rate",
			));
		};

		let msg_sender = handle.context().caller;
		handle.record_log_costs_manual(3, 64)?;

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		ValidatorTokenConversionRateStorage::<Instance>::set::<H160, H160>(
			msg_sender,
			token_address.into(),
			(numerator, denominator),
		);

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

		let conversion_rate = ValidatorTokenConversionRateStorage::<Instance>::get::<H160, H160>(
			validator.into(),
			token_address.into(),
		);

		Ok(conversion_rate)
	}

	#[precompile::public("safeTokenConversionRate(address,address)")]
	#[precompile::view]
	fn safe_token_conversion_rate(
		handle: &mut impl PrecompileHandle,
		validator: Address,
		token_address: Address,
	) -> EvmResult<(U256, U256)> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let is_token_accepted = ValidatorTokenAcceptanceStorage::<Instance>::get::<H160, H160>(
			validator.into(),
			token_address.into(),
		);

		if is_token_accepted {
			Self::token_conversion_rate(handle, validator, token_address)
		} else {
			Err(revert(
				"VALIDATOR_FEE_MANAGER: token not supported by target validator",
			))
		}
	}
}
