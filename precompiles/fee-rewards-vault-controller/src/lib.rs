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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use core::str::FromStr;
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};

use frame_support::parameter_types;
use frame_support::storage::types::{StorageValue, ValueQuery};

use frame_support::traits::StorageInstance;
use precompile_utils::prelude::*;

use evm::ExitReason;
use evm::ExitSucceed;
use evm::Context;
use sp_core::Get;
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;
use sp_std::vec;

use sp_std::marker::PhantomData;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");

pub const SELECTOR_REWARD_CLAIMED: [u8; 32] = keccak256!("RewardClaimed(address,address,address)");
pub const SELECTOR_WHITELIST_STATUS_UPDATED: [u8; 32] = keccak256!("WhitelistStatusUpdated(address,bool)");
pub const SELECTOR_VALIDATOR_PERCENTAGE_UPDATED: [u8; 32] = keccak256!("ValidatorPercentageUpdated(uint256)");

/// Storage prefix for owner.
pub struct OwnerPrefix;

impl StorageInstance for OwnerPrefix {
	const STORAGE_PREFIX: &'static str = "Owner";

	fn pallet_prefix() -> &'static str {
		"PrecompileFeeRewardsVaultController"
	}
}
pub struct ClaimableOwner;

impl StorageInstance for ClaimableOwner {
	const STORAGE_PREFIX: &'static str = "ClaimableOwner";

	fn pallet_prefix() -> &'static str {
		"PrecompileFeeRewardsVaultController"
	}
}

pub type OwnerStorage<DefaultOwner> = StorageValue<OwnerPrefix, H160, ValueQuery, DefaultOwner>;

pub type ClaimableOwnerStorage = StorageValue<ClaimableOwner, H160, ValueQuery, ZeroAddress>;

pub struct FeeRewardsVaultControllerPrecompile<Runtime, DefaultOwner: Get<H160> + 'static>(
	PhantomData<(Runtime, DefaultOwner)>,
);


#[precompile_utils::precompile]
impl<Runtime, DefaultOwner> FeeRewardsVaultControllerPrecompile<Runtime, DefaultOwner>
where
	DefaultOwner: Get<H160> + 'static,
	Runtime: pallet_fee_rewards_vault::Config + pallet_timestamp::Config + pallet_evm::Config + pallet_dnt_fee_controller::Config,
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

	#[precompile::public("claimReward(address,address)")]
	fn claim_reward(handle: &mut impl PrecompileHandle, holder: Address, token: Address) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let sender = handle.context().caller;
		

		if !Self::can_claim_reward(handle, sender.into(), holder)? {
			return Err(revert("sender is not allowed to claim reward"));
		}

		let reward = pallet_fee_rewards_vault::Pallet::<Runtime>::get_claimable_reward(holder.into(), token.into());

		pallet_fee_rewards_vault::Pallet::<Runtime>::sub_claimable_reward(holder.into(), token.into(), reward).map_err(|_| revert("fail trying to sub claimable reward"))?;
	
		let encoded_data = stbl_tools::eth::generate_calldata(&"transfer(address,uint256)", &vec![sender.into(), stbl_tools::misc::u256_to_h256(reward)]);
	
		let (reason, _) =  handle.call(token.into(), None, encoded_data, Some(handle.remaining_gas()), false, &Context {
			address: token.into(),
			caller: handle.context().address, 
			apparent_value: U256::zero()
		});
		
		if reason != ExitReason::Succeed(ExitSucceed::Returned) {
			return Err(revert("fail trying to transfer reward"));
		}

		log3(handle.context().address, SELECTOR_REWARD_CLAIMED, holder.0, sender, Vec::from(token.0.to_fixed_bytes())).record(handle)?;
		
		Ok(())
	}

	#[precompile::public("canClaimReward(address,address)")]
	#[precompile::view]
    fn can_claim_reward(handle: &mut impl PrecompileHandle, claimant: Address, holder: Address) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		// TODO: AFTER UNIFIED ACCOUNT, DONT CHECK IF IS WHITELISTED IF THE HOLDER IS A VALIDATOR
	
		if !pallet_fee_rewards_vault::Pallet::<Runtime>::is_whitelisted(holder.into()) {
			return Ok(false);
		}


		let code = pallet_evm::Pallet::<Runtime>::account_codes::<H160>(holder.into());

		if code.is_empty() {
			return Ok(false);
		}

		if claimant == holder {
			return Ok(true);
		}

		if !stbl_tools::eth::code_implements_function(code.as_slice(), &"owner()") {
			return Ok(false);
		}

		let call_data = stbl_tools::eth::generate_calldata(&"owner()", &vec![]);

		
		let (reason, output) = handle.call(holder.into(), None, call_data, Some(handle.remaining_gas()), true, &Context {
			address: holder.into(),
			caller: handle.context().address, 
			apparent_value: U256::zero()
		});

		if reason != ExitReason::Succeed(ExitSucceed::Returned) {
			return Err(revert("call to owner() failed"));
		}

		let owner = H160::from_slice(&output[12..32]);

		Ok(owner == claimant.into())
	}

	#[precompile::public("getClaimableReward(address,address)")]
	#[precompile::view]
	fn get_claimable_reward(handle: &mut impl PrecompileHandle, holder: Address, token: Address) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let reward = pallet_fee_rewards_vault::Pallet::<Runtime>::get_claimable_reward(holder.into(), token.into());

		Ok(reward)
	}

	#[precompile::public("isWhitelisted(address)")]
	#[precompile::view]
	fn is_whitelisted(handle: &mut impl PrecompileHandle, holder: Address) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let whitelisted = pallet_fee_rewards_vault::Pallet::<Runtime>::is_whitelisted(holder.into());

		Ok(whitelisted)
	}

	#[precompile::public("setWhitelisted(address,bool)")]
	fn set_whitelist(handle: &mut impl PrecompileHandle, holder: Address, is_whitelisted: bool) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if sender != owner {
			return Err(revert("sender is not owner"));
		}

		let code = pallet_evm::Pallet::<Runtime>::account_codes::<H160>(holder.into());

		if code.is_empty() {
			return Err(revert("address is not a smartcontract"));
		}

		pallet_fee_rewards_vault::Pallet::<Runtime>::set_whitelist(holder.into(), is_whitelisted);


		log2(handle.context().address, SELECTOR_WHITELIST_STATUS_UPDATED, holder.0, bool_to_vec_u8(is_whitelisted)).record(handle)?;
		
		Ok(true)
	}

	#[precompile::public("getValidatorPercentage()")]
	fn get_validator_percentage(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let percentage = pallet_dnt_fee_controller::Pallet::<Runtime>::get_validator_percentage();

		Ok(percentage)
	}

	#[precompile::public("setValidatorPercentage(uint256)")]
	fn set_validator_percentage(handle: &mut impl PrecompileHandle, percentage: U256) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if sender != owner {
			return Err(revert("sender is not owner"));
		}

		if percentage > U256::from(100) {
			return Err(revert("percentage is too high"));
		}
	
		pallet_dnt_fee_controller::Pallet::<Runtime>::set_validator_percentage(percentage).unwrap();

		log1(handle.context().address, SELECTOR_VALIDATOR_PERCENTAGE_UPDATED, u256_to_vec_u8(percentage)).record(handle)?;
		
		Ok(true)
	}

}

fn bool_to_vec_u8(value: bool) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.push(value as u8);
    vec
}

fn u256_to_vec_u8(value: U256) -> Vec<u8> {
	let mut bytes = [0u8; 32];
	let bytes_slice = bytes.as_mut_slice();

    value.to_big_endian(bytes_slice);

	bytes_slice.to_vec()
}