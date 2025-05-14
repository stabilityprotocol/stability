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

use frame_support::traits::ChangeMembers;
use frame_support::traits::StorageInstance;
use precompile_utils::prelude::*;
use sp_std::vec::Vec;

use sp_core::Get;
use sp_core::{H160, H256, U256};

use sp_std::marker::PhantomData;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");
pub const SELECTOR_LOG_TRANSFER_OWNER: [u8; 32] =
	keccak256!("OwnershipTransferStarted(address,address)");
pub const SELECTOR_SETTED__APPLICATION_BLOCK: [u8; 32] =
	keccak256!("SettedApplicationBlock(uint256)");
pub const SELECTOR_CODE_PROPOSED_REJECTED: [u8; 32] = keccak256!("CodeProposedRejected()");
pub const SELECTOR_MEMBER_ADDED: [u8; 32] = keccak256!("MemberAdded(address)");
pub const SELECTOR_MEMBER_REMOVED: [u8; 32] = keccak256!("MemberRemoved(address)");

/// Storage prefix for owner.
pub struct OwnerPrefix;

impl StorageInstance for OwnerPrefix {
	const STORAGE_PREFIX: &'static str = "Owner";

	fn pallet_prefix() -> &'static str {
		"PrecompileUpgradeRuntimeController"
	}
}
pub struct ClaimableOwner;

impl StorageInstance for ClaimableOwner {
	const STORAGE_PREFIX: &'static str = "ClaimableOwner";

	fn pallet_prefix() -> &'static str {
		"PrecompileUpgradeRuntimeController"
	}
}

pub type OwnerStorage<DefaultOwner> = StorageValue<OwnerPrefix, H160, ValueQuery, DefaultOwner>;

pub type ClaimableOwnerStorage = StorageValue<ClaimableOwner, H160, ValueQuery, ZeroAddress>;

pub struct UpgradeRuntimeControllerPrecompile<Runtime, DefaultOwner: Get<H160> + 'static>(
	PhantomData<(Runtime, DefaultOwner)>,
);

#[precompile_utils::precompile]
impl<Runtime, DefaultOwner> UpgradeRuntimeControllerPrecompile<Runtime, DefaultOwner>
where
	DefaultOwner: Get<H160> + 'static,
	Runtime: pallet_upgrade_runtime_proposal::Config
		+ pallet_collective::Config<pallet_collective::Instance1>
		+ pallet_timestamp::Config
		+ pallet_evm::Config,
	Runtime::RuntimeCall: From<pallet_upgrade_runtime_proposal::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	<Runtime as frame_system::Config>::Hash: Into<H256>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
	<Runtime as frame_system::Config>::AccountId: Into<H160>,
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

	#[precompile::public("setApplicationBlock(uint32)")]
	fn set_application_block(handle: &mut impl PrecompileHandle, block_number: u32) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			frame_system::RawOrigin::Root.into(),
			pallet_upgrade_runtime_proposal::Call::<Runtime>::set_block_application {
				block_number: block_number.into(),
			},
		)?;

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_SETTED__APPLICATION_BLOCK,
			solidity::encode_event_data(block_number),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("rejectProposedCode()")]
	fn reject_proposed_code(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			frame_system::RawOrigin::Root.into(),
			pallet_upgrade_runtime_proposal::Call::<Runtime>::reject_proposed_code {},
		)?;

		handle.record_log_costs_manual(0, 32)?;
		log0(handle.context().address, SELECTOR_CODE_PROPOSED_REJECTED).record(handle)?;

		Ok(())
	}

	#[precompile::public("addMemberToTechnicalCommittee(address)")]
	fn add_member_to_technical_committee(
		handle: &mut impl PrecompileHandle,
		member: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let member_id: H160 = member.into();

		let old_members =
			pallet_collective::Members::<Runtime, pallet_collective::Instance1>::get();

		if old_members.contains(&member_id.into()) {
			return Err(revert("already a member"));
		}

		let mut new_members = old_members
			.iter()
			.cloned()
			.chain(Some(member_id.into()))
			.collect::<Vec<_>>();

		new_members.sort();

		pallet_collective::Pallet::<Runtime, pallet_collective::Instance1>::set_members_sorted(
			&new_members,
			&old_members,
		);

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_MEMBER_ADDED,
			solidity::encode_event_data(Into::<Address>::into(member_id)),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("removeMemberFromTechnicalCommittee(address)")]
	fn remove_member_to_technical_committee(
		handle: &mut impl PrecompileHandle,
		member: Address,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let member_id: H160 = member.into();
		let member_account = <Runtime as frame_system::Config>::AccountId::from(member_id);

		let old_members =
			pallet_collective::Members::<Runtime, pallet_collective::Instance1>::get();

		if !old_members.contains(&member_account) {
			return Err(revert("not a member"));
		}

		let mut new_members = old_members
			.iter()
			.cloned()
			.filter(|m| *m != member_account)
			.collect::<Vec<_>>();

		new_members.sort();

		pallet_collective::Pallet::<Runtime, pallet_collective::Instance1>::set_members_sorted(
			&new_members,
			&old_members,
		);

		handle.record_log_costs_manual(1, 32)?;
		log1(
			handle.context().address,
			SELECTOR_MEMBER_REMOVED,
			solidity::encode_event_data(Into::<Address>::into(member_id)),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("getTechnicalCommitteeMembers()")]
	#[precompile::view]
	fn get_technical_committee_members(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<Vec<Address>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let members = pallet_collective::Members::<Runtime, pallet_collective::Instance1>::get();

		Ok(members
			.iter()
			.map(|m| Into::<H160>::into(m.clone()))
			.map(|m| Into::<Address>::into(m))
			.collect())
	}

	#[precompile::public("getHashOfProposedCode()")]
	#[precompile::view]
	fn get_hash_of_proposed_code(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let hash = pallet_upgrade_runtime_proposal::Pallet::<Runtime>::hash_of_proposed_code();

		match hash {
			Some(hash) => Ok(hash.into()),
			None => Ok(H256::default()),
		}
	}

	#[precompile::public("getHashOfCurrentCode()")]
	#[precompile::view]
	fn get_hash_of_current_code(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let hash = pallet_upgrade_runtime_proposal::Pallet::<Runtime>::get_current_code_hash();

		match hash {
			Some(hash) => Ok(hash.into()),
			None => Ok(H256::default()),
		}
	}
}
