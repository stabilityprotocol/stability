#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use core::marker::PhantomData;

	use codec::MaxEncodedLen;
	use frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence, Preservation, Provenance, Fortitude};
	use frame_support::traits::{Imbalance, SameOrOther, TryDrop};
	use frame_support::RuntimeDebug;
	use frame_support::{
		pallet_prelude::MaybeSerializeDeserialize,
		traits::{
			tokens::{currency::Currency, fungible::Inspect, Balance},
			ExistenceRequirement, SignedImbalance, WithdrawReasons,
		},
	};
	use pallet_user_fee_selector::UserFeeTokenController;
	use sp_runtime::{DispatchError, DispatchResult, FixedPointOperand};
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
			let second = self.0 - first;
			(Self::new(first), Self::new(second))
		}

		fn merge(self, other: Self) -> Self {
			Self::new(self.0 + other.0)
		}

		fn subsume(&mut self, other: Self) {
			self.0 += other.0;
		}

		fn offset(self, _other: Self::Opposite) -> SameOrOther<Self, Self> {
			SameOrOther::Same(Self::default())
		}

		fn peek(&self) -> T {
			self.0.clone()
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
			_source: &T::AccountId,
			_dest: &T::AccountId,
			value: Self::Balance,
			_existence_requirement: ExistenceRequirement,
		) -> DispatchResult {
			if value == 0u128 {
				Ok(())
			} else {
				Err(DispatchError::Other(
					"Transfer is not supported in this pallet",
				))
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
			_balance: Self::Balance,
		) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
			panic!("make_free_balance_be is not allowed in this pallet")
		}
	}

	impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
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
		fn reducible_balance(who: &T::AccountId, _preservation: Preservation, _force: Fortitude) -> Self::Balance {
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
}
