#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use pallet_erc20_manager::ERC20Manager;
	use pallet_evm::OnChargeEVMTransaction;
	use pallet_user_fee_selector::UserFeeTokenController;
	use pallet_validator_fee_selector::ValidatorFeeTokenController;
	use sp_core::{H160, U256};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		InternalError,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type UserFeeTokenController: UserFeeTokenController;
		type ValidatorTokenController: ValidatorFeeTokenController;
		type ERC20Manager: ERC20Manager;
	}

	impl<R: pallet_evm::Config, T: Config> OnChargeEVMTransaction<R> for Pallet<T> {
		type LiquidityInfo = U256;

		fn withdraw_fee(
			payer: &H160,
			fee: U256,
		) -> Result<Self::LiquidityInfo, pallet_evm::Error<R>> {
			Ok(U256::default())
		}

		fn correct_and_deposit_fee(
			who: &H160,
			corrected_fee: U256,
			base_fee: U256,
			already_withdrawn: Self::LiquidityInfo,
		) -> Self::LiquidityInfo {
			U256::default()
		}

		fn pay_priority_fee(liquidity: Self::LiquidityInfo) {}
	}
}
