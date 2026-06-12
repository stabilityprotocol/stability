// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::DispatchInfo;
use frame_support::pallet_prelude::{TransactionSource, Weight};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AsSystemOriginSigner, DispatchInfoOf, Dispatchable, One, PostDispatchInfoOf,
		TransactionExtension, ValidateResult,
	},
	transaction_validity::{
		InvalidTransaction, TransactionLongevity, TransactionValidityError, ValidTransaction,
	},
	DispatchResult,
};
use sp_std::vec;

/// Nonce check and increment to give replay protection for transactions.
///
/// # Transaction Validity
///
/// This extension affects `requires` and `provides` tags of validity, but DOES NOT
/// set the `priority` field. Make sure that AT LEAST one of the transaction extensions sets
/// some kind of priority upon validating transactions.
///
/// Same version as in https://github.com/paritytech/polkadot-sdk/blob/stable2512/substrate/frame/system/src/extensions/check_nonce.rs
/// But removed the check on account already created (providers/sufficients == 0 bail-out),
/// for making it compatible with the runtime/DNT.
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct StbleCheckNonce<T: frame_system::Config>(#[codec(compact)] pub T::Nonce);

impl<T: frame_system::Config> StbleCheckNonce<T> {
	/// utility constructor. Used only in client/factory code.
	pub fn from(nonce: T::Nonce) -> Self {
		Self(nonce)
	}
}

impl<T: frame_system::Config> core::fmt::Debug for StbleCheckNonce<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "CheckNonce({})", self.0)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

/// Operation to perform from `validate` to `prepare` in [`StbleCheckNonce`].
pub enum Val<T: frame_system::Config> {
	/// Account to check the nonce for.
	CheckNonce(T::AccountId),
	/// The origin was not signed: nothing to do.
	NoCheck,
}

impl<T: frame_system::Config> TransactionExtension<T::RuntimeCall> for StbleCheckNonce<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
	<T::RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
{
	const IDENTIFIER: &'static str = "CheckNonce";
	type Implicit = ();
	type Val = Val<T>;
	type Pre = ();

	fn weight(&self, _call: &T::RuntimeCall) -> Weight {
		<T::ExtensionsWeightInfo as frame_system::ExtensionsWeightInfo>::check_nonce()
	}

	fn validate(
		&self,
		origin: <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
		_call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl Encode,
		_source: TransactionSource,
	) -> ValidateResult<Self::Val, T::RuntimeCall> {
		let Some(who) = origin.as_system_origin_signer() else {
			// The extension only applies to signed origins; pass everything else through.
			return Ok((Default::default(), Val::NoCheck, origin));
		};

		let account = frame_system::Account::<T>::get(who);
		// Contrary to upstream `CheckNonce`, accounts without providers nor sufficients
		// are accepted here: fees are paid in ERC20 tokens (DNT), so the account may not
		// have been "created" from frame-system's point of view.
		if self.0 < account.nonce {
			return Err(InvalidTransaction::Stale.into());
		}

		let provides = vec![Encode::encode(&(who, self.0))];
		let requires = if account.nonce < self.0 {
			vec![Encode::encode(&(who, self.0 - One::one()))]
		} else {
			vec![]
		};

		let validity = ValidTransaction {
			priority: 0,
			requires,
			provides,
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		};

		Ok((validity, Val::CheckNonce(who.clone()), origin))
	}

	fn prepare(
		self,
		val: Self::Val,
		_origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
		_call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let who = match val {
			Val::CheckNonce(who) => who,
			Val::NoCheck => return Ok(()),
		};

		let mut account = frame_system::Account::<T>::get(&who);
		if self.0 != account.nonce {
			return Err(if self.0 < account.nonce {
				InvalidTransaction::Stale
			} else {
				InvalidTransaction::Future
			}
			.into());
		}
		account.nonce += T::Nonce::one();
		frame_system::Account::<T>::insert(&who, account);
		Ok(())
	}

	fn post_dispatch_details(
		_pre: Self::Pre,
		_info: &DispatchInfo,
		_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		_len: usize,
		_result: &DispatchResult,
	) -> Result<Weight, TransactionValidityError> {
		Ok(Weight::zero())
	}
}
