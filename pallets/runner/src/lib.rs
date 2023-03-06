#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use pallet_evm::{runner::Runner as RunnerT, CreateInfo, RunnerError};
use sp_std::vec::Vec;

// This pallet wraps the pallet_evm::runner::stack::Runner trait to provide a custom runner for the EVM pallet.
// Basically, it forces the EVM pallet to use the same runner as the Stability pallet and
// forces the value always to zero.

mod mock;

mod tests;
#[frame_support::pallet]
pub mod pallet {
	use sp_core::{H160, H256, U256};

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {}

	impl<T: Config> RunnerT<T> for Pallet<T>
	where
		pallet_evm::BalanceOf<T>: TryFrom<U256> + Into<U256>,
	{
		type Error = pallet_evm::Error<T>;

		fn call(
			source: H160,
			target: H160,
			input: Vec<u8>,
			_value: U256,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
			is_transactional: bool,
			validate: bool,
			config: &evm::Config,
		) -> Result<pallet_evm::CallInfo, RunnerError<Self::Error>> {
			pallet_evm::runner::stack::Runner::<T>::call(
				source,
				target,
				input,
				U256::zero(),
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
				is_transactional,
				validate,
				config,
			)
		}

		fn validate(
			source: H160,
			target: Option<H160>,
			input: Vec<u8>,
			_value: U256,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
			is_transactional: bool,
			config: &evm::Config,
		) -> Result<(), RunnerError<Self::Error>> {
			pallet_evm::runner::stack::Runner::<T>::validate(
				source,
				target,
				input,
				U256::zero(),
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
				is_transactional,
				config,
			)
		}

		fn create(
			source: H160,
			init: Vec<u8>,
			_value: U256,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
			is_transactional: bool,
			validate: bool,
			config: &evm::Config,
		) -> Result<CreateInfo, RunnerError<Self::Error>> {
			pallet_evm::runner::stack::Runner::<T>::create(
				source,
				init,
				U256::zero(),
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
				is_transactional,
				validate,
				config,
			)
		}

		fn create2(
			source: H160,
			init: Vec<u8>,
			salt: H256,
			_value: U256,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
			is_transactional: bool,
			validate: bool,
			config: &evm::Config,
		) -> Result<CreateInfo, RunnerError<Self::Error>> {
			pallet_evm::runner::stack::Runner::<T>::create2(
				source,
				init,
				salt,
				U256::zero(),
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list,
				is_transactional,
				validate,
				config,
			)
		}
	}
}
