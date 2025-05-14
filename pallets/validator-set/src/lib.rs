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

use core::ops::Add;
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{EstimateNextSessionRotation, Get, ValidatorSet, ValidatorSetWithIdentification},
};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
pub use pallet::*;
use sp_runtime::traits::{Convert, Saturating, Zero};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

pub const LOG_TARGET: &'static str = "runtime::validator-set";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{
		offchain::SubmitTransaction,
		pallet_prelude::{BlockNumberFor, *},
	};

	use frame_support::traits::FindAuthor;

	use sp_application_crypto::RuntimeAppPublic;

	use frame_system::offchain::SendTransactionTypes;

	use sp_core::U256;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_session::Config + SendTransactionTypes<Call<Self>>
	{
		/// The Event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin for adding or removing a validator.
		type AddRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Minimum number of validators to leave in the validator set during
		/// auto removal.
		type MinAuthorities: Get<u32>;

		type SessionBlockManager: SessionBlockManager<BlockNumberFor<Self>>;

		type FindAuthor: FindAuthor<Self::AccountId>;

		type AuthorityId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ Ord
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		type MaxKeys: Get<u32>;

		type AccountIdOfValidator: Convert<Self::AuthorityId, Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn keys)]
	pub type Keys<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators)]
	pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn previous_validators)]
	pub type PreviousValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn approved_validators)]
	pub type ApprovedValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn to_be_added_validators)]
	pub type ToBeAddedValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn max_missed_epochs)]
	pub type MaxMissedEpochs<T: Config> = StorageValue<_, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn block_missed)]
	pub type EpochsMissed<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, U256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn block_authors)]
	pub type BlockAuthors<T: Config> =
		StorageMap<_, Twox64Concat, BlockNumberFor<T>, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New validator addition initiated. Effective in ~2 sessions.
		ValidatorAdditionInitiated(T::AccountId),

		/// Validator removal initiated. Effective in ~2 sessions.
		ValidatorRemovalInitiated(T::AccountId),
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
				Call::add_validator_again {
					heartbeat,
					signature,
				} => {
					Self::ensure_unsigned_origin(heartbeat.clone(), signature.clone())
						.map_err(|_| InvalidTransaction::BadProof)?;

					let account_id =
						T::AccountIdOfValidator::convert(heartbeat.clone().authority_id);

					if ToBeAddedValidators::<T>::get().contains(&account_id) {
						return InvalidTransaction::Call.into();
					}

					if Validators::<T>::get().contains(&account_id) {
						return InvalidTransaction::Call.into();
					}

					let sesion_index = pallet_session::Pallet::<T>::current_index();
					return ValidTransaction::with_tag_prefix("ValidatorSet")
						.priority(u64::MAX)
						.and_provides((sesion_index, heartbeat.clone().authority_id))
						.propagate(true)
						.build();
				}
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(block_number: BlockNumberFor<T>) {
			let digest = <frame_system::Pallet<T>>::digest();
			let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());

			if let Some(validator) = T::FindAuthor::find_author(pre_runtime_digests) {
				BlockAuthors::<T>::insert(block_number, validator);
			}
		}

		fn offchain_worker(now: BlockNumberFor<T>) {
			// Only send messages if we are a potential validator.
			if sp_io::offchain::is_validator() {
				let approved_validators = ApprovedValidators::<T>::get();
				Self::local_keys()
					.into_iter()
					.for_each(|(authority_index, key)| {
						let validator_id = approved_validators[authority_index as usize].clone();
						log::debug!(
							target: LOG_TARGET,
							"Checking heartbeat for validator {:?} ({:?}, {:?})",
							validator_id,
							Validators::<T>::get().contains(&validator_id),
							ToBeAddedValidators::<T>::get().contains(&validator_id)
						);
						if !Validators::<T>::get().contains(&validator_id)
							&& !ToBeAddedValidators::<T>::get().contains(&validator_id)
						{
							let heartbeat = Heartbeat {
								block_number: now,
								session_index: pallet_session::Pallet::<T>::current_index(),
								authority_index,
								authority_id: key.clone(),
							};

							let signature = key.clone().sign(&heartbeat.encode());

							if signature.is_none() {
								log::error!(
									target: LOG_TARGET,
									"Failed to sign heartbeat for validator {:?}",
									validator_id,
								);
								return;
							};

							let call = Call::<T>::add_validator_again {
								heartbeat: heartbeat.clone(),
								signature: signature.unwrap(),
							};

							match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
								call.into(),
							) {
								Err(_) => {
									log::error!(
										target: LOG_TARGET,
										"Failed to submit heartbeat transaction for validator {:?}: validator may already be active or transaction submission encountered an error",
										validator_id,
									);
								}
								_ => {
									log::info!(
										target: LOG_TARGET,
										"✅ Heartbeat transaction successfully submitted for validator (index: {:?}, id: {:?}) at block: {:?}",
										heartbeat.authority_index,
										validator_id,
										now,
									);
								}
							};
						};
					});
			} else {
				log::trace!(
					target: "runtime::validator-set",
					"Skipping heartbeat at {:?}. Not a validator.",
					now,
				)
			}
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_validators: Vec<T::AccountId>,
		pub max_epochs_missed: U256,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_validators: Default::default(),
				max_epochs_missed: 5.into(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_validators(self.initial_validators.clone());
			MaxMissedEpochs::<T>::put(self.max_epochs_missed.clone());
		}
	}

	impl<T: Config> Pallet<T> {
		fn local_keys() -> impl Iterator<Item = (u32, T::AuthorityId)> {
			// on-chain storage
			//
			// At index `idx`:
			// 1. A public key to be used by a validator at index `idx` to send im-online
			//          heartbeats.
			let authorities = ApprovedValidators::<T>::get();

			// local keystore
			//
			// All public (+private) keys currently in the local keystore.
			let mut local_keys = T::AuthorityId::all();

			local_keys.sort();

			authorities
				.into_iter()
				.enumerate()
				.filter_map(move |(index, authority)| {
					let account_ids = local_keys
						.iter()
						.map(|x| T::AccountIdOfValidator::convert(x.clone()))
						.collect::<Vec<T::AccountId>>();
					account_ids
						.binary_search(&authority)
						.ok()
						.map(|location| (index as u32, local_keys[location].clone()))
				})
		}

		fn ensure_unsigned_origin(
			heartbeat: Heartbeat<BlockNumberFor<T>, T::AuthorityId>,
			signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
		) -> Result<(), ()> {
			let is_valid = heartbeat
				.authority_id
				.clone()
				.verify(&heartbeat.encode(), &signature);

			if !is_valid {
				return Err(());
			}

			let approved_validators = ApprovedValidators::<T>::get();

			if approved_validators.len() <= heartbeat.authority_index as usize
				|| approved_validators[heartbeat.authority_index as usize]
					!= T::AccountIdOfValidator::convert(heartbeat.clone().authority_id)
			{
				return Err(());
			}

			Ok(())
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

	impl<T: Config> Pallet<T> {
		fn add_validator_weight() -> Weight {
			Weight::from_parts(21_330_000, 1602)
				.saturating_add(T::DbWeight::get().reads(1_u64))
				.saturating_add(T::DbWeight::get().writes(1_u64))
		}

		fn remove_validator_weight() -> Weight {
			Weight::from_parts(19_840_000, 1602)
				.saturating_add(T::DbWeight::get().reads(2_u64))
				.saturating_add(T::DbWeight::get().writes(2_u64))
		}

		fn add_validator_again_weight() -> Weight {
			Weight::from_parts(21_330_000, 1602)
				.saturating_add(T::DbWeight::get().reads(2_u64))
				.saturating_add(T::DbWeight::get().writes(1_u64))
		}

		fn update_max_missed_epochs_weight() -> Weight {
			Weight::from_parts(21_330_000, 1602).saturating_add(T::DbWeight::get().writes(1_u64))
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a new validator.
		///
		/// New validator's session keys should be set in Session pallet before
		/// calling this.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(0)]
		#[pallet::weight(Pallet::<T>::add_validator_weight())]
		pub fn add_validator(origin: OriginFor<T>, validator_id: T::AccountId) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::approve_validator(validator_id.clone())?;

			Ok(())
		}

		/// Remove a validator.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(1)]
		#[pallet::weight(Pallet::<T>::remove_validator_weight())]
		pub fn remove_validator(
			origin: OriginFor<T>,
			validator_id: T::AccountId,
		) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_remove_validator(validator_id.clone())?;
			Self::unapprove_validator(validator_id)?;

			Ok(())
		}

		/// Update max missed epochs.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(2)]
		#[pallet::weight(Pallet::<T>::update_max_missed_epochs_weight())]
		pub fn update_max_missed_epochs(
			origin: OriginFor<T>,
			max_missed_epochs: U256,
		) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			MaxMissedEpochs::<T>::put(max_missed_epochs);

			Ok(())
		}

		/// Add an approved validator again when it comes back online.
		///
		/// For this call, the dispatch origin must be the validator itself.
		#[pallet::call_index(3)]
		#[pallet::weight(Pallet::<T>::add_validator_again_weight())]
		pub fn add_validator_again(
			origin: OriginFor<T>,
			heartbeat: Heartbeat<BlockNumberFor<T>, T::AuthorityId>,
			_signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let validator_account_id =
				ApprovedValidators::<T>::get()[heartbeat.authority_index as usize].clone();

			if Self::validators().contains(&validator_account_id) {
				return DispatchError::Other("Validator already in the validator set.").into();
			}

			ToBeAddedValidators::<T>::mutate(|v| {
				v.push(validator_account_id.clone());
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn initialize_validators(account_ids: Vec<T::AccountId>) {
		assert!(
			<Validators<T>>::get().is_empty(),
			"Validators are already initialized!"
		);

		<Validators<T>>::put(account_ids.clone());
		<ApprovedValidators<T>>::put(account_ids);
	}

	fn do_remove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut validators = <Validators<T>>::get();

		let validators_count = match validators.len().checked_sub(1) {
			Some(v) => v,
			None => return Err(DispatchError::Other("Arithmetic error due to underflow.")),
		};

		// Ensuring that the post removal, target validator count doesn't go
		// below the minimum.
		ensure!(
			validators_count as u32 >= T::MinAuthorities::get(),
			Error::<T>::TooLowValidatorCount
		);

		validators.retain(|v| *v != validator_id);

		<Validators<T>>::put(validators);

		Self::deposit_event(Event::ValidatorRemovalInitiated(validator_id.clone()));
		log::debug!(
			target: LOG_TARGET,
			"Validator removal initiated: {:?}",
			validator_id.clone()
		);

		Ok(())
	}

	fn approve_validator(validator_id: T::AccountId) -> DispatchResult {
		ensure!(
			!<ApprovedValidators<T>>::get().contains(&validator_id),
			Error::<T>::Duplicate
		);
		log::debug!(
			target: LOG_TARGET,
			"Approving validator {:?}",
			validator_id.clone()
		);
		<ApprovedValidators<T>>::mutate(|v| v.push(validator_id.clone()));
		Ok(())
	}

	fn unapprove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut approved_set = <ApprovedValidators<T>>::get();
		approved_set.retain(|v| validator_id.ne(v));
		<ApprovedValidators<T>>::put(approved_set);
		Ok(())
	}

	// Adds offline validators to a local cache for removal at new session.
	fn increment_missed_block(validator_id: T::AccountId) {
		<EpochsMissed<T>>::mutate(validator_id.clone(), |v| {
			*v = v.clone().add(1);
		});
	}

	// Removes offline validators from the validator set and clears the offline
	// cache. It is called in the session change hook and removes the validators
	// who were reported offline during the session that is ending. We do not
	// check for `MinAuthorities` here, because the offline validators will not
	// produce blocks and will have the same overall effect on the runtime.
	fn update_validators() {
		Validators::<T>::mutate(|validators| {
			validators.retain(|x| {
				let missed_epochs = EpochsMissed::<T>::get(x.clone());
				if missed_epochs < MaxMissedEpochs::<T>::get() {
					true
				} else {
					log::debug!(
						target: LOG_TARGET,
						"Removing offline validator {:?}",
						x.clone()
					);
					EpochsMissed::<T>::remove(x.clone());
					false
				}
			});

			ToBeAddedValidators::<T>::take().iter().for_each(|x| {
				log::debug!(target: LOG_TARGET, "Adding validator {:?}", x.clone());
				validators.push(x.clone());
			});
		})
	}
}

// Provides the new set of validators to the session module when session is
// being rotated.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	// Plan a new session and provide new validator set.

	fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
		let validators = Self::validators();
		log::debug!(
			target: LOG_TARGET,
			"New session called; updated validator set provided: {:?}",
			validators
		);

		Some(validators)
	}

	fn end_session(end_index: u32) {
		log::debug!(target: LOG_TARGET, "Session ended.");

		// Add to offline validator list those validators who didn't mine a block in the session.
		let validators = Validators::<T>::get();

		// Get current block number
		let session_start_block = T::SessionBlockManager::session_start_block(end_index);
		let session_end_block =
			T::SessionBlockManager::session_start_block(end_index.saturating_add(1));

		let mut epoch_block_authors = BTreeSet::<T::AccountId>::new();

		let mut i = session_start_block;
		while i < session_end_block {
			let block_author = BlockAuthors::<T>::get(i);

			if let Some(author) = block_author {
				epoch_block_authors.insert(author);
			}
			i.saturating_inc();
		}

		for validator in validators {
			if epoch_block_authors.contains(&validator) {
				EpochsMissed::<T>::remove(validator);
			} else {
				Self::increment_missed_block(validator);
			}
		}

		Self::update_validators();
	}

	fn start_session(_start_index: u32) {}
}

impl<T: Config> EstimateNextSessionRotation<BlockNumberFor<T>> for Pallet<T> {
	fn average_session_length() -> BlockNumberFor<T> {
		Zero::zero()
	}

	fn estimate_current_session_progress(
		_now: BlockNumberFor<T>,
	) -> (
		Option<sp_runtime::Permill>,
		frame_support::pallet_prelude::Weight,
	) {
		(None, Zero::zero())
	}

	fn estimate_next_session_rotation(
		_now: BlockNumberFor<T>,
	) -> (
		Option<BlockNumberFor<T>>,
		frame_support::pallet_prelude::Weight,
	) {
		(None, Zero::zero())
	}
}

pub trait OpaqueKeysPublisher<PubKey1, PubKey2, Keys> {
	fn publish_keys(key: PubKey1, key2: PubKey2) -> Result<(), ()>;
	fn get_published_keys(key: PubKey1) -> Option<Keys>;
}

// Implementation of Convert trait for mapping ValidatorId with AccountId.
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::ValidatorId, Option<T::ValidatorId>> for ValidatorOf<T> {
	fn convert(account: T::ValidatorId) -> Option<T::ValidatorId> {
		Some(account)
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

impl<T: Config> ValidatorSet<T::AccountId> for Pallet<T> {
	type ValidatorId = T::ValidatorId;
	type ValidatorIdOf = T::ValidatorIdOf;

	fn session_index() -> sp_staking::SessionIndex {
		pallet_session::Pallet::<T>::current_index()
	}

	fn validators() -> Vec<Self::ValidatorId> {
		pallet_session::Pallet::<T>::validators()
	}
}

impl<T: Config> ValidatorSetWithIdentification<T::AccountId> for Pallet<T> {
	type Identification = T::ValidatorId;
	type IdentificationOf = ValidatorOf<T>;
}

pub trait SessionBlockManager<BlockNumber> {
	fn session_start_block(session_index: sp_staking::SessionIndex) -> BlockNumber;
}
