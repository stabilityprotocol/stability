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
use sp_runtime::BoundedSlice;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{EstimateNextSessionRotation, Get, ValidatorSet, ValidatorSetWithIdentification},
};
use log;
pub use pallet::*;
use sp_runtime::traits::{Convert, Zero};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

pub const LOG_TARGET: &'static str = "runtime::validator-set";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::{offchain::SubmitTransaction, pallet_prelude::*};

	use frame_support::{traits::FindAuthor, WeakBoundedVec};

	use sp_application_crypto::RuntimeAppPublic;

	use frame_system::offchain::SendTransactionTypes;

	use sp_core::U256;

	/// The current set of keys that may issue a heartbeat.
	#[pallet::storage]
	#[pallet::getter(fn keys)]
	pub(crate) type Keys<T: Config> =
		StorageValue<_, WeakBoundedVec<T::AuthorityId, T::MaxKeys>, ValueQuery>;

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

		type SessionBlockManager: SessionBlockManager<Self::BlockNumber>;

		type FindAuthor: FindAuthor<Self::AccountId>;

		type AuthorityId: Member
			+ Parameter
			+ RuntimeAppPublic
			+ Ord
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		type MaxKeys: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

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
	pub type BlockAuthors<T: Config> = StorageMap<
		_,
		Twox64Concat,
		<T as frame_system::Config>::BlockNumber,
		T::AccountId,
		OptionQuery,
	>;

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
			if let Call::add_validator_again {
				heartbeat,
				signature,
			} = call
			{
				let is_valid = heartbeat
					.authority_id
					.verify(&heartbeat.encode(), &signature);

				if !is_valid {
					return InvalidTransaction::BadProof.into();
				}

				let sesion_index = pallet_session::Pallet::<T>::current_index();
				return ValidTransaction::with_tag_prefix("ImOnline")
					.priority(TransactionPriority::max_value())
					.and_provides((sesion_index, heartbeat.clone().authority_id))
					.longevity(
						TryInto::<u64>::try_into(
							T::NextSessionRotation::average_session_length() / 2u32.into(),
						)
						.unwrap_or(64_u64),
					)
					.propagate(true)
					.build();
			}

			return InvalidTransaction::Call.into();
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
								heartbeat,
								signature: signature.unwrap(),
							};

							match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
								call.into(),
							) {
								Err(_) => {
									log::error!(target: LOG_TARGET, "Failed to submit transaction",);
								}
								_ => {
									log::info!(
										target: LOG_TARGET,
										"Successfully submitted transaction",
									);
								}
							};
						};
					});
			} else {
				log::trace!(
					target: "runtime::im-online",
					"Skipping heartbeat at {:?}. Not a validator.",
					now,
				)
			}
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_validators: Vec<T::AccountId>,
		pub inital_keys: Vec<T::AuthorityId>,
		pub max_blocks_missed: U256,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_validators: Default::default(),
				inital_keys: Default::default(),
				max_blocks_missed: 5.into(),
			}
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
			<Self as GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
			<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_validators(&self.initial_validators);
			Pallet::<T>::initialize_keys(&self.inital_keys);
		}
	}

	impl<T: Config> Pallet<T> {
		fn local_keys() -> impl Iterator<Item = (u32, T::AuthorityId)> {
			// on-chain storage
			//
			// At index `idx`:
			// 1. A (ImOnline) public key to be used by a validator at index `idx` to send im-online
			//          heartbeats.
			let authorities = Keys::<T>::get();

			// local keystore
			//
			// All `ImOnline` public (+private) keys currently in the local keystore.
			let mut local_keys = T::AuthorityId::all();

			local_keys.sort();

			authorities
				.into_iter()
				.enumerate()
				.filter_map(move |(index, authority)| {
					local_keys
						.binary_search(&authority)
						.ok()
						.map(|location| (index as u32, local_keys[location].clone()))
				})
		}

		fn initialize_keys(keys: &[T::AuthorityId]) {
			if !keys.is_empty() {
				assert!(Keys::<T>::get().is_empty(), "Keys are already initialized!");
				let bounded_keys = <BoundedSlice<'_, _, T::MaxKeys>>::try_from(keys)
					.expect("More than the maximum number of keys provided");
				Keys::<T>::put(bounded_keys);
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
		#[pallet::weight(0)]
		pub fn add_validator(origin: OriginFor<T>, validator_id: T::AccountId) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_add_validator(validator_id.clone())?;
			Self::approve_validator(validator_id)?;

			Ok(())
		}

		/// Remove a validator.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
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
		#[pallet::weight(0)]
		pub fn add_validator_again(
			origin: OriginFor<T>,
			heartbeat: Heartbeat<T::BlockNumber, T::AuthorityId>,
			signature: <T::AuthorityId as RuntimeAppPublic>::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			let signature_valid = heartbeat.clone().using_encoded(|encoded_heartbeat| {
				heartbeat
					.clone()
					.authority_id
					.verify(&encoded_heartbeat, &signature)
			});

			ensure!(signature_valid, Error::<T>::BadOrigin);

			let validator_account_id =
				ApprovedValidators::<T>::get()[heartbeat.authority_index as usize].clone();

			ToBeAddedValidators::<T>::mutate(|v| v.push(validator_account_id));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn initialize_validators(validators: &[T::AccountId]) {
		assert!(
			validators.len() as u32 >= T::MinAuthorities::get(),
			"Initial set of validators must be at least T::MinAuthorities"
		);
		assert!(
			<Validators<T>>::get().is_empty(),
			"Validators are already initialized!"
		);

		<Validators<T>>::put(validators);
		<ApprovedValidators<T>>::put(validators);
	}

	fn do_add_validator(validator_id: T::AccountId) -> DispatchResult {
		ensure!(
			!<Validators<T>>::get().contains(&validator_id),
			Error::<T>::Duplicate
		);
		<Validators<T>>::mutate(|v| v.push(validator_id.clone()));
		<EpochsMissed<T>>::insert(validator_id.clone(), sp_core::U256::zero());

		Self::deposit_event(Event::ValidatorAdditionInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator addition initiated.");

		Ok(())
	}

	fn do_remove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut validators = <Validators<T>>::get();

		// Ensuring that the post removal, target validator count doesn't go
		// below the minimum.
		ensure!(
			validators.len().saturating_sub(1) as u32 >= T::MinAuthorities::get(),
			Error::<T>::TooLowValidatorCount
		);

		validators.retain(|v| *v != validator_id);

		<Validators<T>>::put(validators);

		Self::deposit_event(Event::ValidatorRemovalInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator removal initiated.");

		Ok(())
	}

	fn approve_validator(validator_id: T::AccountId) -> DispatchResult {
		ensure!(
			!<ApprovedValidators<T>>::get().contains(&validator_id),
			Error::<T>::Duplicate
		);
		<ApprovedValidators<T>>::mutate(|v| v.push(validator_id.clone()));
		Ok(())
	}

	fn unapprove_validator(validator_id: T::AccountId) -> DispatchResult {
		let mut approved_set = <ApprovedValidators<T>>::get();
		approved_set.retain(|v| *v != validator_id);
		<ApprovedValidators<T>>::put(approved_set);
		Ok(())
	}

	// Adds offline validators to a local cache for removal at new session.
	fn increment_missed_block(validator_id: T::AccountId) {
		<EpochsMissed<T>>::mutate(validator_id.clone(), |v| {
			*v = v.clone().add(1);
			log::debug!(
				target: LOG_TARGET,
				"Missed epoch by {:?} total count: {:?}",
				validator_id.clone(),
				v.clone()
			);
		});
	}

	// Removes offline validators from the validator set and clears the offline
	// cache. It is called in the session change hook and removes the validators
	// who were reported offline during the session that is ending. We do not
	// check for `MinAuthorities` here, because the offline validators will not
	// produce blocks and will have the same overall effect on the runtime.
	fn update_validators() {
		PreviousValidators::<T>::put(Validators::<T>::get());

		// Delete from active validator set.
		<Validators<T>>::mutate(|vs| {
			vs.retain(|v| <EpochsMissed<T>>::get(v) < MaxMissedEpochs::<T>::get());
		});

		ToBeAddedValidators::<T>::mutate(|pending_vs| {
			pending_vs
				.iter()
				.for_each(|v| match Self::do_add_validator(v.clone()) {
					Err(_) => {
						log::error!(target: LOG_TARGET, "Failed to add validator")
					}
					Ok(_) => {
						log::info!(target: LOG_TARGET, "Added validator")
					}
				});
			*pending_vs = vec![];
		});
	}
}

// Provides the new set of validators to the session module when session is
// being rotated.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	// Plan a new session and provide new validator set.
	fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
		log::debug!(
			target: LOG_TARGET,
			"New session called; updated validator set provided."
		);

		Some(Self::validators())
	}

	fn end_session(end_index: u32) {
		log::debug!(target: LOG_TARGET, "Session ended.");

		// Add to offline validator list those validators who didn't mine a block in the session.
		let validators = PreviousValidators::<T>::get();

		// Get current block number
		let session_start_block = T::SessionBlockManager::session_start_block(end_index);
		let session_end_block = T::SessionBlockManager::session_start_block(end_index + 1);

		let mut epoch_block_authors = BTreeSet::<T::AccountId>::new();

		let mut i = session_start_block;
		while i < session_end_block {
			let block_author = BlockAuthors::<T>::get(i);

			if let Some(author) = block_author {
				epoch_block_authors.insert(author);
			}
			i += 1u32.into();
		}

		for validator in validators {
			if !epoch_block_authors.contains(&validator) {
				Self::increment_missed_block(validator);
			}
		}

		Self::update_validators();
	}

	fn start_session(_start_index: u32) {}
}

impl<T: Config> EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
	fn average_session_length() -> T::BlockNumber {
		Zero::zero()
	}

	fn estimate_current_session_progress(
		_now: T::BlockNumber,
	) -> (Option<sp_runtime::Permill>, frame_support::dispatch::Weight) {
		(None, Zero::zero())
	}

	fn estimate_next_session_rotation(
		_now: T::BlockNumber,
	) -> (Option<T::BlockNumber>, frame_support::dispatch::Weight) {
		(None, Zero::zero())
	}
}

// Implementation of Convert trait for mapping ValidatorId with AccountId.
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::ValidatorId, Option<T::ValidatorId>> for ValidatorOf<T> {
	fn convert(account: T::ValidatorId) -> Option<T::ValidatorId> {
		Some(account)
	}
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
