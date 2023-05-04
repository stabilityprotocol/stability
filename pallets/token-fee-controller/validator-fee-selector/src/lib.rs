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

	use core::str::FromStr;

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
		NotMutableDefaultTokenConversionRate,
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
	#[cfg(feature = "std")]
	pub struct GenesisConfig {
		pub initial_default_conversion_rate_controller: H160,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				initial_default_conversion_rate_controller: H160::from_str(
					"0x444212d6E4827893A70d19921E383130281Cda4a",
				)
				.expect("invalid address"),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			DefaultController::<T>::put(self.initial_default_conversion_rate_controller);
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

			let output = T::SimulatorRunner::call(
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
				&pallet_evm::EvmConfig::london(),
			)
			.map_err(|_| ())
			.unwrap()
			.value;

			(
				U256::from_big_endian(output[0..32].as_ref()),
				U256::from_big_endian(output[32..64].as_ref()),
			)
		}

		fn update_conversion_rate_controller(
			validator: H160,
			conversion_rate_controller: H160,
		) -> Result<(), Self::Error> {
			ValidatorConversionRateController::<T>::insert(validator, conversion_rate_controller);
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
}

pub trait ValidatorSupportedTokens {
	fn validator_supported_tokens(validator: H160) -> Vec<H160>;
}
