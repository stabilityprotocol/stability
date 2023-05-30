// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! <!-- markdown-link-check-disable -->
//! # Offchain Worker Example Pallet
//!
//! The Offchain Worker Example: A simple pallet demonstrating
//! concepts, APIs and structures common to most offchain workers.
//!
//! Run `cargo doc --package pallet-example-offchain-worker --open` to view this module's
//! documentation.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! **This pallet serves as an example showcasing Substrate off-chain worker and is not meant to
//! be used in production.**
//!
//! ## Overview
//!
//! In this example we are going to build a very simplistic, naive and definitely NOT
//! production-ready oracle for BTC/USD price.
//! Offchain Worker (OCW) will be triggered after every block, fetch the current price
//! and prepare either signed or unsigned transaction to feed the result back on chain.
//! The on-chain logic will simply aggregate the results and store last `64` values to compute
//! the average price.
//! Additional logic in OCW is put in place to prevent spamming the network with both signed
//! and unsigned transactions, and custom `UnsignedValidator` makes sure that there is only
//! one unsigned transaction floating in the network.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::traits::ValidatorSetWithIdentification;
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
};
use lite_json::json::JsonValue;
use sp_core::crypto::KeyTypeId;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::{
	offchain::{
		http,
		storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		Duration,
	},
	traits::Zero,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::vec::Vec;

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
	public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for KeepAlivePayload<T::Public, T::BlockNumber> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_validator_set::StabilityValidatorSet;

	/// This pallet's configuration trait
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type ValidatorSet: ValidatorSetWithIdentification<Self::AccountId>;

		type StbleValidatorSet: StabilityValidatorSet<Self::AccountId>;

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
		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// successfully imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: T::BlockNumber) {
			// Note that having logs compiled to WASM may cause the size of the blob to increase
			// significantly. You can use `RuntimeDebug` custom derive to hide details of the types
			// in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
			// all logging and thus, remove any logging from the WASM.
			log::info!("pallet-keep-alive offchain_worker");

			// Since off-chain workers are just part of the runtime code, they have direct access
			// to the storage and other included pallets.
			//
			// We can easily import `frame_system` and retrieve a block hash of the parent block.
			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!(
				"Current block: {:?} (parent hash: {:?})",
				block_number,
				parent_hash
			);

			let signer = Signer::<T, T::AuthorityId>::all_accounts();

			if signer.can_sign() {
				log::debug!("Sending unsigned txn for block {:?}", block_number);

				// For this example we are going to send both signed and unsigned transactions
				// depending on the block number.
				// Usually it's enough to choose one or the other.
				let res = Self::send_unsigned_for_all_accounts(block_number);
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
			// now increment the block number at which we expect next unsigned transaction.
			let current_block = <system::Pallet<T>>::block_number();
			<NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
			Ok(().into())
		}
	}

	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when a new heartbeat is submitted.
		HeartbeatSubmitted(T::AccountId, T::BlockNumber),
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
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());

				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}

				// check if its from approved validator
				let public_key = SignedPayload::<T>::public(payload);
				let msg_account_id = public_key.into_account();
				let approved_vals = T::StbleValidatorSet::approved_validators();

				if !approved_vals.contains(&msg_account_id) {
					return InvalidTransaction::BadSigner.into();
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

	impl<T: Config> Pallet<T> {
		/// A helper function to fetch the price, sign payload and send an unsigned transaction
		fn send_unsigned_for_all_accounts(
			block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			let next_unsigned_at = <NextUnsignedAt<T>>::get();
			if next_unsigned_at > block_number {
				return Err("Too early to send unsigned transaction");
			}

			// -- Sign using all accounts
			let transaction_results = Signer::<T, T::AuthorityId>::all_accounts()
				.send_unsigned_transaction(
					|account| {
						Self::deposit_event(Event::HeartbeatSubmitted(
							account.clone().public.into_account(),
							block_number,
						));
						KeepAlivePayload {
							block_number,
							public: account.public.clone(),
						}
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
