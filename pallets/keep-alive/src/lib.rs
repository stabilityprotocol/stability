#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::traits::OneSessionHandler;
use frame_support::traits::ValidatorSet;
use frame_support::traits::ValidatorSetWithIdentification;
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
		SigningTypes,
	},
};
use pallet_validator_set::StabilityValidatorSet;
use sp_application_crypto::RuntimeAppPublic;
use sp_core::crypto::KeyTypeId;
use sp_runtime::offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef};
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::{
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	Perbill, RuntimeDebug, Saturating,
};
use sp_staking::offence::{Kind, Offence, ReportOffence};
use sp_staking::SessionIndex;
use sp_std::prelude::*;

// #[cfg(test)]
// mod tests;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"imon");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::ecdsa::Signature as ECDSASignature;
	use sp_runtime::{
		app_crypto::{app_crypto, ecdsa},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(ecdsa, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ecdsa::Signature;
		type GenericPublic = sp_core::ecdsa::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<ECDSASignature as Verify>::Signer, ECDSASignature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::ecdsa::Signature;
		type GenericPublic = sp_core::ecdsa::Public;
	}
}

/// Payload used by this example crate to hold price
/// data required to submit a transaction.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct KeepAlivePayload<Public, BlockNumber> {
	block_number: BlockNumber,
	session_index: SessionIndex,
	public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for KeepAlivePayload<T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

/// A type for representing the validator id in a session.
pub type ValidatorId<T> = <<T as Config>::StbleValidatorSet as ValidatorSet<
	<T as frame_system::Config>::AccountId,
>>::ValidatorId;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_validator_set::StabilityValidatorSet;
	use sp_runtime::app_crypto::ecdsa;

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_validator_set::Config
	{
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<ecdsa::Public, ecdsa::Signature> + Decode + RuntimeAppPublic;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type StbleValidatorSet: StabilityValidatorSet<Self>
			+ ValidatorSetWithIdentification<Self::AccountId>;

		type ReportUnresponsiveness: ReportOffence<
			Self::AccountId,
			Self::AccountId,
			UnresponsivenessOffence<Self::AccountId>,
		>;

		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;

		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<Self::BlockNumber>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("pallet-keep-alive offchain_worker");

			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!(
				"Current block: {:?} (parent hash: {:?})",
				block_number,
				parent_hash
			);

			// check if heartbeat is needed
			const RECENTLY_SENT: () = ();
			let val = StorageValueRef::persistent(b"keep-alive::last_send");
			let res = val.mutate(
				|last_send: Result<Option<T::BlockNumber>, StorageRetrievalError>| {
					match last_send {
						// If we already have a value in storage and the block number is recent enough
						// we avoid sending another transaction at this time.
						Ok(Some(block)) if block_number < block + T::GracePeriod::get() => {
							Err(RECENTLY_SENT)
						}
						// In every other case we attempt to acquire the lock and send a transaction.
						_ => Ok(block_number),
					}
				},
			);

			match res {
				Ok(block_number) => {
					let signer = Signer::<T, T::AuthorityId>::all_accounts();

					if signer.can_sign() {
						log::debug!("Sending unsigned txn for block {:?}", block_number);

						// For this example we are going to send both signed and unsigned transactions
						// depending on the block number.
						// Usually it's enough to choose one or the other.
						let session_index = T::StbleValidatorSet::session_index();
						let res = Self::send_unsigned_for_all_accounts(block_number, session_index);
						if let Err(e) = res {
							log::error!("Error: {}", e);
						}
					} else {
						log::trace!(
							target: "runtime::keep-alive",
							"Skipping heartbeat at {:?}. Not a validator.",
							block_number,
						)
					}
				}
				// We are in the grace period, we should not send a transaction this time.
				Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => {
					log::trace!(
						target: "runtime::keep-alive",
						"Skipping heartbeat at {:?}. Recently sent.",
						block_number,
					)
				}
				// We wanted to send a transaction, but failed to write the block number (acquire a
				// lock). This indicates that another offchain worker that was running concurrently
				// most likely executed the same logic and succeeded at writing to storage.
				// Thus we don't really want to send the transaction, knowing that the other run
				// already did.
				Err(MutateStorageError::ConcurrentModification(_)) => {
					log::trace!(
						target: "runtime::keep-alive",
						"Skipping heartbeat at {:?}. Concurrent lock.",
						block_number,
					)
				}
			}
		}
	}

	/// A public part of the pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({0})]
		#[pallet::call_index(0)]
		pub fn submit_heartbeat_unsigned_with_signed_payload(
			origin: OriginFor<T>,
			keep_alive_payload: KeepAlivePayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			let msg_account_id = keep_alive_payload.public.clone().into_account();

			// check if the received heartbeat is valid.
			ensure!(
				T::StbleValidatorSet::is_approved_validator(&msg_account_id),
				Error::<T>::InvalidKey
			);

			// stores the received heartbeat in the storage.
			let current_session = T::StbleValidatorSet::session_index();
			ReceivedHeartbeats::<T>::insert(current_session, msg_account_id.clone(), true);

			Self::deposit_event(Event::HeartbeatSubmitted(
				msg_account_id,
				keep_alive_payload.block_number,
				keep_alive_payload.session_index,
			));

			// now increment the block number at which we expect next unsigned transaction.
			let current_block = <system::Pallet<T>>::block_number();
			<NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
			Ok(().into())
		}
	}

	/// Errors for the pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// Non existent public key.
		InvalidKey,
	}

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when a new heartbeat is submitted.
		HeartbeatSubmitted(T::AccountId, T::BlockNumber, u32 /* session_index */),
		AllGood,
		SomeOffline(Vec<T::AccountId>),
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::submit_heartbeat_unsigned_with_signed_payload {
				keep_alive_payload: ref payload,
				ref signature,
			} = call
			{
				// check if its from approved validator
				let public_key = SignedPayload::<T>::public(payload);
				let msg_account_id = public_key.into_account();
				let approved_vals = T::StbleValidatorSet::approved_validators();

				if !approved_vals.contains(&msg_account_id) {
					return InvalidTransaction::BadSigner.into();
				}

				// expensive, should be in the last position
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());

				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}

				Self::validate_transaction_parameters(&payload.block_number)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	/// Defines the block when next unsigned transaction will be accepted.
	///
	/// To prevent spam of unsigned (and unpaid!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// For each session index we stored the validators that have submitted a heartbeat.
	/// SessionIndex-> Validator-AccountId -> bool
	#[pallet::storage]
	#[pallet::getter(fn received_heartbeats)]
	pub(crate) type ReceivedHeartbeats<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		SessionIndex,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
	>;

	impl<T: Config> Pallet<T> {
		pub fn is_online(account_id: T::AccountId) -> bool {
			let current_session = T::StbleValidatorSet::session_index();
			Self::is_online_at_session(current_session, account_id)
		}

		pub fn is_online_at_session(session_index: SessionIndex, account_id: T::AccountId) -> bool {
			<ReceivedHeartbeats<T>>::contains_key(session_index, account_id)
		}

		pub fn online_validators_at_session(session_index: SessionIndex) -> Vec<T::AccountId> {
			<ReceivedHeartbeats<T>>::iter_prefix(session_index)
				.map(|(account_id, _)| account_id)
				.collect::<Vec<_>>()
		}
		pub fn online_validators() -> Vec<T::AccountId> {
			let current_session = T::StbleValidatorSet::session_index();
			Self::online_validators_at_session(current_session)
		}

		/// A helper function to fetch the price, sign payload and send an unsigned transaction
		fn send_unsigned_for_all_accounts(
			block_number: T::BlockNumber,
			session_index: u32,
		) -> Result<(), &'static str> {
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if next_unsigned_at > block_number {
				return Err("Too early to send unsigned transaction");
			}

			// Sign using all accounts that are capable of signing transactions.
			let transaction_results = Signer::<T, T::AuthorityId>::all_accounts()
				.send_unsigned_transaction(
					|account| KeepAlivePayload {
						block_number,
						session_index,
						public: account.public.clone(),
					},
					|payload, signature| Call::submit_heartbeat_unsigned_with_signed_payload {
						keep_alive_payload: payload,
						signature,
					},
				);

			for (_account_id, result) in transaction_results.into_iter() {
				if result.is_err() {
					return Err("Unable to submit transaction");
				}
			}

			Ok(())
		}

		fn validate_transaction_parameters(block_number: &T::BlockNumber) -> TransactionValidity {
			// Now let's check if the transaction has any chance to succeed.
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if &next_unsigned_at > block_number {
				return InvalidTransaction::Stale.into();
			}
			// Let's make sure to reject transactions from the future.
			let current_block = <system::Pallet<T>>::block_number();
			if &current_block < block_number {
				return InvalidTransaction::Future.into();
			}

			ValidTransaction::with_tag_prefix("KeepAliveOffchainWorker")
				.priority(T::UnsignedPriority::get())
				// This transaction does not require anything else to go before into the pool.
				// In theory we could require `previous_unsigned_at` transaction to go first,
				// but it's not necessary in our case.
				//.and_requires()
				// We set the `provides` tag to be the same as `next_unsigned_at`. This makes
				// sure only one transaction produced after `next_unsigned_at` will ever
				// get to the transaction pool and will end up in the block.
				// We can still have multiple transactions compete for the same "spot",
				// and the one with higher priority will replace other one in the pool.
				.and_provides(next_unsigned_at)
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				.longevity(5)
				// It's fine to propagate that transaction to other peers, which means it can be
				// created even by nodes that don't produce blocks.
				// Note that sometimes it's better to keep it for yourself (if you are the block
				// producer), since for instance in some schemes others may copy your solution and
				// claim a reward.
				.propagate(true)
				.build()
		}
	}
}

impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = T::AuthorityId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = T::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(_validators: I) {}

	fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _queued_validators: I) {}

	fn on_before_session_ending() {
		let session_index = T::StbleValidatorSet::session_index();
		let approved_validators = T::StbleValidatorSet::approved_validators();

		let offenders = approved_validators
			.iter()
			.filter_map(|validator| {
				if !Self::is_online_at_session(session_index, validator.clone()) {
					Some(validator.clone())
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		// Remove all received heartbeats and number of authored blocks from the
		// current session, they have already been processed and won't be needed anymore.
		let _removed_heartbeats =
			ReceivedHeartbeats::<T>::clear_prefix(session_index, u32::MAX, None);

		if offenders.is_empty() {
			Self::deposit_event(Event::<T>::AllGood);
		} else {
			Self::deposit_event(Event::<T>::SomeOffline(offenders.clone()));

			let validator_set_count = approved_validators.len() as u32;
			let offence = UnresponsivenessOffence {
				session_index,
				validator_set_count,
				offenders,
			};
			if let Err(e) = T::ReportUnresponsiveness::report_offence(vec![], offence) {
				sp_runtime::print(e);
			}
		}
	}

	fn on_disabled(_i: u32) {}
}

/// An offence that is filed if a validator didn't send a heartbeat message.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub struct UnresponsivenessOffence<Offender> {
	/// The current session index in which we report the unresponsive validators.
	///
	/// It acts as a time measure for unresponsiveness reports and effectively will always point
	/// at the end of the session.
	pub session_index: SessionIndex,
	/// The size of the approved validator set.
	pub validator_set_count: u32,
	/// Authorities that were unresponsive during the current era.
	pub offenders: Vec<Offender>,
}

impl<Offender: Clone> Offence<Offender> for UnresponsivenessOffence<Offender> {
	const ID: Kind = *b"k-keep-alive:off";
	type TimeSlot = SessionIndex;

	fn offenders(&self) -> Vec<Offender> {
		self.offenders.clone()
	}

	fn session_index(&self) -> SessionIndex {
		self.session_index
	}

	fn validator_set_count(&self) -> u32 {
		self.validator_set_count
	}

	fn time_slot(&self) -> Self::TimeSlot {
		self.session_index
	}

	fn slash_fraction(&self, offenders: u32) -> Perbill {
		let validator_set_count = self.validator_set_count();
		// the formula is min((3 * (k - (n / 10 + 1))) / n, 1) * 0.07
		// basically, 10% can be offline with no slash, but after that, it linearly climbs up to 7%
		// when 13/30 are offline (around 5% when 1/3 are offline).
		if let Some(threshold) = offenders.checked_sub(validator_set_count / 10 + 1) {
			let x = Perbill::from_rational(3 * threshold, validator_set_count);
			x.saturating_mul(Perbill::from_percent(7))
		} else {
			Perbill::default()
		}
	}
}
