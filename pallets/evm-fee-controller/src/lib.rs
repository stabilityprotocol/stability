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
	impl<T: Config> OnChargeDNTTransaction for Pallet<T> {
		type LiquidityInfo = U256;

		fn withdraw_fee(
			payer: &H160,
			fee: U256,
		) -> Result<Self::LiquidityInfo, pallet_evm::Error<R>> {
			let fee_token = T::UserFeeTokenController::get_user_fee_token(*payer);

			let author = map_err!(Self::get_author(), |_| {
				pallet_evm::Error::<R>::WithdrawFailed
			});

			let corrected_fee_token_amount =
				Self::convert_fee_to_user_token(fee_token, author, fee);

			T::ERC20Manager::withdraw_amount(&fee_token, payer, corrected_fee_token_amount)
				.map_err(|_| pallet_evm::Error::<R>::WithdrawFailed)
		}

		fn correct_and_deposit_fee(
			who: &H160,
			corrected_fee: U256,
			_base_fee: U256,
			already_withdrawn: Self::LiquidityInfo,
		) -> Self::LiquidityInfo {
			let fee_token = T::UserFeeTokenController::get_user_fee_token(*who);

			let author = Self::get_author().expect("author must be available");

			let corrected_fee_token_amount =
				Self::convert_fee_to_user_token(fee_token, author, corrected_fee);
			let already_withdrawn_token_amount =
				Self::convert_fee_to_user_token(fee_token, author, already_withdrawn);

			let excess_fee_token_amount =
				already_withdrawn_token_amount - corrected_fee_token_amount;

			T::ERC20Manager::deposit_amount(&fee_token, who, excess_fee_token_amount)
				.or(Err(pallet_evm::Error::<T>::Undefined))
				.expect("deposit must succeed since has been withdrawn before");

			T::ERC20Manager::deposit_amount(&fee_token, &author, corrected_fee_token_amount)
				.or(Err(pallet_evm::Error::<T>::Undefined))
				.expect("deposit must succeed");

			U256::from(0)
		}

		fn pay_priority_fee(_priority_fee: Self::LiquidityInfo) {
			// priority fee is paid along the correct_and_deposit_fee
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
