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
use frame_support::parameter_types;
use frame_support::storage::types::{StorageValue, ValueQuery};
use sp_runtime::traits::Dispatchable;

use frame_support::traits::StorageInstance;
use precompile_utils::prelude::*;

use evm::Context;
use evm::ExitReason;
use evm::ExitSucceed;
use pallet_evm::AddressMapping;
use sp_core::Get;
use sp_core::{H160, H256, U256};
use sp_std::marker::PhantomData;
use sp_std::vec;
use sp_std::vec::Vec;
use stbl_tools::misc::{bool_to_vec_u8, u256_to_vec_u8};

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");
pub const SELECTOR_LOG_TRANSFER_OWNER: [u8; 32] =
	keccak256!("OwnershipTransferStarted(address,address)");
pub const SELECTOR_REWARD_CLAIMED: [u8; 32] = keccak256!("RewardClaimed(address,address,address)");
pub const SELECTOR_WHITELIST_STATUS_UPDATED: [u8; 32] =
	keccak256!("WhitelistStatusUpdated(address,bool)");
pub const SELECTOR_VALIDATOR_PERCENTAGE_UPDATED: [u8; 32] =
	keccak256!("ValidatorPercentageUpdated(uint256)");

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
	Runtime: pallet_fee_rewards_vault::Config
		+ pallet_timestamp::Config
		+ pallet_evm::Config
		+ pallet_dnt_fee_controller::Config
		+ pallet_validator_set::Config,
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

	#[precompile::public("claimReward(address,address)")]
	fn claim_reward(
		handle: &mut impl PrecompileHandle,
		holder: Address,
		token: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;

		if !Self::can_claim_reward(handle, msg_sender.into(), holder)? {
			return Err(revert("sender is not allowed to claim reward"));
		}

		let reward = pallet_fee_rewards_vault::Pallet::<Runtime>::get_claimable_reward(
			holder.into(),
			token.into(),
		);

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		pallet_fee_rewards_vault::Pallet::<Runtime>::sub_claimable_reward(
			holder.into(),
			token.into(),
			reward,
		)
		.map_err(|_| revert("fail trying to sub claimable reward"))?;

		let encoded_data = stbl_tools::eth::generate_calldata(
			&"transfer(address,uint256)",
			&vec![msg_sender.into(), stbl_tools::misc::u256_to_h256(reward)],
		);

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let (reason, _) = handle.call(
			token.into(),
			None,
			encoded_data,
			Some(handle.remaining_gas()),
			false,
			&Context {
				address: token.into(),
				caller: handle.context().address,
				apparent_value: U256::zero(),
			},
		);

		if reason != ExitReason::Succeed(ExitSucceed::Returned) {
			return Err(revert("fail trying to transfer reward"));
		}

		handle.record_log_costs_manual(3, 32)?;
		log3(
			handle.context().address,
			SELECTOR_REWARD_CLAIMED,
			holder.0,
			msg_sender,
			Vec::from(token.0.to_fixed_bytes()),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("canClaimReward(address,address)")]
	#[precompile::view]
	fn can_claim_reward(
		handle: &mut impl PrecompileHandle,
		claimant: Address,
		holder: Address,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let holder_account_id: <Runtime as frame_system::Config>::AccountId =
			<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(holder.into());
		let is_whitelisted =
			pallet_fee_rewards_vault::Pallet::<Runtime>::is_whitelisted(holder.into());
		let is_validator =
			pallet_validator_set::Validators::<Runtime>::get().contains(&holder_account_id);

		if !is_whitelisted && !is_validator {
			return Ok(false);
		}

		if claimant == holder {
			return Ok(true);
		}

		let code = pallet_evm::AccountCodes::<Runtime>::get(Into::<H160>::into(holder));

		if code.is_empty() {
			return Ok(false);
		}

		if !stbl_tools::eth::code_implements_function(code.as_slice(), &"owner()") {
			return Ok(false);
		}

		let call_data = stbl_tools::eth::generate_calldata(&"owner()", &vec![]);

		let (reason, output) = handle.call(
			holder.into(),
			None,
			call_data,
			Some(handle.remaining_gas()),
			true,
			&Context {
				address: holder.into(),
				caller: handle.context().address,
				apparent_value: U256::zero(),
			},
		);

		match reason {
			ExitReason::Succeed(ExitSucceed::Returned) => {
				let owner = H160::from_slice(&output[12..32]);
				Ok(owner == claimant.into())
			}
			_ => Err(revert("call to owner() failed")),
		}
	}

	#[precompile::public("getClaimableReward(address,address)")]
	#[precompile::view]
	fn get_claimable_reward(
		handle: &mut impl PrecompileHandle,
		holder: Address,
		token: Address,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let reward = pallet_fee_rewards_vault::Pallet::<Runtime>::get_claimable_reward(
			holder.into(),
			token.into(),
		);

		Ok(reward)
	}

	#[precompile::public("isWhitelisted(address)")]
	#[precompile::view]
	fn is_whitelisted(handle: &mut impl PrecompileHandle, holder: Address) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let whitelisted =
			pallet_fee_rewards_vault::Pallet::<Runtime>::is_whitelisted(holder.into());

		Ok(whitelisted)
	}

	#[precompile::public("setWhitelisted(address,bool)")]
	fn set_whitelist(
		handle: &mut impl PrecompileHandle,
		holder: Address,
		is_whitelisted: bool,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		let code = pallet_evm::AccountCodes::<Runtime>::get(Into::<H160>::into(holder));

		if code.is_empty() {
			return Err(revert("address is not a smartcontract"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		pallet_fee_rewards_vault::Pallet::<Runtime>::set_whitelist(holder.into(), is_whitelisted);

		handle.record_log_costs_manual(2, 32)?;
		log2(
			handle.context().address,
			SELECTOR_WHITELIST_STATUS_UPDATED,
			holder.0,
			bool_to_vec_u8(is_whitelisted),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("getValidatorPercentage()")]
	#[precompile::view]
	fn get_validator_percentage(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let percentage = pallet_dnt_fee_controller::Pallet::<Runtime>::get_validator_percentage();

		Ok(percentage)
	}

	#[precompile::public("setValidatorPercentage(uint256)")]
	fn set_validator_percentage(
		handle: &mut impl PrecompileHandle,
		percentage: U256,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		if percentage > U256::from(100) {
			return Err(revert("percentage is too high"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		pallet_dnt_fee_controller::Pallet::<Runtime>::set_validator_percentage(percentage).unwrap();

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_VALIDATOR_PERCENTAGE_UPDATED,
			u256_to_vec_u8(percentage),
		)
		.record(handle)?;

		Ok(true)
	}
}
