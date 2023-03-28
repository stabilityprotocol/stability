#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use pallet_erc20_manager::ERC20Manager;
	use pallet_user_fee_selector::UserFeeTokenController;
	use pallet_validator_fee_selector::ValidatorFeeTokenController;
	use runner::OnChargeDecentralizedNativeTokenFee;
	use sp_core::{H160, U256};

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
	pub trait Config:
		frame_system::Config + pallet_evm::Config + pallet_fee_rewards_vault::Config
	{
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
			from: H160,
			token: H160,
			conversion_rate: (U256, U256),
			paid_amount: U256,
			actual_amount: U256,
		) -> Result<(), Self::Error> {
			let over_fee = paid_amount.saturating_sub(actual_amount);
			let mapped_amount = over_fee
				.saturating_mul(conversion_rate.0)
				.div_mod(conversion_rate.1)
				.0;
			let fee_vault = FeeVaultPrecompileAddressStorage::<T>::get().unwrap();
			T::ERC20Manager::withdraw_amount(token, fee_vault, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;
			T::ERC20Manager::deposit_amount(token, from, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;

			Ok(())
		}

		fn pay_fees(
			token: H160,
			conversion_rate: (U256, U256),
			actual_amount: U256,
			validator: H160,
			to: H160,
		) -> Result<(), Self::Error> {
			let mapped_amount = actual_amount
				.saturating_mul(conversion_rate.0)
				.div_mod(conversion_rate.1)
				.0;
			let amount_validator = mapped_amount.saturating_sub(mapped_amount / 2);

			pallet_fee_rewards_vault::Pallet::<T>::add_claimable_reward(
				validator,
				token,
				amount_validator,
			);
			pallet_fee_rewards_vault::Pallet::<T>::add_claimable_reward(
				to,
				token,
				mapped_amount - amount_validator,
			);

			Ok(())
		}
	}
}
