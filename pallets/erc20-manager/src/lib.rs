#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{U256, H160};

pub trait ERC20Manager {
	fn forceTransferFrom(token: &H160, payer: &H160, author: &H160, fee: U256) -> Result<U256, ()>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::traits::FindAuthor;
	use pallet_evm::{EvmConfig, OnChargeEVMTransaction, Runner};
	use sp_core::{Get, H160, H256, U256};
	use sp_std::vec::Vec;
	use stbl_tools::{eth, map_err};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {}

	impl<T: Config> ERC20Manager for Pallet<T> {
		fn forceTransferFrom(token: &H160, payer: &H160, author: &H160, fee: U256) -> Result<U256, ()> {
            Ok(U256::from(0))
        }
	}
}
