#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use sp_std::prelude::*;

use frame_support::dispatch::GetDispatchInfo;
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
	pub trait Config: frame_system::Config {}

	#[pallet::error]
	pub enum Error<T> {
		/// The Substrate account is already linked to an EVM account.
		AlreadyLinked,
		/// Fail to recover the EVM account from the signature.
		RecoverFailed,
		/// The address from signature does not match with the received address.
		AddressNotMatch,
		/// Invalidad signature.
		InvalidSignature,
	}

	#[pallet::storage]
	#[pallet::getter(fn substrate_to_evm)]
	pub(super) type SubstrateToEvm<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, H160, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn link_evm_account(
			origin: OriginFor<T>,
			address: H160,
			message: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let linkedAccount = SubstrateToEvm::<T>::get(who);

			if let Some(_) = linkedAccount {
				return Err(Error::<T>::AlreadyLinked.into());
			}

			// check ethereum message
			let expectedMessage = b"\x19Ethereum Signed Message:\n93"
				.iter()
				.chain(who.encode().iter())
				.cloned()
				.collect::<Vec<u8>>();

			let signatureFixed: &[u8; 65] = signature[0..65]
				.try_into()
				.map_err(|_| Error::<T>::InvalidSignature)?;

			let messageHash = blake2_256(message.as_slice());

			let pubkey = secp256k1_ecdsa_recover(signatureFixed, &messageHash)
				.map_err(|_| Error::<T>::RecoverFailed)?;

			let addressRecovered = H160::from_slice(&keccak_256(&pubkey)[12..32]);

			if address != addressRecovered {
				return Err(Error::<T>::AddressNotMatch.into());
			}

			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn unlink_evm_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ok(Pays::No.into())
		}
	}
}

pub trait AccountIdEncode {
	type AccountId: Member + Parameter + MaybeSerializeDeserialize + MaybeDisplay + Ord + Default;

	fn encode_account_id(account_id: Self::AccountId) -> Vec<u8>;
}
