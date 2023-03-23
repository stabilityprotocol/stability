#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::traits::FindAuthor;
	use pallet_erc20_manager::ERC20Manager;
	use pallet_evm::OnChargeEVMTransaction;
	use pallet_user_fee_selector::UserFeeTokenController;
	use pallet_validator_fee_selector::ValidatorFeeTokenController;
	use runner::OnChargeDecentralizedNativeTokenFee;
	use sp_core::{H160, U256};
	use stbl_tools::{map_err, some_or_err};

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

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type UserFeeTokenController: UserFeeTokenController;
		type ValidatorTokenController: ValidatorFeeTokenController;
		type ERC20Manager: ERC20Manager;
	}

	// todo: create other trait for token fee controller
	impl<T: Config> OnChargeDecentralizedNativeTokenFee for Pallet<T> {
		type Error = ();

		fn get_transaction_fee_token(from: &H160) -> H160 {
			todo!()
		}

		fn get_transaction_conversion_rate(validator: &H160) -> (U256, U256) {
			todo!()
		}

		fn withdraw_fee(
			from: &H160,
			token: &H160,
			conversion_rate: (U256, U256),
			amount: U256,
		) -> Result<(), Self::Error> {
			todo!()
		}

		fn correct_fee(
			from: &H160,
			token: &H160,
			conversion_rate: (U256, U256),
			paid_amount: U256,
			actual_amount: U256,
		) -> Result<(), Self::Error> {
			todo!()
		}

		fn pay_fees(actual_amount: U256, validator: &H160, to: &H160) -> Result<(), Self::Error> {
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
