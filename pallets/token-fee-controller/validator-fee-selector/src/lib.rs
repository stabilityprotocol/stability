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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::{H160, U256};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		pallet_prelude::*,
		storage::types::{StorageDoubleMap, StorageMap, StorageValue},
		Blake2_128Concat,
	};

	use pallet_evm::Runner;
	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::{H160, H256, U256};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type SupportedTokensManager: SupportedTokensManager;
		type SimulatorRunner: pallet_evm::Runner<Self>;
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::storage]
	#[pallet::getter(fn validator_support_token)]
	pub type ValidatorSupportFeeToken<T: Config> = StorageDoubleMap<
		_,
		// Owner
		Blake2_128Concat,
		H160,
		Blake2_128Concat,
		H160,
		// Nonce
		bool,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn default_controller)]
	pub type DefaultController<T: Config> = StorageValue<_, H160, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn conversion_rate_fee_tokens)]
	pub type ValidatorConversionRateController<T: Config> = StorageMap<
		_,
		// Owner
		Blake2_128Concat,
		H160,
		// CR controller
		H160,
		OptionQuery,
	>;

	pub enum ValidatorFeeTokenError {
		ControllerIsEOA,
		NotSupportedToken,
	}

	impl<T: Config> ValidatorSupportedTokens for Pallet<T> {
		fn validator_supported_tokens(validator: H160) -> Vec<H160> {
			T::SupportedTokensManager::get_supported_tokens()
				.into_iter()
				.filter(|token| Self::validator_supports_fee_token(validator, *token))
				.collect()
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		pub initial_default_conversion_rate_controller: H160,
		#[serde(skip)]
		pub _config: PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_default_conversion_rate_controller: H160::zero(),
				_config: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			DefaultController::<T>::put(self.initial_default_conversion_rate_controller);
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn set_default_controller(controller: H160) {
			DefaultController::<T>::put(controller);
		}
	}

	impl<T: Config> ValidatorFeeTokenController for Pallet<T> {
		type Error = ValidatorFeeTokenError;

		fn validator_supports_fee_token(validator: H160, token: H160) -> bool {
			ValidatorSupportFeeToken::<T>::get(validator, token)
				.unwrap_or(token == T::SupportedTokensManager::get_default_token())
				&& T::SupportedTokensManager::is_supported_token(token)
		}

		fn update_fee_token_acceptance(
			validator: H160,
			token: H160,
			support: bool,
		) -> Result<(), Self::Error> {
			if !T::SupportedTokensManager::is_supported_token(token) {
				Err(ValidatorFeeTokenError::NotSupportedToken)
			} else {
				ValidatorSupportFeeToken::<T>::insert(validator, token, support);

				Ok(())
			}
		}

		fn conversion_rate_controller(validator: H160) -> H160 {
			ValidatorConversionRateController::<T>::get(validator)
				.unwrap_or(DefaultController::<T>::get().unwrap())
		}

		fn conversion_rate(sender: H160, validator: H160, token: H160) -> (U256, U256) {
			let conversion_rate_controller = Self::conversion_rate_controller(validator);

			let args: sp_std::vec::Vec<H256> =
				sp_std::vec![sender.into(), validator.into(), token.into()];

			T::SimulatorRunner::call(
				H160::from_low_u64_be(0),
				conversion_rate_controller,
				stbl_tools::eth::generate_calldata(
					"getConversionRate(address,address,address)",
					&args,
				),
				0.into(),
				3_000_000,
				None,
				None,
				None,
				Default::default(),
				false,
				false,
				None,
				None,
				&pallet_evm::EvmConfig::shanghai(),
			)
			.map(|execution_info| {
				let value_len = execution_info.value.len();
				if value_len >= 64 {
					(
						U256::from_big_endian(&execution_info.value[0..32]),
						U256::from_big_endian(&execution_info.value[32..64]),
					)
				} else if value_len >= 32 {
					(
						U256::from_big_endian(&execution_info.value[0..32]),
						U256::from(1),
					)
				} else {
					(U256::from(1), U256::from(1))
				}
			})
			.unwrap_or((U256::from(1), U256::from(1)))
		}

		fn update_conversion_rate_controller(
			validator: H160,
			conversion_rate_controller: H160,
		) -> Result<(), Self::Error> {
			if pallet_evm::AccountCodes::<T>::get(conversion_rate_controller).is_empty() {
				Err(ValidatorFeeTokenError::ControllerIsEOA)
			} else {
				ValidatorConversionRateController::<T>::insert(
					validator,
					conversion_rate_controller,
				);
				Ok(())
			}
		}

		fn update_default_controller(controller: H160) -> Result<(), Self::Error> {
			Self::set_default_controller(controller);
			Ok(())
		}
	}
}

pub trait ValidatorFeeTokenController {
	type Error;

	fn validator_supports_fee_token(validator: H160, token: H160) -> bool;

	fn update_fee_token_acceptance(
		validator: H160,
		token: H160,
		support: bool,
	) -> Result<(), Self::Error>;

	fn conversion_rate_controller(validator: H160) -> H160;

	fn conversion_rate(sender: H160, validator: H160, token: H160) -> (U256, U256);

	fn update_conversion_rate_controller(
		validator: H160,
		conversion_rate_controller: H160,
	) -> Result<(), Self::Error>;

	fn update_default_controller(controller: H160) -> Result<(), Self::Error>;
}

pub trait ValidatorSupportedTokens {
	fn validator_supported_tokens(validator: H160) -> Vec<H160>;
}
