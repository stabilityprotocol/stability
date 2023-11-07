#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

use sp_core::{H160, U256};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	// double map
	#[pallet::storage]
	#[pallet::getter(fn claimable_reward)]
	pub(super) type ClaimableReward<T: Config> =
		StorageDoubleMap<_, Twox64Concat, H160, Twox64Concat, H160, U256, ValueQuery>;

	// map
	#[pallet::storage]
	#[pallet::getter(fn whitelist)]
	pub(super) type Whitelist<T: Config> = StorageMap<_, Twox64Concat, H160, bool, ValueQuery>;

	impl<T: Config> Pallet<T> {
		pub fn is_whitelisted(address: H160) -> bool {
			Self::whitelist(address)
		}

		pub fn set_whitelist(address: H160, is_whitelisted: bool) {
			Whitelist::<T>::insert(address, is_whitelisted);
		}

		pub fn get_claimable_reward(address: H160, token: H160) -> U256 {
			Self::claimable_reward(address, token)
		}

		pub fn add_claimable_reward(
			address: H160,
			token: H160,
			amount: U256,
		) -> Result<(), &'static str> {
			if amount.is_zero() {
				return Ok(());
			}
			let current_amount = Self::claimable_reward(address, token);

			let new_amount = current_amount
				.checked_add(amount)
				.ok_or("Overflow adding a new claimable reward")?;

			ClaimableReward::<T>::insert(address, token, new_amount);
			Ok(())
		}

		pub fn sub_claimable_reward(
			address: H160,
			token: H160,
			amount: U256,
		) -> Result<(), &'static str> {
			if amount.is_zero() {
				return Ok(());
			}
			let current_amount = Self::claimable_reward(address, token);

			let new_amount = current_amount
				.checked_sub(amount)
				.ok_or("Insufficient balance")?;

			ClaimableReward::<T>::insert(address, token, new_amount);
			Ok(())
		}
	}
}
