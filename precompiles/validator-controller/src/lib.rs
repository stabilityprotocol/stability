// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use core::str::FromStr;
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::parameter_types;
use frame_support::storage::types::{StorageValue, ValueQuery};
use frame_support::traits::StorageInstance;
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::Get;
use sp_core::{H160, H256, U256};
use sp_std::marker::PhantomData;

parameter_types! {
    pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

/// Solidity selector of the Withdraw log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");

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
    Runtime: pallet_validator_set::Config + pallet_timestamp::Config + pallet_evm::Config,
    Runtime::RuntimeCall: From<pallet_validator_set::Call<Runtime>>,
    <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
    Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
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
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

        let sender = handle.context().caller;
        let owner = OwnerStorage::<DefaultOwner>::get();

        if sender != owner {
            return Err(revert("sender is not owner"));
        }

        ClaimableOwnerStorage::mutate(move |claimable_owner| {
            *claimable_owner = new_owner.into();
        });

        Ok(())
    }

    #[precompile::public("acceptOwnership()")]
    fn claim_ownership(handle: &mut impl PrecompileHandle) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

        handle.record_log_costs_manual(1, 32)?;

        let sender = handle.context().caller;

        let target_new_owner = ClaimableOwnerStorage::get();

        if target_new_owner != sender {
            return Err(revert("target owner is not the claimer"));
        }

        ClaimableOwnerStorage::kill();
        OwnerStorage::<DefaultOwner>::mutate(|owner| {
            *owner = target_new_owner;
        });

        log1(
            handle.context().address,
            SELECTOR_LOG_NEW_OWNER,
            EvmDataWriter::new()
                .write(Into::<H256>::into(target_new_owner))
                .build(),
        )
        .record(handle)?;

        Ok(())
    }

    #[precompile::public("addValidator(address)")]
    fn add_validator(handle: &mut impl PrecompileHandle, new_validator: Address) -> EvmResult {
        handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
        handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

        let sender = handle.context().caller;
        let owner = OwnerStorage::<DefaultOwner>::get();

        if sender != owner {
            return Err(revert("sender is not owner"));
        }

        let new_validator_account_id =
            Runtime::AddressMapping::into_account_id(new_validator.into());

        RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            frame_system::RawOrigin::Root.into(),
            pallet_validator_set::Call::<Runtime>::add_validator {
                validator_id: new_validator_account_id,
            },
        )?;

        Ok(())
    }
}
