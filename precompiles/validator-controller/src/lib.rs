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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use core::str::FromStr;
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use sp_runtime::traits::Dispatchable;

use frame_support::parameter_types;
use frame_support::storage::types::{StorageValue, ValueQuery};

use frame_support::traits::StorageInstance;
use pallet_custom_balances::AccountIdMapping;
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;

use sp_core::Get;
use sp_core::{H160, H256, U256};

use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");
pub const SELECTOR_LOG_TRANSFER_OWNER: [u8; 32] =
	keccak256!("OwnershipTransferStarted(address,address)");
pub const SELECTOR_VALIDATOR_ADDED: [u8; 32] = keccak256!("ValidatorAdded(bytes32)");
pub const SELECTOR_VALIDATOR_REMOVED: [u8; 32] = keccak256!("ValidatorRemoved(bytes32)");

/// Storage prefix for owner.
pub struct OwnerPrefix;

impl StorageInstance for OwnerPrefix {
	const STORAGE_PREFIX: &'static str = "Owner";

	fn pallet_prefix() -> &'static str {
		"PrecompileValidatorController"
	}
}
pub struct ClaimableOwner;

impl StorageInstance for ClaimableOwner {
	const STORAGE_PREFIX: &'static str = "ClaimableOwner";

	fn pallet_prefix() -> &'static str {
		"PrecompileValidatorController"
	}
}

pub type OwnerStorage<DefaultOwner> = StorageValue<OwnerPrefix, H160, ValueQuery, DefaultOwner>;

pub type ClaimableOwnerStorage = StorageValue<ClaimableOwner, H160, ValueQuery, ZeroAddress>;

pub struct ValidatorControllerPrecompile<Runtime, DefaultOwner: Get<H160> + 'static>(
	PhantomData<(Runtime, DefaultOwner)>,
);

#[precompile_utils::precompile]
impl<Runtime, DefaultOwner> ValidatorControllerPrecompile<Runtime, DefaultOwner>
where
	DefaultOwner: Get<H160> + 'static,
	Runtime: pallet_validator_set::Config
		+ pallet_timestamp::Config
		+ pallet_evm::Config
		+ pallet_custom_balances::Config
		+ pallet_evm::Config,
	Runtime::RuntimeCall: From<pallet_validator_set::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
{
	#[precompile::public("owner()")]
	#[precompile::view]
	fn owner(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(OwnerStorage::<DefaultOwner>::get().into())
	}

	#[precompile::public("pendingOwner()")]
	#[precompile::view]
	fn pending_owner(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ClaimableOwnerStorage::get().into())
	}

	#[precompile::public("transferOwnership(address)")]
	fn transfer_ownership(handle: &mut impl PrecompileHandle, new_owner: Address) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		ClaimableOwnerStorage::mutate(move |claimable_owner| {
			*claimable_owner = new_owner.into();
		});

		let target_new_owner: H160 = new_owner.into();

		handle.record_log_costs_manual(2, 32)?;
		log2(
			handle.context().address,
			SELECTOR_LOG_TRANSFER_OWNER,
			Into::<H256>::into(owner),
			solidity::encode_event_data(Into::<H256>::into(target_new_owner)),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("acceptOwnership()")]
	fn claim_ownership(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;

		let target_new_owner = ClaimableOwnerStorage::get();

		if target_new_owner != msg_sender {
			return Err(revert("target owner is not the claimer"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		ClaimableOwnerStorage::kill();
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		OwnerStorage::<DefaultOwner>::mutate(|owner| {
			*owner = target_new_owner;
		});

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_LOG_NEW_OWNER,
			solidity::encode_event_data(Into::<H256>::into(target_new_owner)),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("getValidatorList()")]
	#[precompile::view]
	fn get_validator_list(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Address>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let validators = pallet_validator_set::ApprovedValidators::<Runtime>::get();

		let validators_h160: Vec<H160> = validators
			.iter()
			.map(|v| Runtime::AccountIdMapping::into_evm_address(v))
			.collect();

		Ok(validators_h160
			.iter()
			.map(|v| Into::<Address>::into(*v))
			.collect())
	}

	#[precompile::public("getActiveValidatorList()")]
	#[precompile::view]
	fn get_active_validator_list(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Address>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let validators = pallet_validator_set::Validators::<Runtime>::get();

		let validators_h160: Vec<H160> = validators
			.iter()
			.map(|v| Runtime::AccountIdMapping::into_evm_address(v))
			.collect();

		Ok(validators_h160
			.iter()
			.map(|v| Into::<Address>::into(*v))
			.collect())
	}

	#[precompile::public("getValidatorMissingBlocks(address)")]
	#[precompile::view]
	fn get_validator_missing_blocks(
		handle: &mut impl PrecompileHandle,
		validator: Address,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let account_id =
			<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(validator.into());
		let epochs_missed = pallet_validator_set::EpochsMissed::<Runtime>::get(account_id);
		Ok(epochs_missed)
	}

	#[precompile::public("addValidator(address)")]
	fn add_validator(handle: &mut impl PrecompileHandle, new_validator: Address) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		let origin_id: H160 = new_validator.into();

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			frame_system::RawOrigin::Root.into(),
			pallet_validator_set::Call::<Runtime>::add_validator {
				validator_id: origin_id.into(),
			},
		)?;

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_VALIDATOR_ADDED,
			solidity::encode_event_data(Into::<H256>::into(origin_id)),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("removeValidator(address)")]
	fn remove_validator(
		handle: &mut impl PrecompileHandle,
		removed_validator: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		let origin_id: H160 = removed_validator.into();

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			frame_system::RawOrigin::Root.into(),
			pallet_validator_set::Call::<Runtime>::remove_validator {
				validator_id: origin_id.into(),
			},
		)?;

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_VALIDATOR_REMOVED,
			solidity::encode_event_data(Into::<H256>::into(origin_id)),
		)
		.record(handle)?;

		Ok(())
	}
}
