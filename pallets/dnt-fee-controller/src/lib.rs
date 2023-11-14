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
		ERC20DepositFailed,
		FeeVaultOverflow,
		ArithmeticError,
		InvalidPercentage,
	}

	#[pallet::storage]
	#[pallet::getter(fn fee_vault_precompile_address)]
	pub type FeeVaultPrecompileAddressStorage<T: Config> = StorageValue<_, H160, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validator_percentage)]
	pub type ValidatorPercentageStorage<T: Config> = StorageValue<_, U256, OptionQuery>;

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
		pub validator_percentage: U256,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				fee_vault_precompile_address: <H160 as core::str::FromStr>::from_str(
					"0x0000000000000000000000000000000000000807",
				)
				.unwrap(),
				validator_percentage: 50.into(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			FeeVaultPrecompileAddressStorage::<T>::put(self.fee_vault_precompile_address);
			ValidatorPercentageStorage::<T>::put(self.validator_percentage);
		}
	}

	impl<T: Config> OnChargeDecentralizedNativeTokenFee for Pallet<T> {
		type Error = Error<T>;

		fn get_transaction_fee_token(from: H160) -> H160 {
			T::UserFeeTokenController::get_user_fee_token(from)
		}

		fn get_transaction_conversion_rate(
			sender: H160,
			validator: H160,
			token: H160,
		) -> (U256, U256) {
			T::ValidatorTokenController::conversion_rate(sender, validator, token)
		}

		fn get_fee_vault() -> H160 {
			Self::fee_vault_precompile_address().unwrap()
		}

		fn withdraw_fee(
			from: H160,
			token: H160,
			conversion_rate: (U256, U256),
			amount: U256,
		) -> Result<(), Self::Error> {
			let fee_vault = FeeVaultPrecompileAddressStorage::<T>::get().unwrap();
			let mapped_amount = amount
				.checked_mul(conversion_rate.0)
				.map(|v| v.div_mod(conversion_rate.1).0);

			let mapped_amount = match mapped_amount {
				Some(amount) => amount,
				None => return Err(Error::ArithmeticError),
			};

			T::ERC20Manager::withdraw_amount(token, from, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;
			T::ERC20Manager::deposit_amount(token, fee_vault, mapped_amount)
				.map_err(|_| Error::<T>::ERC20DepositFailed)?;

			Ok(())
		}

		fn correct_fee(
			from: H160,
			token: H160,
			conversion_rate: (U256, U256),
			paid_amount: U256,
			actual_amount: U256,
		) -> Result<(), Self::Error> {
			let over_fee = paid_amount.checked_sub(actual_amount);

			let over_fee = match over_fee {
				Some(amount) => amount,
				None => return Err(Error::ArithmeticError),
			};

			let mapped_amount = over_fee
				.checked_mul(conversion_rate.0)
				.map(|v| v.div_mod(conversion_rate.1).0);

			let mapped_amount = match mapped_amount {
				Some(amount) => amount,
				None => return Err(Error::ArithmeticError),
			};

			let fee_vault = FeeVaultPrecompileAddressStorage::<T>::get().unwrap();
			T::ERC20Manager::withdraw_amount(token, fee_vault, mapped_amount)
				.map_err(|_| Error::<T>::ERC20WithdrawFailed)?;
			T::ERC20Manager::deposit_amount(token, from, mapped_amount)
				.map_err(|_| Error::<T>::ERC20DepositFailed)?;

			Ok(())
		}

		fn pay_fees(
			token: H160,
			conversion_rate: (U256, U256),
			actual_amount: U256,
			validator: H160,
			to: Option<H160>,
		) -> Result<(U256, U256), Self::Error> {
			let fee_in_user_token = actual_amount
				.checked_mul(conversion_rate.0)
				.map(|v| v.div_mod(conversion_rate.1).0);

			let fee_in_user_token = match fee_in_user_token {
				Some(amount) => amount,
				None => return Err(Error::ArithmeticError),
			};

			let validator_share = match to {
				None => 100.into(),
				Some(_) => ValidatorPercentageStorage::<T>::get().unwrap(),
			};

			let validator_fee = fee_in_user_token
				.checked_mul(validator_share.into())
				.map(|v| v.div_mod(U256::from(100)).0);

			let validator_fee = match validator_fee {
				Some(a) => a,
				None => return Err(Error::ArithmeticError),
			};

			let dapp_fee = fee_in_user_token - validator_fee;

			pallet_fee_rewards_vault::Pallet::<T>::add_claimable_reward(
				validator,
				token,
				validator_fee,
			)
			.map_err(|_| Error::<T>::FeeVaultOverflow)?;

			if to.is_some() {
				pallet_fee_rewards_vault::Pallet::<T>::add_claimable_reward(
					to.unwrap(),
					token,
					dapp_fee,
				)
				.map_err(|_| Error::<T>::FeeVaultOverflow)?;
			}

			Ok((validator_fee, dapp_fee))
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn set_validator_percentage(percentage: U256) -> Result<(), Error<T>> {
			if percentage > 100.into() {
				return Err(Error::<T>::InvalidPercentage);
			}
			ValidatorPercentageStorage::<T>::put(percentage);
			Ok(())
		}

		pub fn get_validator_percentage() -> U256 {
			ValidatorPercentageStorage::<T>::get().unwrap()
		}
	}
}
