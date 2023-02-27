#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::{H160, U256};
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{
		pallet_prelude::{OptionQuery, ValueQuery},
		storage::types::StorageDoubleMap,
		Blake2_128Concat,
	};
	use pallet_supported_tokens_manager::SupportedTokensManager;
	use sp_core::{Get, H160, U256};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type DefaultFeeToken: sp_core::Get<H160>;
		type SupportedTokensManager: SupportedTokensManager;
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::storage]
	#[pallet::getter(fn supported_fee_tokens)]
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

	#[derive(codec::Encode, sp_core::Decode, frame_support::pallet_prelude::TypeInfo)]
	pub struct ConversionRate(U256, U256);

	impl Default for ConversionRate {
		fn default() -> Self {
			ConversionRate(U256::from(1), U256::from(1))
		}
	}

	impl Into<(U256, U256)> for ConversionRate {
		fn into(self) -> (U256, U256) {
			(self.0, self.1)
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn conversion_rate_fee_tokens)]
	pub type ValidatorConversionRateToken<T: Config> = StorageDoubleMap<
		_,
		// Owner
		Blake2_128Concat,
		H160,
		Blake2_128Concat,
		H160,
		// Nonce
		ConversionRate,
		ValueQuery,
	>;

	pub enum ValidatorFeeTokenError {
		NotMutableDefaultTokenConversionRate,
		NotSupportedToken,
	}

	impl<T: Config> ValidatorSupportedTokens for Pallet<T> {
		fn validator_supported_tokens(validator: H160) -> Vec<H160> {
			T::SupportedTokensManager::get_supported_tokens().into_iter()
				.filter(|token| Self::validator_supports_fee_token(validator, *token))
				.collect()
		}
	}

	impl<T: Config> ValidatorFeeTokenController for Pallet<T> {
		type Error = ValidatorFeeTokenError;

		fn validator_supports_fee_token(validator: H160, token: H160) -> bool {
			ValidatorSupportFeeToken::<T>::get(validator, token)
				.unwrap_or(token.eq(&T::DefaultFeeToken::get()))
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

		fn conversion_rate(validator: H160, token: H160) -> (U256, U256) {
			ValidatorConversionRateToken::<T>::get(validator, token).into()
		}

		fn update_conversion_rate(
			account: H160,
			token: H160,
			conversion_rate: (U256, U256),
		) -> Result<(), Self::Error> {
			if token.eq(&T::DefaultFeeToken::get()) {
				return Err(ValidatorFeeTokenError::NotMutableDefaultTokenConversionRate);
			}
			ValidatorConversionRateToken::<T>::set(
				account,
				token,
				ConversionRate(conversion_rate.0, conversion_rate.1),
			);

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
	fn conversion_rate(validator: H160, token: H160) -> (U256, U256);
	fn update_conversion_rate(
		validator: H160,
		token: H160,
		conversion_rate: (U256, U256),
	) -> Result<(), Self::Error>;
}

pub trait ValidatorSupportedTokens {
	fn validator_supported_tokens(validator: H160) -> Vec<H160>;
}
