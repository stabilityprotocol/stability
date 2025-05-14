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

//! Precompile to interact with pallet_balances instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use core::str::FromStr;
use sp_std::vec::Vec;

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::pallet_prelude::{StorageValue, ValueQuery};
use frame_support::parameter_types;
use sp_runtime::traits::Dispatchable;

use frame_support::traits::StorageInstance;
use precompile_utils::prelude::*;
use sp_core::{Get, H160, H256, U256};
use sp_std::marker::PhantomData;

use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};

pub trait InstanceToPrefix {
	/// Prefix used for the Owner storage.
	type OwnerPrefix: StorageInstance;

	/// Prefix used for the ClaimableOwner storage.
	type ClaimableOwnerPrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($instance:ident, $name:literal) => {
		// Using `paste!` we generate a dedicated module to avoid collisions
		// between each instance `Approves` struct.
		paste::paste! {
			mod [<_impl_prefix_ $instance:snake>] {
				use super::*;


				pub struct Owner;

				impl StorageInstance for Owner {
					const STORAGE_PREFIX: &'static str = "Owner";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}

				pub struct ClaimableOwner;

				impl StorageInstance for ClaimableOwner {
					const STORAGE_PREFIX: &'static str = "ClaimableOwner";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}


				impl InstanceToPrefix for $instance {
					type OwnerPrefix = Owner;
					type ClaimableOwnerPrefix = ClaimableOwner;
				}
			}
		}
	};
}

// Since the macro expect a `ident` to be used with `paste!` we cannot provide `()` directly.
type Instance0 = ();

impl_prefix!(Instance0, "Erc20Instance0Balances");
impl_prefix!(Instance1, "Erc20Instance1Balances");
impl_prefix!(Instance2, "Erc20Instance2Balances");
impl_prefix!(Instance3, "Erc20Instance3Balances");
impl_prefix!(Instance4, "Erc20Instance4Balances");
impl_prefix!(Instance5, "Erc20Instance5Balances");
impl_prefix!(Instance6, "Erc20Instance6Balances");
impl_prefix!(Instance7, "Erc20Instance7Balances");
impl_prefix!(Instance8, "Erc20Instance8Balances");
impl_prefix!(Instance9, "Erc20Instance9Balances");
impl_prefix!(Instance10, "Erc20Instance10Balances");
impl_prefix!(Instance11, "Erc20Instance11Balances");
impl_prefix!(Instance12, "Erc20Instance12Balances");
impl_prefix!(Instance13, "Erc20Instance13Balances");
impl_prefix!(Instance14, "Erc20Instance14Balances");
impl_prefix!(Instance15, "Erc20Instance15Balances");
impl_prefix!(Instance16, "Erc20Instance16Balances");

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TOKEN_SUPPORT_CHANGE: [u8; 32] = keccak256!("TokenSupportChange(address)");

/// Solidity selector of the Withdraw log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_NEW_OWNER: [u8; 32] = keccak256!("NewOwner(address)");
pub const SELECTOR_LOG_TRANSFER_OWNER: [u8; 32] =
	keccak256!("OwnershipTransferStarted(address,address)");

pub type OwnerStorage<Instance, DefaultOwner> =
	StorageValue<<Instance as InstanceToPrefix>::OwnerPrefix, H160, ValueQuery, DefaultOwner>;

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}
pub type ClaimableOwnerStorage<Instance> = StorageValue<
	<Instance as InstanceToPrefix>::ClaimableOwnerPrefix,
	H160,
	ValueQuery,
	ZeroAddress,
>;

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct SupportedTokensManagerPrecompile<
	Runtime,
	UserFeeTokenController,
	DefaultOwner,
	Instance: 'static = (),
>(PhantomData<(Runtime, UserFeeTokenController, DefaultOwner, Instance)>);

#[precompile_utils::precompile]
impl<Runtime, SupportedTokensManager, DefaultOwner, Instance>
	SupportedTokensManagerPrecompile<Runtime, SupportedTokensManager, DefaultOwner, Instance>
where
	DefaultOwner: Get<H160> + 'static,
	SupportedTokensManager: pallet_supported_tokens_manager::SupportedTokensManager,
	Instance: 'static + InstanceToPrefix,
	Runtime: pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	#[precompile::public("addToken(address,bytes32)")]
	fn add_token(handle: &mut impl PrecompileHandle, token: Address, slot: H256) -> EvmResult<()> {
		let msg_sender = handle.context().caller;

		Self::require_owner(handle, msg_sender)?;

		if token == H160::zero().into() {
			return Err(revert("SupportedTokensManager: Invalid address"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		match SupportedTokensManager::add_supported_token(token.into(), slot) {
			Ok(_) => {
				handle.record_log_costs_manual(2, 32)?;
				log2(
					handle.context().address,
					SELECTOR_LOG_TOKEN_SUPPORT_CHANGE,
					Into::<H160>::into(token),
					solidity::encode_event_data(true),
				)
				.record(handle)?;
				Ok(())
			}
			Err(_) => Err(revert("SupportedTokensManager: Token is already supported")),
		}
	}

	#[precompile::public("supportedTokens()")]
	#[precompile::view]
	fn supported_tokens(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Address>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let supported_tokens = SupportedTokensManager::get_supported_tokens();

		Ok(supported_tokens.iter().map(|x| Address(*x)).collect())
	}

	#[precompile::public("isTokenSupported(address)")]
	#[precompile::view]
	fn is_token_supported(handle: &mut impl PrecompileHandle, token: Address) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(SupportedTokensManager::is_supported_token(token.into()))
	}

	#[precompile::public("removeToken(address)")]
	fn remove_token(handle: &mut impl PrecompileHandle, token: Address) -> EvmResult<()> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let msg_sender = handle.context().caller;

		Self::require_owner(handle, msg_sender)?;

		handle.record_log_costs_manual(2, 32)?;
		log2(
			handle.context().address,
			SELECTOR_LOG_TOKEN_SUPPORT_CHANGE,
			Into::<H160>::into(token),
			solidity::encode_event_data(false),
		)
		.record(handle)?;

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		match SupportedTokensManager::remove_supported_token(token.into()) {
			Ok(_) => Ok(()),
			Err(_) => Err(revert(
				"SupportedTokensManager: Token not found in supported tokens",
			)),
		}
	}

	#[precompile::public("updateDefaultToken(address)")]
	fn update_default_token(handle: &mut impl PrecompileHandle, token: Address) -> EvmResult<()> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let msg_sender = handle.context().caller;

		if token == H160::zero().into() {
			return Err(revert("SupportedTokensManager: Invalid address"));
		}

		Self::require_owner(handle, msg_sender)
			.map_err(|_| revert("SupportedTokensManager: Caller is not the owner"))?;

		SupportedTokensManager::set_default_token(token.into())
			.map_err(|_| revert("SupportedTokensManager: Target token is not supported"))?;

		Ok(())
	}

	#[precompile::public("defaultToken()")]
	fn default_token(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(SupportedTokensManager::get_default_token().into())
	}

	fn require_owner(handle: &mut impl PrecompileHandle, caller: H160) -> EvmResult<()> {
		let owner = Self::owner(handle)?;

		if caller != owner.into() {
			return Err(revert("SupportedTokensManager: Caller is not the owner"));
		}

		Ok(())
	}

	#[precompile::public("owner()")]
	#[precompile::view]
	fn owner(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(OwnerStorage::<Instance, DefaultOwner>::get().into())
	}

	#[precompile::public("pendingOwner()")]
	#[precompile::view]
	fn pending_owner(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ClaimableOwnerStorage::<Instance>::get().into())
	}

	#[precompile::public("transferOwnership(address)")]
	fn transfer_ownership(handle: &mut impl PrecompileHandle, new_owner: Address) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<Instance, DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("SupportedTokensManager: Sender is not owner"));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		ClaimableOwnerStorage::<Instance>::mutate(move |claimable_owner| {
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

		let target_new_owner = ClaimableOwnerStorage::<Instance>::get();

		if target_new_owner != msg_sender {
			return Err(revert(
				"SupportedTokensManager: Target owner is not the claimer",
			));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		ClaimableOwnerStorage::<Instance>::kill();
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		OwnerStorage::<Instance, DefaultOwner>::mutate(|owner| {
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
}
