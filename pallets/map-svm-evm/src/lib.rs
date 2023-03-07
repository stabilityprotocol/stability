#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use sp_std::prelude::*;

use frame_support::dispatch::Pays;

use sp_core::H160;

use sp_io::crypto::secp256k1_ecdsa_recover;
use sp_io::hashing::keccak_256;

use frame_support::dispatch::DispatchResultWithPostInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*};
	use pallet_evm::Runner;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
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
		/// Account not linked to any evm account
		AccountNotLinked,
		/// Fail calling to ERC1271 smart contract
		FailCallingErc1271,
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

			let expected_message_hash = Self::get_expected_message_hash(address);

			if signature.len() != 65 {
				return Err(Error::<T>::InvalidSignature.into());
			}

			let signature_fixed: &[u8; 65] = signature[0..65]
				.try_into()
				.map_err(|_| Error::<T>::InvalidSignature)?;

			if pallet_evm::AccountCodes::<T>::get(address).is_empty() {
				//Address is an EOA
				let pubkey = secp256k1_ecdsa_recover(signature_fixed, &expected_message_hash)
					.map_err(|_| Error::<T>::RecoverFailed)?;

				let pubkey_hash = keccak_256(&pubkey);

				let address_recovered = H160::from_slice(&pubkey_hash[12..32]);

				if address != address_recovered {
					return Err(Error::<T>::AddressNotMatch.into());
				}
			} else {
				Self::check_erc_1271(address, &expected_message_hash, signature_fixed)?;
			}

			// mutate the storage for set as linked evm and substrate
			SubstrateToEvm::<T>::insert(who.clone(), address);
			EvmToSubstrate::<T>::insert(address, who.clone());

			// mutate the nonce
			EvmLinkNonce::<T>::mutate(address, |nonce| *nonce += 1);

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

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub linked_accounts: Vec<(T::AccountId, H160)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				linked_accounts: Vec::new(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_linked_accounts(&self.linked_accounts);
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_linked_substrate_account(address: H160) -> Option<T::AccountId> {
			EvmToSubstrate::<T>::get(address)
		}

		pub fn get_linked_evm_account(account: T::AccountId) -> Option<H160> {
			SubstrateToEvm::<T>::get(account)
		}

		pub fn unlink_account_from_evm_account(address: H160) -> Result<T::AccountId, ()> {
			let account = EvmToSubstrate::<T>::get(address);

			if let None = account {
				return Err(());
			}

			EvmToSubstrate::<T>::remove(address);
			SubstrateToEvm::<T>::remove(account.clone().unwrap());

			Ok(account.unwrap())
		}

		pub fn initialize_linked_accounts(linked_accounts: &Vec<(T::AccountId, H160)>) {
			for (account, address) in linked_accounts {
				SubstrateToEvm::<T>::insert(account.clone(), address);
				EvmToSubstrate::<T>::insert(address, account.clone());
				EvmLinkNonce::<T>::insert(address, 1);
			}
		}

		fn encode_abi_with_signature_erc_1271(message: &[u8; 32], signature: &[u8; 65]) -> Vec<u8> {
			let mut encoded: Vec<u8> = Vec::new();

			let method_signature_hash = keccak_256(b"isValidSignature(bytes32,bytes)");
			let method_signature: &[u8; 4] = method_signature_hash[0..4].try_into().unwrap();

			encoded.extend_from_slice(method_signature);
			encoded.extend_from_slice(message);
			encoded.extend_from_slice(
				(vec![
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65,
				])
				.as_slice(),
			);
			encoded.extend_from_slice(signature);
			encoded.extend_from_slice(
				(vec![
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0, 0, 0,
				])
				.as_slice(),
			);
			encoded
		}

		fn check_erc_1271(
			address: H160,
			message: &[u8; 32],
			signature: &[u8; 65],
		) -> Result<(), Error<T>> {
			let encoded_data = Self::encode_abi_with_signature_erc_1271(&message, signature);
			let enconded_response = T::Runner::call(
				Default::default(),
				address,
				encoded_data,
				Default::default(),
				u64::MAX,
				None,
				None,
				None,
				Default::default(),
				false,
				false,
				&pallet_evm::EvmConfig::istanbul(),
			)
			.map_err(|_| Error::<T>::FailCallingErc1271)?;

			if !Self::is_successful_response_erc_1271(&enconded_response.value) {
				return Err(Error::<T>::InvalidSignature.into());
			}
			Ok(())
		}

		fn get_expected_message_hash(address: H160) -> [u8; 32] {
			let nonce = EvmLinkNonce::<T>::get(address);

			let nonce_string = u64_to_buffer_in_ascii(nonce);

			let chain_id = T::ChainId::get();
			let chain_id_string = u64_to_buffer_in_ascii(chain_id);


			let message = b"I consent to bind my ETH address for time "
				.iter()
				.chain(nonce_string.iter())
				.chain(b" in chain: ".iter())
				.chain(chain_id_string.iter())
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

			keccak_256(expected_message.as_slice())
		}

		fn is_successful_response_erc_1271(response: &[u8]) -> bool {
			let success_response: Vec<u8> = vec![
				22, 38, 186, 126, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0,
			];

			if response.ne(&success_response) {
				return false;
			}

			true
		}
	}
}

fn u64_to_buffer_in_ascii(u: u64) -> Vec<u8> {
	let mut buffer = Vec::new();
	let mut u = u;

	if u == 0 {
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
