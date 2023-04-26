#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::{H160, U256};
use sp_std::prelude::*;

use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::dispatch::Pays;
use frame_support::parameter_types;
use stbl_tools::{none_or_err, some_or_err};

parameter_types! {
	pub ZeroAddress:H160 = H160::from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as [u8; 20]);
}

pub trait DelegatedTransaction<T: pallet::Config> {
	type Error;

	fn delegate_transaction(
		origin: OriginFor<T>,
		to: H160,
		input: Vec<u8>,
	) -> Result<(), Self::Error>;

	fn execute_delegated_transaction(
		origin: OriginFor<T>, // delegate
		delegator: T::PublicKey,
		to: H160,
		input: Vec<u8>,
		nonce: u64,
		signature: T::Signature,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
	) -> DispatchResultWithPostInfo;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, Account};
	use pallet_evm::{EnsureAddressOrigin, Runner};
	use runner::{Runner as RunnerImpl, RunnerExt};
	use sp_io::hashing::blake2_256;
	use sp_runtime::traits::{AccountIdConversion, IdentifyAccount, Verify};
	use sp_runtime::verify_encoded_lazy;

	/* pub const TX_TYPEHASH: [u8; 32] = keccak256!(
		"CallPermit(address from,address to,uint256 value,bytes data,uint64 gaslimit\
	,uint256 nonce,uint256 deadline)"
	); */

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Signature: Verify<Signer = Self::PublicKey> + Encode + Decode + Parameter;
		type PublicKey: IdentifyAccount<AccountId = Self::PublicKey> + Encode + Decode + Parameter;
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransaction,
		InvalidSignature,
		InvalidDelegatorAddress,
		NotSigned,
		DelegateIsNotOrigin,
		TransactionNonceUnavailable,
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::type_value]
	pub(super) fn DefaultNonce<T: Config>() -> u64 {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn delegator_nonce)]
	pub(super) type DelegatorNonce<T: Config> =
		StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery, DefaultNonce<T>>; // delegator -> next_nonce

	// TODO: optimize to only hold active nonces and remove old ones and empty accounts
	#[pallet::storage]
	#[pallet::getter(fn delegator_txs_by_nonce)]
	pub(super) type DelegatorTxsByNonce<T: Config> = StorageDoubleMap<
		_,
		// Delegator
		Blake2_128Concat,
		H160,
		// Nonce
		Blake2_128Concat,
		u64,
		// Has been called
		bool,
		OptionQuery,
	>; // delegator -> nonce -> signed data

	impl<T: Config> Pallet<T> {
		pub fn hash_tx(delegator: &H160, to: &H160, input: &[u8], nonce: &u64) -> [u8; 32] {
			let encoded_tx = (&delegator, &to, &input, &nonce).encode();

			blake2_256(&encoded_tx)
		}
	}

	/* #[pallet::call] */
	impl<T: Config> DelegatedTransaction for Pallet<T> {
		type Error = Error<T>;

		/* pub fn delegate_transaction(
			origin: OriginFor<T>,
			to: H160,
			input: Vec<u8>,
		) -> Result<([u8; 32], u64), Error<T>> {
			let delegator_result = ensure_signed(origin);
			if delegator_result.is_err() {
				return Err(Error::<T>::NotSigned);
			}
			let delegator = delegator_result.unwrap();

			let outcome = Account::<T>::get(&delegator);
			let outcome2: H160 = outcome.clone().into();

			let delegator_h160 = H160::from_slice(&delegator.encode()[12..]);

			// let delegator_h160: H160 = delegator::Get().clone_into();

			let transaction_nonce = DelegatorNonce::<T>::get(delegator_h160.clone());

			DelegatorNonce::<T>::mutate(delegator_h160.clone(), |nonce| *nonce += 1);
			DelegatorTxsByNonce::<T>::insert(delegator_h160.clone(), transaction_nonce, true);

			let hashedPayload = Self::hash_tx(&delegator_h160, &to, &input, &transaction_nonce);
			let completed_values = (hashedPayload, transaction_nonce);

			Ok(completed_values)
		}
		*/
		/* #[pallet::call_index(0)]
		#[pallet::weight(0)] */
		pub fn delegate_transaction(
			origin: OriginFor<T>,
			to: H160,
			input: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			/* let delegator_result = ensure_signed(origin)?;
			let delegator = delegator_result.clone();

			let outcome = Account::<T>::get(&delegator);
			let outcome2: H160 = outcome.clone().into();

			let delegator_h160 = H160::from_slice(&delegator.encode()[12..]);

			// let delegator_h160: H160 = delegator::Get().clone_into();

			let transaction_nonce = DelegatorNonce::<T>::get(delegator_h160.clone());

			DelegatorNonce::<T>::mutate(delegator_h160.clone(), |nonce| *nonce += 1);
			DelegatorTxsByNonce::<T>::insert(delegator_h160.clone(), transaction_nonce, true);

			let hashedPayload = Self::hash_tx(&delegator_h160, &to, &input, &transaction_nonce);
			let completed_values = (hashedPayload, transaction_nonce); */

			Ok(().into())
		}

		// Take encoded_data, signature
		// If matches signature, execute
		/* #[pallet::call_index(1)]
		#[pallet::weight(0)] */
		pub fn execute_delegated_transaction(
			origin: OriginFor<T>, // delegate
			delegator: T::PublicKey,
			to: H160,
			input: Vec<u8>,
			nonce: u64,
			signature: T::Signature,
			gas_limit: u64,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
		) -> DispatchResultWithPostInfo {
			/* let delegate = ensure_signed(origin)?;
			if delegator == ZeroAddress::get() {
				return Err(Error::<T>::InvalidDelegatorAddress.into());
			}

			let delegator_h160: H160 = delegator.into_account().get().into();
			let delegate_h160: H160 = delegate.get().into();

			// Prevent replay attacks
			if DelegatorTxsByNonce::<T>::contains_key(delegator, nonce) {
				return Err(Error::<T>::TransactionNonceUnavailable.into());
			}

			let encoded_tx_hash = Self::hash_tx(&delegator_h160, &to, &input, &nonce);

			// Verify transaction data hasn't been tampered with
			let delegator_is_signer =
				verify_encoded_lazy(&signature, &encoded_tx_hash, &delegator.into_account());

			if !delegator_is_signer {
				return Err(Error::<T>::InvalidSignature.into());
			}

			// Execute transaction
			let execution_result = RunnerImpl::call_delegated(
				delegator_h160,
				delegate_h160,
				to,
				input,
				U256::from(0),
				gas_limit,
				None,
				None,
				None, // TODO: should I manually pass a nonce?
				Vec::new(),
				false, // TODO: does this need to be set based on transaction type?
				false,
				&pallet_evm::EvmConfig::istanbul(),
			)?;

			// Set nonce as used
			DelegatorTxsByNonce::<T>::remove(delegator_h160, nonce); */

			Ok(().into())
		}
	}
}
