// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

//! # Validator Set Pallet
//!
//! The Validator Set Pallet allows addition and removal of
//! authorities/validators via extrinsics (transaction calls), in
//! Substrate-based PoA networks. It also integrates with the im-online pallet
//! to automatically remove offline validators.
//!
//! The pallet uses the Session pallet and implements related traits for session
//! management. Currently it uses periodic session rotation provided by the
//! session pallet to automatically rotate sessions. For this reason, the
//! validator addition and removal becomes effective only after 2 sessions
//! (queuing + applying).

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

use frame_support::pallet_prelude::*;
use log;
pub use pallet::*;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

pub const LOG_TARGET: &'static str = "runtime::validator-set";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{offchain::SubmitTransaction, pallet_prelude::*};

	use sp_application_crypto::RuntimeAppPublic;

	use frame_system::offchain::SendTransactionTypes;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_session::Config
		+ pallet_validator_set::Config
		+ SendTransactionTypes<Call<Self>>
	{
		/// The Event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type FinalizationId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ Ord
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		type SessionKeysBuilder: SessionKeysBuilder<
			Self::AuthorityId,
			Self::FinalizationId,
			Self::Keys,
		>;

		type ValidatorIdOfValidation: Convert<Self::AuthorityId, Self::ValidatorId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New validator addition initiated. Effective in ~2 sessions.
		ValidatorKeysPublished(T::ValidatorId, T::Keys),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Target (post-removal) validator count is below the minimum.
		TooLowValidatorCount,
		/// Validator is already in the validator set.
		Duplicate,
		/// Validator is not approved for re-addition.
		ValidatorNotApproved,
		/// Only the validator can add itself back after coming online.
		BadOrigin,
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::publish_keys { keys, signature } => {
					let validator_id = T::ValidatorIdOfValidation::convert(keys.aura.clone());

					Self::verify_signature(keys.clone(), signature.clone()).map_err(|_| {
						log::error!(target: LOG_TARGET, "Failed to verify signature",);
						InvalidTransaction::BadProof
					})?;

					if pallet_session::NextKeys::<T>::get(validator_id).is_some() {
						return InvalidTransaction::Call.into();
					}

					let sesion_index = pallet_session::Pallet::<T>::current_index();

					return ValidTransaction::with_tag_prefix("ValidatorKeysController")
						.priority(u64::MAX)
						.and_provides((sesion_index, keys.aura.clone()))
						.propagate(true)
						.build();
				}
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(now: BlockNumberFor<T>) {
			// Only send messages if we are a potential validator.
			if sp_io::offchain::is_validator() {
				Self::offchain_publish_keys(now);
			} else {
				log::trace!(
					target: "runtime::validator-keys-controller",
					"Skipping heartbeat at {:?}. Not a validator.",
					now,
				)
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn offchain_publish_keys(block_number: BlockNumberFor<T>) {
			let aura_local_keys = T::AuthorityId::all();
			let grandpa_local_keys = T::FinalizationId::all();

			aura_local_keys
				.iter()
				.zip(grandpa_local_keys.iter())
				.for_each(|(aura, grandpa)| {
					let validator_id = T::ValidatorIdOfValidation::convert(aura.clone());
					let account_id = T::AccountIdOfValidator::convert(aura.clone());
					if pallet_validator_set::Validators::<T>::get().contains(&account_id)
						&& pallet_session::NextKeys::<T>::get(validator_id).is_none()
					{
						let keys = PublishingKeys {
							aura: aura.clone(),
							grandpa: grandpa.clone(),
							block_number,
						};

						let signature = aura.sign(&keys.encode());

						if signature.is_none() {
							log::error!(target: LOG_TARGET, "Failed to sign keys",);
							return;
						}

						let signature = signature.unwrap();

						let call = Call::<T>::publish_keys { keys, signature };

						match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
							call.into(),
						) {
							Err(_) => {
								log::error!(target: LOG_TARGET, "Failed to submit transaction",);
							}
							_ => {
								log::info!(
									target: LOG_TARGET,
									"🔑 Successfully inserted new validator keys",
								);
							}
						};
					}
				});
		}

		fn verify_signature(
			publishing_keys: PublishingKeys<T::AuthorityId, T::FinalizationId, BlockNumberFor<T>>,
			signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
		) -> Result<(), ()> {
			if publishing_keys
				.aura
				.verify(&publishing_keys.encode(), &signature)
			{
				Ok(())
			} else {
				Err(())
			}
		}
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct Heartbeat<BlockNumber, AuthorityId>
	where
		BlockNumber: PartialEq + Eq + Decode + Encode,
	{
		/// Block number at the time heartbeat is created..
		pub block_number: BlockNumber,
		/// Index of the current session.
		pub session_index: sp_staking::SessionIndex,
		/// An index of the authority on the list of approved validators.
		pub authority_id: AuthorityId,
		/// An index of the authority on the list of approved validators.
		pub authority_index: u32,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct PublishingKeys<AuthorityId, FinalizationId, BlockNumber> {
		pub aura: AuthorityId,
		pub grandpa: FinalizationId,
		pub block_number: BlockNumber,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn publish_keys(
			origin: OriginFor<T>,
			keys: PublishingKeys<T::AuthorityId, T::FinalizationId, BlockNumberFor<T>>,
			_signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let validator_id = T::ValidatorIdOfValidation::convert(keys.aura.clone());

			let session_keys = T::SessionKeysBuilder::new(keys.aura.clone(), keys.grandpa.clone());

			if pallet_validator_set::ApprovedValidators::<T>::get()
				.iter()
				.find(|x| (*x).eq(&T::AccountIdOfValidator::convert(keys.aura.clone())))
				.is_none()
			{
				return Err(Error::<T>::ValidatorNotApproved.into());
			}

			pallet_session::NextKeys::<T>::insert(validator_id.clone(), session_keys.clone());

			pallet_session::QueuedKeys::<T>::mutate(|x| {
				x.push((validator_id.clone(), session_keys.clone()));
			});

			pallet_session::QueuedChanged::<T>::set(true);

			Self::deposit_event(Event::ValidatorKeysPublished(
				validator_id.clone(),
				session_keys,
			));

			Ok(())
		}
	}
}

pub trait SessionKeysBuilder<K1, K2, Keys> {
	fn new(k1: K1, k2: K2) -> Keys;
}
