#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use sp_runtime::AccountId32;
use sp_std::prelude::*;

use frame_support::dispatch::Pays;

use sp_core::H160;

use sp_io::crypto::secp256k1_ecdsa_recover;
use sp_io::hashing::{blake2_256, keccak_256};

use sp_runtime::traits::{MaybeDisplay, MaybeSerializeDeserialize, Member};

use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	traits::{Get, PalletInfo, StoredMap, TypedGet},
	Parameter,
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A substrate account is linked to an EVM account
		AccountLinked {
			substrate: T::AccountId,
			evm: H160,
		},
		AccountUnlinked {
			substrate: T::AccountId,
			evm: H160,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The Substrate account is already linked to an EVM account.
		SubstrateAlreadyLinked,
		/// The Evm account is already linked to an Substrate account.
		EvmAlreadyLinked,
		/// Fail to recover the EVM account from the signature.
		RecoverFailed,
		/// The address from signature does not match with the received address.
		AddressNotMatch,
		/// Invalidad signature.
		InvalidSignature,
		/// The message is not valid.
		InvalidMessage,
		// Account not linked to any evm account
		AccountNotLinked,
	}

	#[pallet::storage]
	#[pallet::getter(fn substrate_to_evm)]
	pub(super) type SubstrateToEvm<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, H160, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn evm_to_substrate)]
	pub(super) type EvmToSubstrate<T: Config> =
		StorageMap<_, Blake2_128Concat, H160, T::AccountId, OptionQuery>;

	#[pallet::type_value]
	pub(super) fn DefaultNonce<T: Config>() -> u64 {
		0
	}
	#[pallet::storage]
	#[pallet::getter(fn evm_link_nonce)]
	pub(super) type EvmLinkNonce<T: Config> =
		StorageMap<_, Blake2_128Concat, H160, u64, ValueQuery, DefaultNonce<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn link_evm_account(
			origin: OriginFor<T>,
			address: H160,
			signature: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let evm_linked_account = SubstrateToEvm::<T>::get(who.clone());

			if let Some(_) = evm_linked_account {
				return Err(Error::<T>::SubstrateAlreadyLinked.into());
			}

			let substrate_linked_account = EvmToSubstrate::<T>::get(address);

			if let Some(_) = substrate_linked_account {
				return Err(Error::<T>::EvmAlreadyLinked.into());
			}

			let nonce = EvmLinkNonce::<T>::get(address);

			let nonce_string = u64_to_buffer_in_ascii(nonce);

			let message = b"I consent to bind my ETH address for time "
				.iter()
				.chain(nonce_string.iter())
				.cloned()
				.collect::<Vec<u8>>();

			let message_len = message.len();

			let message_len_string = u64_to_buffer_in_ascii(message_len.try_into().unwrap());

			let expected_message = b"\x19Ethereum Signed Message:\n"
				.iter()
				.chain(message_len_string.iter())
				.chain(message.iter())
				.cloned()
				.collect::<Vec<u8>>();

			let expected_message_hash = keccak_256(expected_message.as_slice());

			let signature_fixed: &[u8; 65] = signature[0..65]
				.try_into()
				.map_err(|_| Error::<T>::InvalidSignature)?;

			let pubkey = secp256k1_ecdsa_recover(signature_fixed, &expected_message_hash)
				.map_err(|_| Error::<T>::RecoverFailed)?;

			let pubkey_hash = keccak_256(&pubkey);

			let address_recovered = H160::from_slice(&pubkey_hash[12..32]);

			if address != address_recovered {
				return Err(Error::<T>::AddressNotMatch.into());
			}

			// mutate the storage for set as linked evm and substrate
			SubstrateToEvm::<T>::insert(who.clone(), address);
			EvmToSubstrate::<T>::insert(address, who.clone());

			// mutate the nonce
			EvmLinkNonce::<T>::insert(address, nonce + 1);

			// emit event
			Self::deposit_event(Event::AccountLinked {
				substrate: who,
				evm: address,
			});

			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn unlink_evm_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let evm_linked_account_option = SubstrateToEvm::<T>::get(who.clone());

			if let None = evm_linked_account_option {
				return Err(Error::<T>::AccountNotLinked.into());
			}

			let evm_linked_account = evm_linked_account_option.unwrap();

			// mutate the storage for unset linked evm and substrate
			SubstrateToEvm::<T>::remove(who.clone());
			EvmToSubstrate::<T>::remove(evm_linked_account);

			// emit event
			Self::deposit_event(Event::AccountUnlinked {
				substrate: who,
				evm: evm_linked_account,
			});

			Ok(Pays::No.into())
		}
	}
}

fn u64_to_buffer_in_ascii(u: u64) -> Vec<u8> {
	let mut buffer = Vec::new();
	let mut u = u;

	if (u == 0) {
		buffer.push(48);
		return buffer;
	}

	while u > 0 {
		let digit = u % 10;
		buffer.push((digit + 48) as u8);
		u /= 10;
	}
	buffer.reverse();
	buffer
}
