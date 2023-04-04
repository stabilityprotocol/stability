#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use evm::{
	backend::Backend as BackendT,
	executor::stack::{Accessed, StackExecutor, StackState as StackStateT, StackSubstateMetadata},
	ExitError, ExitReason, Handler, Transfer,
};
use fp_evm::{CallInfo, CreateInfo, ExecutionInfo, Log, Vicinity};
use frame_support::sp_runtime::traits::UniqueSaturatedInto;
use frame_support::{
	traits::{Currency, ExistenceRequirement},
	weights::Weight,
};
use pallet_evm::Pallet;
use pallet_evm::{
	AccountCodes, AccountStorages, AddressMapping, BalanceOf, BlockHashMapping, Config, Error,
	Event, FeeCalculator, PrecompileSet, Runner as RunnerT, RunnerError,
};
use pallet_user_fee_selector::UserFeeTokenController;
use precompile_utils::prelude::keccak256;
use sp_core::{Get, H160, H256, U256};
use sp_std::{
	boxed::Box,
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	mem, vec,
	vec::Vec,
};
use stbl_tools;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const TRANSACTION_FEE_TOPIC: [u8; 32] =
	keccak256!("TransactionFee(address,uint256,address,uint256,address,uint256)");

#[cfg(feature = "forbid-evm-reentrancy")]
environmental::thread_local_impl!(static IN_EVM: environmental::RefCell<bool> = environmental::RefCell::new(false));

#[derive(Default)]
pub struct Runner<T: Config, FC: OnChargeDecentralizedNativeTokenFee, U: UserFeeTokenController> {
	_marker: PhantomData<(T, FC, U)>,
}

impl<T: Config, FC: OnChargeDecentralizedNativeTokenFee, U: UserFeeTokenController> Runner<T, FC, U>
where
	BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
	#[allow(clippy::let_and_return)]
	/// Execute an already validated EVM operation.
	fn execute<'config, 'precompiles, F, R>(
		source: H160,
		target: Option<H160>,
		value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		config: &'config evm::Config,
		precompiles: &'precompiles T::PrecompilesType,
		is_transactional: bool,
		f: F,
	) -> Result<ExecutionInfo<R>, RunnerError<Error<T>>>
	where
		F: FnOnce(
			&mut StackExecutor<
				'config,
				'precompiles,
				SubstrateStackState<'_, 'config, T>,
				T::PrecompilesType,
			>,
		) -> (ExitReason, R),
	{
		let (base_fee, weight) = T::FeeCalculator::min_gas_price();

		#[cfg(feature = "forbid-evm-reentrancy")]
		if IN_EVM.with(|in_evm| in_evm.replace(true)) {
			return Err(RunnerError {
				error: Error::<T>::Reentrancy,
				weight,
			});
		}

		let res = Self::execute_inner(
			source,
			target,
			value,
			gas_limit,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			config,
			precompiles,
			is_transactional,
			f,
			base_fee,
			weight,
		);

		// Set IN_EVM to false
		// We should make sure that this line is executed whatever the execution path.
		#[cfg(feature = "forbid-evm-reentrancy")]
		let _ = IN_EVM.with(|in_evm| in_evm.take());

		res
	}

	// Execute an already validated EVM operation.
	fn execute_inner<'config, 'precompiles, F, R>(
		source: H160,
		dapp: Option<H160>,
		value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		config: &'config evm::Config,
		precompiles: &'precompiles T::PrecompilesType,
		is_transactional: bool,
		f: F,
		base_fee: U256,
		weight: Weight,
	) -> Result<ExecutionInfo<R>, RunnerError<Error<T>>>
	where
		F: FnOnce(
			&mut StackExecutor<
				'config,
				'precompiles,
				SubstrateStackState<'_, 'config, T>,
				T::PrecompilesType,
			>,
		) -> (ExitReason, R),
	{
		// EIP-3607: https://eips.ethereum.org/EIPS/eip-3607
		// Do not allow transactions for which `tx.sender` has any code deployed.
		//
		// We extend the principle of this EIP to also prevent `tx.sender` to be the address
		// of a precompile. While mainnet Ethereum currently only has stateless precompiles,
		// projects using Frontier can have stateful precompiles that can manage funds or
		// which calls other contracts that expects this precompile address to be trustworthy.
		if !<AccountCodes<T>>::get(source).is_empty() || precompiles.is_precompile(source) {
			return Err(RunnerError {
				error: Error::<T>::TransactionMustComeFromEOA,
				weight,
			});
		}

		let (total_fee_per_gas, _actual_priority_fee_per_gas) =
			match (max_fee_per_gas, max_priority_fee_per_gas, is_transactional) {
				// Zero max_fee_per_gas for validated transactional calls exist in XCM -> EVM
				// because fees are already withdrawn in the xcm-executor.
				(Some(max_fee), _, true) if max_fee.is_zero() => (U256::zero(), U256::zero()),
				// With no tip, we pay exactly the base_fee
				(Some(_), None, _) => (base_fee, U256::zero()),
				// With tip, we include as much of the tip on top of base_fee that we can, never
				// exceeding max_fee_per_gas
				(Some(max_fee_per_gas), Some(max_priority_fee_per_gas), _) => {
					let actual_priority_fee_per_gas = max_fee_per_gas
						.saturating_sub(base_fee)
						.min(max_priority_fee_per_gas);
					(
						base_fee.saturating_add(actual_priority_fee_per_gas),
						actual_priority_fee_per_gas,
					)
				}
				// Gas price check is skipped for non-transactional calls that don't
				// define a `max_fee_per_gas` input.
				(None, _, false) => (Default::default(), U256::zero()),
				// Unreachable, previously validated. Handle gracefully.
				_ => {
					return Err(RunnerError {
						error: Error::<T>::GasPriceTooLow,
						weight,
					})
				}
			};

		// After eip-1559 we make sure the account can pay both the evm execution and priority fees.
		let total_fee =
			total_fee_per_gas
				.checked_mul(U256::from(gas_limit))
				.ok_or(RunnerError {
					error: Error::<T>::FeeOverflow,
					weight,
				})?;

		let token = FC::get_transaction_fee_token(source);
		let validator = <pallet_evm::Pallet<T>>::find_author();
		let vault = FC::get_fee_vault();

		let conversion_rate = FC::get_transaction_conversion_rate(validator, token);

		// Deduct fee from the `source` account. Returns `None` if `total_fee` is Zero.
		FC::withdraw_fee(source, token, conversion_rate, total_fee).map_err(|_| RunnerError {
			error: Error::<T>::FeeOverflow,
			weight,
		})?;

		// Execute the EVM call.
		let vicinity = Vicinity {
			gas_price: base_fee,
			origin: source,
		};

		let metadata = StackSubstateMetadata::new(gas_limit, config);
		let state = SubstrateStackState::new(&vicinity, metadata);
		let mut executor = StackExecutor::new_with_precompiles(state, config, precompiles);

		let (reason, retv) = f(&mut executor);

		// Post execution.
		let used_gas = U256::from(executor.used_gas());
		let actual_fee = executor.fee(total_fee_per_gas);

		log::debug!(
			target: "evm",
			"Execution {:?} [source: {:?}, value: {}, gas_limit: {}, token: {}, validator: {}, actual_fee: {}, is_transactional: {}]",
			reason,
			source,
			value,
			gas_limit,
			token,
			validator,
			actual_fee,
			is_transactional
		);

		FC::correct_fee(source, token, conversion_rate, total_fee, actual_fee).map_err(|_| {
			RunnerError {
				error: Error::<T>::FeeOverflow,
				weight,
			}
		})?;

		let (validator_fee, dapp_fee) =
			FC::pay_fees(token, conversion_rate, actual_fee, validator, dapp).map_err(|_| {
				RunnerError {
					error: Error::<T>::FeeOverflow,
					weight,
				}
			})?;

		executor
			.log(
				vault,
				sp_std::vec![TRANSACTION_FEE_TOPIC.into()],
				stbl_tools::eth::args_to_bytes(sp_std::vec![
					token.into(),
					stbl_tools::misc::u256_to_h256(validator_fee + dapp_fee),
					validator.into(),
					stbl_tools::misc::u256_to_h256(validator_fee),
					match dapp {
						None => H160::zero().into(),
						Some(dapp) => dapp.into(),
					},
					stbl_tools::misc::u256_to_h256(dapp_fee),
				]),
			)
			.map_err(|_| RunnerError {
				error: Error::<T>::FeeOverflow,
				weight,
			})?;

		let state = executor.into_state();

		for address in state.substate.deletes {
			log::debug!(
				target: "evm",
				"Deleting account at {:?}",
				address
			);
			Pallet::<T>::remove_account(&address)
		}

		for log in &state.substate.logs {
			log::trace!(
				target: "evm",
				"Inserting log for {:?}, topics ({}) {:?}, data ({}): {:?}]",
				log.address,
				log.topics.len(),
				log.topics,
				log.data.len(),
				log.data
			);
			{
				let event = Event::<T>::Log {
					log: Log {
						address: log.address,
						topics: log.topics.clone(),
						data: log.data.clone(),
					},
				};
				let event = <<T as Config>::RuntimeEvent as From<Event<T>>>::from(event);
				let event = <<T as Config>::RuntimeEvent as Into<
					<T as frame_system::Config>::RuntimeEvent,
				>>::into(event);
				<frame_system::Pallet<T>>::deposit_event(event)
			};
		}

		Ok(ExecutionInfo {
			value: retv,
			exit_reason: reason,
			used_gas,
			logs: state.substate.logs,
		})
	}
}

impl<T: Config, FC: OnChargeDecentralizedNativeTokenFee, U: UserFeeTokenController> RunnerT<T>
	for Runner<T, FC, U>
where
	BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
	type Error = Error<T>;

	fn validate(
		source: H160,
		target: Option<H160>,
		input: Vec<u8>,
		_value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
		is_transactional: bool,
		evm_config: &evm::Config,
	) -> Result<(), RunnerError<Self::Error>> {
		// we force the value to be zero because we don't support value transfer in EVM
		let value = U256::from(0);
		let (base_fee, mut weight) = T::FeeCalculator::min_gas_price();
		let (mut source_account, inner_weight) = Pallet::<T>::account_basic(&source);

		// TODO: REMOVE THIS LINES ONCE WE UPDATE THE BALANCES PALLET
		// Update the balance with the actual one.
		source_account.balance = U::balance_of(source);
		// END OF TODO
		weight = weight.saturating_add(inner_weight);

		let _ = fp_evm::CheckEvmTransaction::<Self::Error>::new(
			fp_evm::CheckEvmTransactionConfig {
				evm_config,
				block_gas_limit: T::BlockGasLimit::get(),
				base_fee,
				chain_id: T::ChainId::get(),
				is_transactional,
			},
			fp_evm::CheckEvmTransactionInput {
				chain_id: Some(T::ChainId::get()),
				to: target,
				input,
				nonce: nonce.unwrap_or(source_account.nonce),
				gas_limit: gas_limit.into(),
				gas_price: None,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				value,
				access_list,
			},
		)
		.validate_in_block_for(&source_account)
		.and_then(|v| v.with_base_fee())
		.and_then(|v| v.with_balance_for(&source_account))
		.map_err(|error| RunnerError { error, weight })?;
		Ok(())
	}

	fn call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		_value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
		is_transactional: bool,
		validate: bool,
		config: &evm::Config,
	) -> Result<CallInfo, RunnerError<Self::Error>> {
		// we force the value to be zero because we don't support value transfer in EVM
		let value = U256::from(0);
		if validate {
			Self::validate(
				source,
				Some(target),
				input.clone(),
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.clone(),
				is_transactional,
				config,
			)?;
		}
		// input length greater than 2 means that we are calling a contract
		// where only the first two bytes are just the 0x prefix
		if input.len() > 0 {
			let precompiles = T::PrecompilesValue::get();
			Self::execute(
				source,
				Some(target),
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				config,
				&precompiles,
				is_transactional,
				|executor| {
					executor.transact_call(source, target, value, input, gas_limit, access_list)
				},
			)
		} else {
			// input less than 2 (0x) means that we are doing a regular ETH transfer
			// we get the user fee token address from the source account
			let user_token_address = U::get_user_fee_token(source);
			// substract the value from the user
			let transfer_value = stbl_tools::misc::u256_to_h256(_value);

			Self::call(
				source,
				user_token_address,
				stbl_tools::eth::generate_calldata(
					"transfer(address,uint256)",
					&vec![target.into(), transfer_value],
				),
				0.into(),
				350_000_u64, // TODO: look for how to estimate the gas
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.clone(),
				is_transactional,
				validate,
				config,
			)
		}
	}

	fn create(
		source: H160,
		init: Vec<u8>,
		_value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
		is_transactional: bool,
		validate: bool,
		config: &evm::Config,
	) -> Result<CreateInfo, RunnerError<Self::Error>> {
		// we force the value to be zero because we don't support value transfer in EVM
		let value = U256::from(0);
		if validate {
			Self::validate(
				source,
				None,
				init.clone(),
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.clone(),
				is_transactional,
				config,
			)?;
		}
		let precompiles = T::PrecompilesValue::get();
		Self::execute(
			source,
			None,
			value,
			gas_limit,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			config,
			&precompiles,
			is_transactional,
			|executor| {
				let address = executor.create_address(evm::CreateScheme::Legacy { caller: source });
				let (reason, _) =
					executor.transact_create(source, value, init, gas_limit, access_list);
				(reason, address)
			},
		)
	}

	fn create2(
		source: H160,
		init: Vec<u8>,
		salt: H256,
		_value: U256,
		gas_limit: u64,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
		access_list: Vec<(H160, Vec<H256>)>,
		is_transactional: bool,
		validate: bool,
		config: &evm::Config,
	) -> Result<CreateInfo, RunnerError<Self::Error>> {
		// we force the value to be zero because we don't support value transfer in EVM
		let value = U256::from(0);
		if validate {
			Self::validate(
				source,
				None,
				init.clone(),
				value,
				gas_limit,
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.clone(),
				is_transactional,
				config,
			)?;
		}
		let precompiles = T::PrecompilesValue::get();
		let code_hash = H256::from(sp_io::hashing::keccak_256(&init));
		Self::execute(
			source,
			None,
			value,
			gas_limit,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			config,
			&precompiles,
			is_transactional,
			|executor| {
				let address = executor.create_address(evm::CreateScheme::Create2 {
					caller: source,
					code_hash,
					salt,
				});
				let (reason, _) =
					executor.transact_create2(source, value, init, salt, gas_limit, access_list);
				(reason, address)
			},
		)
	}
}

struct SubstrateStackSubstate<'config> {
	metadata: StackSubstateMetadata<'config>,
	deletes: BTreeSet<H160>,
	logs: Vec<Log>,
	parent: Option<Box<SubstrateStackSubstate<'config>>>,
}

impl<'config> SubstrateStackSubstate<'config> {
	pub fn metadata(&self) -> &StackSubstateMetadata<'config> {
		&self.metadata
	}

	pub fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'config> {
		&mut self.metadata
	}

	pub fn enter(&mut self, gas_limit: u64, is_static: bool) {
		let mut entering = Self {
			metadata: self.metadata.spit_child(gas_limit, is_static),
			parent: None,
			deletes: BTreeSet::new(),
			logs: Vec::new(),
		};
		mem::swap(&mut entering, self);

		self.parent = Some(Box::new(entering));

		sp_io::storage::start_transaction();
	}

	pub fn exit_commit(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot commit on root substate");
		mem::swap(&mut exited, self);

		self.metadata.swallow_commit(exited.metadata)?;
		self.logs.append(&mut exited.logs);
		self.deletes.append(&mut exited.deletes);

		sp_io::storage::commit_transaction();
		Ok(())
	}

	pub fn exit_revert(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot discard on root substate");
		mem::swap(&mut exited, self);
		self.metadata.swallow_revert(exited.metadata)?;

		sp_io::storage::rollback_transaction();
		Ok(())
	}

	pub fn exit_discard(&mut self) -> Result<(), ExitError> {
		let mut exited = *self.parent.take().expect("Cannot discard on root substate");
		mem::swap(&mut exited, self);
		self.metadata.swallow_discard(exited.metadata)?;

		sp_io::storage::rollback_transaction();
		Ok(())
	}

	pub fn deleted(&self, address: H160) -> bool {
		if self.deletes.contains(&address) {
			return true;
		}

		if let Some(parent) = self.parent.as_ref() {
			return parent.deleted(address);
		}

		false
	}

	pub fn set_deleted(&mut self, address: H160) {
		self.deletes.insert(address);
	}

	pub fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) {
		self.logs.push(Log {
			address,
			topics,
			data,
		});
	}

	fn recursive_is_cold<F: Fn(&Accessed) -> bool>(&self, f: &F) -> bool {
		let local_is_accessed = self.metadata.accessed().as_ref().map(f).unwrap_or(false);
		if local_is_accessed {
			false
		} else {
			self.parent
				.as_ref()
				.map(|p| p.recursive_is_cold(f))
				.unwrap_or(true)
		}
	}
}

/// Substrate backend for EVM.
pub struct SubstrateStackState<'vicinity, 'config, T> {
	vicinity: &'vicinity Vicinity,
	substate: SubstrateStackSubstate<'config>,
	original_storage: BTreeMap<(H160, H256), H256>,
	_marker: PhantomData<T>,
}

impl<'vicinity, 'config, T: Config> SubstrateStackState<'vicinity, 'config, T> {
	/// Create a new backend with given vicinity.
	pub fn new(vicinity: &'vicinity Vicinity, metadata: StackSubstateMetadata<'config>) -> Self {
		Self {
			vicinity,
			substate: SubstrateStackSubstate {
				metadata,
				deletes: BTreeSet::new(),
				logs: Vec::new(),
				parent: None,
			},
			_marker: PhantomData,
			original_storage: BTreeMap::new(),
		}
	}
}

impl<'vicinity, 'config, T: Config> BackendT for SubstrateStackState<'vicinity, 'config, T> {
	fn gas_price(&self) -> U256 {
		self.vicinity.gas_price
	}
	fn origin(&self) -> H160 {
		self.vicinity.origin
	}

	fn block_hash(&self, number: U256) -> H256 {
		if number > U256::from(u32::MAX) {
			H256::default()
		} else {
			T::BlockHashMapping::block_hash(number.as_u32())
		}
	}

	fn block_number(&self) -> U256 {
		let number: u128 = frame_system::Pallet::<T>::block_number().unique_saturated_into();
		U256::from(number)
	}

	fn block_coinbase(&self) -> H160 {
		Pallet::<T>::find_author()
	}

	fn block_timestamp(&self) -> U256 {
		let now: u128 = pallet_timestamp::Pallet::<T>::get().unique_saturated_into();
		U256::from(now / 1000)
	}

	fn block_difficulty(&self) -> U256 {
		U256::zero()
	}

	fn block_gas_limit(&self) -> U256 {
		T::BlockGasLimit::get()
	}

	fn chain_id(&self) -> U256 {
		U256::from(T::ChainId::get())
	}

	fn exists(&self, _address: H160) -> bool {
		true
	}

	fn basic(&self, address: H160) -> evm::backend::Basic {
		let (account, _) = Pallet::<T>::account_basic(&address);

		evm::backend::Basic {
			balance: account.balance,
			nonce: account.nonce,
		}
	}

	fn code(&self, address: H160) -> Vec<u8> {
		<AccountCodes<T>>::get(address)
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		<AccountStorages<T>>::get(address, index)
	}

	fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
		// Not being cached means that it was never changed, which means we
		// can fetch it from storage.
		Some(
			self.original_storage
				.get(&(address, index))
				.cloned()
				.unwrap_or_else(|| self.storage(address, index)),
		)
	}

	fn block_base_fee_per_gas(&self) -> sp_core::U256 {
		let (base_fee, _) = T::FeeCalculator::min_gas_price();
		base_fee
	}
}

impl<'vicinity, 'config, T: Config> StackStateT<'config>
	for SubstrateStackState<'vicinity, 'config, T>
where
	BalanceOf<T>: TryFrom<U256> + Into<U256>,
{
	fn metadata(&self) -> &StackSubstateMetadata<'config> {
		self.substate.metadata()
	}

	fn metadata_mut(&mut self) -> &mut StackSubstateMetadata<'config> {
		self.substate.metadata_mut()
	}

	fn enter(&mut self, gas_limit: u64, is_static: bool) {
		self.substate.enter(gas_limit, is_static)
	}

	fn exit_commit(&mut self) -> Result<(), ExitError> {
		self.substate.exit_commit()
	}

	fn exit_revert(&mut self) -> Result<(), ExitError> {
		self.substate.exit_revert()
	}

	fn exit_discard(&mut self) -> Result<(), ExitError> {
		self.substate.exit_discard()
	}

	fn is_empty(&self, address: H160) -> bool {
		Pallet::<T>::is_account_empty(&address)
	}

	fn deleted(&self, address: H160) -> bool {
		self.substate.deleted(address)
	}

	fn inc_nonce(&mut self, address: H160) {
		let account_id = T::AddressMapping::into_account_id(address);
		frame_system::Pallet::<T>::inc_account_nonce(&account_id);
	}

	fn set_storage(&mut self, address: H160, index: H256, value: H256) {
		// We cache the current value if this is the first time we modify it
		// in the transaction.
		use sp_std::collections::btree_map::Entry::Vacant;
		if let Vacant(e) = self.original_storage.entry((address, index)) {
			let original = <AccountStorages<T>>::get(address, index);
			// No need to cache if same value.
			if original != value {
				e.insert(original);
			}
		}

		// Then we insert or remove the entry based on the value.
		if value == H256::default() {
			log::debug!(
				target: "evm",
				"Removing storage for {:?} [index: {:?}]",
				address,
				index,
			);
			<AccountStorages<T>>::remove(address, index);
		} else {
			log::debug!(
				target: "evm",
				"Updating storage for {:?} [index: {:?}, value: {:?}]",
				address,
				index,
				value,
			);
			<AccountStorages<T>>::insert(address, index, value);
		}
	}

	fn reset_storage(&mut self, address: H160) {
		#[allow(deprecated)]
		let _ = <AccountStorages<T>>::remove_prefix(address, None);
	}

	fn log(&mut self, address: H160, topics: Vec<H256>, data: Vec<u8>) {
		self.substate.log(address, topics, data)
	}

	fn set_deleted(&mut self, address: H160) {
		self.substate.set_deleted(address)
	}

	fn set_code(&mut self, address: H160, code: Vec<u8>) {
		log::debug!(
			target: "evm",
			"Inserting code ({} bytes) at {:?}",
			code.len(),
			address
		);
		Pallet::<T>::create_account(address, code);
	}

	fn transfer(&mut self, transfer: Transfer) -> Result<(), ExitError> {
		let source = T::AddressMapping::into_account_id(transfer.source);
		let target = T::AddressMapping::into_account_id(transfer.target);

		T::Currency::transfer(
			&source,
			&target,
			transfer
				.value
				.try_into()
				.map_err(|_| ExitError::OutOfFund)?,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| ExitError::OutOfFund)
	}

	fn reset_balance(&mut self, _address: H160) {
		// Do nothing on reset balance in Substrate.
		//
		// This function exists in EVM because a design issue
		// (arguably a bug) in SELFDESTRUCT that can cause total
		// issuance to be reduced. We do not need to replicate this.
	}

	fn touch(&mut self, _address: H160) {
		// Do nothing on touch in Substrate.
		//
		// EVM pallet considers all accounts to exist, and distinguish
		// only empty and non-empty accounts. This avoids many of the
		// subtle issues in EIP-161.
	}

	fn is_cold(&self, address: H160) -> bool {
		self.substate
			.recursive_is_cold(&|a| a.accessed_addresses.contains(&address))
	}

	fn is_storage_cold(&self, address: H160, key: H256) -> bool {
		self.substate
			.recursive_is_cold(&|a: &Accessed| a.accessed_storage.contains(&(address, key)))
	}
}

pub trait OnChargeDecentralizedNativeTokenFee {
	type Error;

	// Get the fee token of the user.
	fn get_transaction_fee_token(from: H160) -> H160;

	// Get the fee token of the validator and its conversion rate.
	fn get_transaction_conversion_rate(validator: H160, token: H160) -> (U256, U256);

	// Get fee vault address
	fn get_fee_vault() -> H160;

	// Withdraws the fee from the user.
	fn withdraw_fee(
		from: H160,
		token: H160,
		conversion_rate: (U256, U256),
		amount: U256,
	) -> Result<(), Self::Error>;

	// Corrects the fee if the actual amount is different from the paid amount.
	fn correct_fee(
		from: H160,
		token: H160,
		conversion_rate: (U256, U256),
		paid_amount: U256,
		actual_amount: U256,
	) -> Result<(), Self::Error>;

	// Distributes the fee to the validator and dApp.
	fn pay_fees(
		token: H160,
		conversion_rate: (U256, U256),
		actual_amount: U256,
		validator: H160,
		to: Option<H160>,
	) -> Result<(U256, U256), Self::Error>;
}
