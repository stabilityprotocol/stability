#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_support::traits::FindAuthor;
	use pallet_erc20_manager::ERC20Manager;
	use pallet_user_fee_selector::UserFeeTokenController;
	use pallet_validator_fee_selector::ValidatorFeeTokenController;
	use runner::OnChargeDecentralizedNativeTokenFee;
	use sp_core::{H160, U256};
	use stbl_tools::some_or_err;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		NoAuthorFound,
		CachedTokenNotAvailable,
		ERC20WithdrawFailed,
	}

	#[pallet::storage]
	#[pallet::getter(fn fee_vault_precompile_address)]
	pub type FeeVaultPrecompileAddressStorage<T: Config> = StorageValue<_, H160, OptionQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type UserFeeTokenController: UserFeeTokenController;
		type ValidatorTokenController: ValidatorFeeTokenController;
		type ERC20Manager: ERC20Manager;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub fee_vault_precompile_address: H160,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				fee_vault_precompile_address: H160::from_low_u64_be(0),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			FeeVaultPrecompileAddressStorage::<T>::put(self.fee_vault_precompile_address);
		}
	}

	impl<T: Config> OnChargeDecentralizedNativeTokenFee for Pallet<T> {
		type Error = Error<T>;

		fn get_transaction_fee_token(from: H160) -> H160 {
			T::UserFeeTokenController::get_user_fee_token(from)
		}

		fn get_transaction_conversion_rate(validator: H160, token: H160) -> (U256, U256) {
			T::ValidatorTokenController::conversion_rate(validator, token)
		}

		fn withdraw_fee(
			from: H160,
			token: H160,
			conversion_rate: (U256, U256),
			amount: U256,
		) -> Result<(), Self::Error> {
			let fee_vault = FeeVaultPrecompileAddressStorage::<T>::get().unwrap();
			let mapped_amount = amount
				.saturating_mul(conversion_rate.0)
				.div_mod(conversion_rate.1)
				.0;
			T::ERC20Manager::withdraw_amount(token, from, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;
			T::ERC20Manager::deposit_amount(token, fee_vault, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;

			Ok(())
		}

		fn correct_fee(
			_from: H160,
			_token: H160,
			_conversion_rate: (U256, U256),
			_paid_amount: U256,
			_actual_amount: U256,
		) -> Result<(), Self::Error> {
			todo!()
		}

		fn pay_fees(_actual_amount: U256, _validator: H160, _to: H160) -> Result<(), Self::Error> {
			todo!()
		}
	}

	impl<T: Config> Pallet<T> {
		fn convert_fee_to_user_token(token: H160, author: H160, fee: U256) -> U256 {
			let (numerator, denominator) =
				T::ValidatorTokenController::conversion_rate(author, token);

			fee.saturating_mul(numerator).div_mod(denominator).0
		}

		fn get_author() -> Result<H160, Error<T>> {
			let digest = <frame_system::Pallet<T>>::digest();
			let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());

			let author = some_or_err!(T::FindAuthor::find_author(pre_runtime_digests), || {
				Error::<T>::NoAuthorFound
			});

			Ok(author)
		}
	}
}
