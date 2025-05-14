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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use core::marker::PhantomData;

	use frame_support::traits::tokens::{
		DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence,
	};
	use frame_support::traits::{fungible, Imbalance, SameOrOther, TryDrop};
	use frame_support::{
		pallet_prelude::MaybeSerializeDeserialize,
		traits::{
			tokens::{currency::Currency, Balance},
			ExistenceRequirement, SignedImbalance, WithdrawReasons,
		},
	};
	use pallet_user_fee_selector::UserFeeTokenController;
	use parity_scale_codec::MaxEncodedLen;
	use sp_runtime::{DispatchError, DispatchResult, FixedPointOperand, RuntimeDebug};
	use sp_std::fmt::Debug;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type AccountIdMapping: AccountIdMapping<Self>;
		type UserFeeTokenController: pallet_user_fee_selector::UserFeeTokenController;
	}

	pub trait AccountIdMapping<T: Config> {
		fn into_evm_address(account: &T::AccountId) -> sp_core::H160;
	}

	#[must_use]
	#[derive(RuntimeDebug, PartialEq, Eq, Clone)]
	pub struct NeutralImbalance<
		T: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand,
	>(T);
	impl<T: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand>
		NeutralImbalance<T>
	{
		/// Create a new positive imbalance from a balance.
		pub fn new(amount: T) -> Self {
			NeutralImbalance(amount)
		}
	}

	impl<T: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand> TryDrop
		for NeutralImbalance<T>
	{
		fn try_drop(self) -> Result<(), Self> {
			Ok(())
		}
	}

	impl<T: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand> Default
		for NeutralImbalance<T>
	{
		fn default() -> Self {
			Self::new(<T as Default>::default())
		}
	}

	impl<T: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand>
		Imbalance<T> for NeutralImbalance<T>
	{
		type Opposite = Self;

		fn zero() -> Self {
			Self::default()
		}

		fn drop_zero(self) -> Result<(), Self> {
			Self::try_drop(self)
		}

		fn split(self, amount: T) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0.saturating_sub(first);
			(Self::new(first), Self::new(second))
		}

		fn merge(self, other: Self) -> Self {
			Self::new(self.0.saturating_add(other.0))
		}

		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
		}

		fn offset(self, _other: Self::Opposite) -> SameOrOther<Self, Self> {
			SameOrOther::Same(Self::default())
		}

		fn peek(&self) -> T {
			self.0.clone()
		}

		fn extract(&mut self, _balance: T) -> Self {
			NeutralImbalance(self.0)
		}
	}

	impl<T: Config> Currency<T::AccountId> for Pallet<T> {
		type Balance = u128;
		type PositiveImbalance = NeutralImbalance<Self::Balance>;
		type NegativeImbalance = NeutralImbalance<Self::Balance>;

		/// The combined balance of `who`.
		fn total_balance(who: &T::AccountId) -> Self::Balance {
			let evm_address = T::AccountIdMapping::into_evm_address(who);
			let actual_balance =
				<T::UserFeeTokenController as UserFeeTokenController>::balance_of(evm_address);

			let maximum_balance = sp_core::U256::from(u128::MAX);

			if maximum_balance < actual_balance {
				u128::MAX
			} else {
				actual_balance.as_u128()
			}
		}

		/// Same result as `slash(who, value)` (but without the side-effects) assuming there are no
		/// balance changes in the meantime and only the reserved balance is not taken into account.
		fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
			Self::ensure_can_withdraw(
				who,
				value,
				WithdrawReasons::TRANSACTION_PAYMENT,
				Self::Balance::from(0u128),
			)
			.is_ok()
		}

		/// The total amount of issuance in the system.
		fn total_issuance() -> Self::Balance {
			0u128
		}

		// Minimum balance an address could have to exist.
		fn minimum_balance() -> Self::Balance {
			0u128
		}

		fn burn(_amount: Self::Balance) -> Self::PositiveImbalance {
			Self::PositiveImbalance::new(Self::Balance::from(0u128))
		}

		fn issue(_amount: Self::Balance) -> Self::NegativeImbalance {
			Self::NegativeImbalance::new(Self::Balance::from(0u128))
		}

		fn free_balance(account: &T::AccountId) -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_balance(account)
		}

		fn ensure_can_withdraw(
			who: &T::AccountId,
			amount: Self::Balance,
			_reasons: frame_support::traits::WithdrawReasons,
			_new_balance: Self::Balance,
		) -> Result<(), sp_runtime::DispatchError> {
			if <Self as Currency<T::AccountId>>::total_balance(who) >= amount {
				Ok(())
			} else {
				Err(DispatchError::Other("Not enough balance"))
			}
		}

		fn transfer(
			source: &T::AccountId,
			dest: &T::AccountId,
			value: Self::Balance,
			_existence_requirement: ExistenceRequirement,
		) -> DispatchResult {
			if value == 0u128 {
				Ok(())
			} else {
				let source_evm_address = T::AccountIdMapping::into_evm_address(source);
				let dest_evm_address = T::AccountIdMapping::into_evm_address(dest);

				<T::UserFeeTokenController as UserFeeTokenController>::transfer(
					source_evm_address,
					dest_evm_address,
					value.into(),
				)
				.map_err(|_| DispatchError::Other("Transfer failed"))?;

				Ok(())
			}
		}

		fn slash(
			_who: &T::AccountId,
			_value: Self::Balance,
		) -> (Self::NegativeImbalance, Self::Balance) {
			(Self::PositiveImbalance::default(), 0u128)
		}

		fn deposit_into_existing(
			_who: &T::AccountId,
			_value: Self::Balance,
		) -> Result<Self::PositiveImbalance, DispatchError> {
			Err(DispatchError::Other(
				"Deposits are not allowed in this pallet",
			))
		}

		fn resolve_into_existing(
			_who: &T::AccountId,
			value: Self::NegativeImbalance,
		) -> Result<(), Self::NegativeImbalance> {
			Err(value)
		}

		fn deposit_creating(_who: &T::AccountId, _value: Self::Balance) -> Self::PositiveImbalance {
			Self::PositiveImbalance::default()
		}

		fn resolve_creating(_who: &T::AccountId, _value: Self::NegativeImbalance) {}

		fn withdraw(
			_who: &T::AccountId,
			_value: Self::Balance,
			_reasons: WithdrawReasons,
			_liveness: ExistenceRequirement,
		) -> Result<Self::NegativeImbalance, DispatchError> {
			Err(DispatchError::Other(
				"Withdrawals are not allowed in this pallet",
			))
		}

		fn settle(
			_who: &T::AccountId,
			value: Self::PositiveImbalance,
			_reasons: WithdrawReasons,
			_liveness: ExistenceRequirement,
		) -> Result<(), Self::PositiveImbalance> {
			Err(value)
		}

		fn make_free_balance_be(
			_who: &T::AccountId,
			balance: Self::Balance,
		) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
			SignedImbalance::Positive(NeutralImbalance::new(balance))
		}
	}

	impl<T: Config> fungible::Inspect<T::AccountId> for Pallet<T> {
		/// Scalar type for representing balance of an account.
		type Balance = <Self as Currency<T::AccountId>>::Balance;

		/// The total amount of issuance in the system.
		fn total_issuance() -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_issuance()
		}

		/// The total amount of issuance in the system excluding those which are controlled by the
		/// system.
		fn active_issuance() -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_issuance()
		}

		fn total_balance(who: &T::AccountId) -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_balance(who)
		}

		/// The minimum balance any single account may have.
		fn minimum_balance() -> Self::Balance {
			<Self as Currency<T::AccountId>>::minimum_balance()
		}

		/// Get the balance of `who`.
		fn balance(who: &T::AccountId) -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_balance(who)
		}

		/// Get the maximum amount that `who` can withdraw/transfer successfully.
		fn reducible_balance(
			who: &T::AccountId,
			_preservation: Preservation,
			_force: Fortitude,
		) -> Self::Balance {
			<Self as Currency<T::AccountId>>::total_balance(who)
		}

		/// Returns `true` if the balance of `who` may be increased by `amount`.
		///
		/// - `who`: The account of which the balance should be increased by `amount`.
		/// - `amount`: How much should the balance be increased?
		/// - `mint`: Will `amount` be minted to deposit it into `account`?
		fn can_deposit(
			_who: &T::AccountId,
			_amount: Self::Balance,
			_provenance: Provenance,
		) -> DepositConsequence {
			DepositConsequence::UnknownAsset
		}

		/// Returns `Failed` if the balance of `who` may not be decreased by `amount`, otherwise
		/// the consequence.
		fn can_withdraw(
			_who: &T::AccountId,
			_amount: Self::Balance,
		) -> WithdrawConsequence<Self::Balance> {
			Self::ensure_can_withdraw(_who, _amount, WithdrawReasons::all(), 0u128)
				.map(|_| WithdrawConsequence::Success)
				.unwrap_or(WithdrawConsequence::BalanceLow)
		}
	}

	impl<T: Config> fungible::Unbalanced<T::AccountId> for Pallet<T> {
		fn handle_dust(_dust: fungible::Dust<T::AccountId, Self>) {
			()
		}
		fn write_balance(
			_who: &T::AccountId,
			_amount: Self::Balance,
		) -> Result<Option<Self::Balance>, DispatchError> {
			Ok(Some(0u128))
		}

		fn set_total_issuance(_amount: Self::Balance) {
			()
		}

		fn deactivate(_amount: Self::Balance) {
			()
		}

		fn reactivate(_amount: Self::Balance) {
			()
		}
	}

	impl<T: Config> fungible::Balanced<T::AccountId> for Pallet<T> {
		type OnDropCredit = fungible::DecreaseIssuance<T::AccountId, Self>;
		type OnDropDebt = fungible::IncreaseIssuance<T::AccountId, Self>;
	}
}
