#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use fp_ethereum::TransactionData;
	use fp_evm::FeeCalculator;
	use frame_support::dispatch::GetDispatchInfo;
	use frame_support::pallet_prelude::{StorageMap, *};
	use frame_support::sp_runtime::traits::UniqueSaturatedInto;
	use frame_system::pallet_prelude::*;
	use pallet_evm::GasWeightMapping;
	use pallet_user_fee_selector::UserFeeTokenController;
	use sp_core::U256;
	use sp_std::vec;

	pub use fp_rpc::TransactionStatus;

	use pallet_ethereum::Transaction;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type SponsorNonce<T: Config> = StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_ethereum::Config {
		type RuntimeCall: Parameter + GetDispatchInfo;
		type UserFeeTokenController: UserFeeTokenController;
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		Result<pallet_ethereum::RawOrigin, <T as frame_system::Config>::RuntimeOrigin>:
			From<<T as frame_system::Config>::RuntimeOrigin>,
		<T as frame_system::Config>::RuntimeOrigin: From<pallet_ethereum::RawOrigin>,
	{
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::send_zero_gas_transaction { transaction } => {
					let from =
						Self::ensure_transaction_signature(transaction.clone()).map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::BadProof)
						})?;

					let transaction_data: TransactionData = transaction.into();

					Self::pool_ensure_transaction_unicity(&from, transaction)
						.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

					return sp_runtime::transaction_validity::ValidTransactionBuilder::default()
						.and_provides((from, transaction_data.nonce))
						.priority(u64::MAX)
						.build();
				}
				_ => Err(TransactionValidityError::Unknown(
					UnknownTransaction::Custom(0),
				)),
			}?;

			return sp_runtime::transaction_validity::ValidTransactionBuilder::default()
				.and_provides((H160::zero(), U256::zero()))
				.priority(u64::MAX)
				.build();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		Result<pallet_ethereum::RawOrigin, <T as frame_system::Config>::RuntimeOrigin>:
			From<<T as frame_system::Config>::RuntimeOrigin>,
		<T as frame_system::Config>::RuntimeOrigin: From<pallet_ethereum::RawOrigin>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight({
			let without_base_extrinsic_weight = true;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				let transaction_data: TransactionData = transaction.into();
				transaction_data.gas_limit.unique_saturated_into()
			}, without_base_extrinsic_weight)
		})]
		pub fn send_zero_gas_transaction(
			_origin: OriginFor<T>,
			transaction: Transaction,
		) -> DispatchResultWithPostInfo {
			let from = Self::ensure_transaction_signature(transaction.clone())
				.map_err(|_| DispatchError::Other("Invalid transaction signature"))?;

			let origin: T::RuntimeOrigin =
				pallet_ethereum::Origin::EthereumTransaction(from).into();
			let dispatch = pallet_ethereum::Pallet::<T>::transact(origin, transaction)
				.map_err(|_| DispatchError::Other("Signature doesn't meet with sponsor address"))?;

			let used_gas = Self::gas_from_actual_weight(dispatch.actual_weight.unwrap());

			Ok(frame_support::dispatch::PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(
					used_gas.unique_saturated_into(),
					true,
				)),
				pays_fee: Pays::No,
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn gas_from_actual_weight(weight: Weight) -> u64 {
			let actual_weight = weight.saturating_add(
				T::BlockWeights::get()
					.get(frame_support::dispatch::DispatchClass::Normal)
					.base_extrinsic,
			);

			<T as pallet_evm::Config>::GasWeightMapping::weight_to_gas(actual_weight)
		}

		fn ensure_transaction_signature(
			transaction: pallet_ethereum::Transaction,
		) -> Result<H160, ()> {
			match stbl_tools::eth::recover_signer(&transaction) {
				None => Err(()),
				Some(address) => Ok(address),
			}
		}

		fn pool_ensure_transaction_unicity(
			origin: &H160,
			transaction: &pallet_ethereum::Transaction,
		) -> Result<(), ()> {
			let transaction_data: TransactionData = transaction.into();

			let (base_fee, _) = <T as pallet_evm::Config>::FeeCalculator::min_gas_price();
			let (who, _) = pallet_evm::Pallet::<T>::account_basic(origin);

			fp_evm::CheckEvmTransaction::<pallet_ethereum::InvalidTransactionWrapper>::new(
				fp_evm::CheckEvmTransactionConfig {
					evm_config: T::config(),
					block_gas_limit: T::BlockGasLimit::get(),
					base_fee,
					chain_id: T::ChainId::get(),
					is_transactional: true,
				},
				transaction_data.into(),
			)
			.validate_in_pool_for(&who)
			.and_then(|v| v.with_chain_id())
			.map_err(|_| ())?;

			Ok(())
		}
	}
}
