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

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::parameter_types;
use sp_runtime::traits::Dispatchable;

use frame_support::pallet_prelude::{StorageValue, ValueQuery};
use frame_support::traits::StorageInstance;

use precompile_utils::prelude::*;
use sp_core::{Get, H160, H256, U256};
use sp_std::marker::PhantomData;

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

type Instance0 = ();

impl_prefix!(Instance0, "VFSOwnable");

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

pub type ClaimableOwnerStorage<Instance> = StorageValue<
	<Instance as InstanceToPrefix>::ClaimableOwnerPrefix,
	H160,
	ValueQuery,
	ZeroAddress,
>;

/// Solidity selector of the Token Acceptance Changed log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED: [u8; 32] =
	keccak256!("ValidatorTokenAcceptanceChanged(address,address,bool)");

/// Solidity selector of the Controller Changed log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_VALIDATOR_CONTROLLER_CHANGED: [u8; 32] =
	keccak256!("ValidatorTokenRateControllerChanged(address,address)");

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
	pub DefaultAcceptance:bool = false;
	pub DefaultConversionRate:(U256, U256) = (U256::from(1), U256::from(1));
}

/// Precompile for manage tokens in which validators would accept fees to be paid on
/// Also the validators will set their conversion rate for those tokens
pub struct ValidatorFeeManagerPrecompile<
	Runtime,
	ValidatorFeeTokenController,
	DefaultOwner,
	Instance: 'static = (),
>(PhantomData<(Runtime, ValidatorFeeTokenController, DefaultOwner, Instance)>);

#[precompile_utils::precompile]
impl<Runtime, ValidatorFeeTokenController, DefaultOwner, Instance>
	ValidatorFeeManagerPrecompile<Runtime, ValidatorFeeTokenController, DefaultOwner, Instance>
where
	DefaultOwner: Get<H160> + 'static,
	ValidatorFeeTokenController: pallet_validator_fee_selector::ValidatorFeeTokenController,
	Instance: 'static + InstanceToPrefix,
	Runtime: pallet_evm::Config + pallet_timestamp::Config + pallet_validator_set::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	<Runtime as frame_system::Config>::AccountId: From<H160>,
{
	#[precompile::public("setTokenAcceptance(address,bool)")]
	fn set_token_acceptance(
		handle: &mut impl PrecompileHandle,
		token_address: Address,
		acceptance_value: bool,
	) -> EvmResult {
		let msg_sender = handle.context().caller;

		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let validators = pallet_validator_set::Pallet::<Runtime>::approved_validators();

		if !validators.contains(&msg_sender.into()) {
			return Err(revert(
				"ValidatorFeeTokenController: sender is not an approved validator",
			));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		ValidatorFeeTokenController::update_fee_token_acceptance(
			msg_sender,
			token_address.into(),
			acceptance_value,
		)
		.map_err(|_| revert("ValidatorFeeTokenController: token not supported"))?;

		handle.record_log_costs_manual(3, 32)?;

		handle.record_log_costs_manual(3, 32)?;
		log3(
			handle.context().address,
			SELECTOR_LOG_VALIDATOR_TOKEN_ACCEPTANCE_CHANGED,
			msg_sender,
			Into::<H160>::into(token_address),
			solidity::encode_event_data(acceptance_value),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("validatorSupportsToken(address,address)")]
	#[precompile::view]
	fn validator_supports_token(
		handle: &mut impl PrecompileHandle,
		validator: Address,
		token_address: Address,
	) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		Ok(ValidatorFeeTokenController::validator_supports_fee_token(
			validator.into(),
			token_address.into(),
		))
	}

	#[precompile::public("updateConversionRateController(address)")]
	fn update_conversion_rate_controller(
		handle: &mut impl PrecompileHandle,
		cr_controller: Address,
	) -> EvmResult {
		let msg_sender = handle.context().caller;

		let validators = pallet_validator_set::Pallet::<Runtime>::approved_validators();

		if !validators.contains(&msg_sender.into()) {
			return Err(revert(
				"ValidatorFeeTokenController: sender is not an approved validator",
			));
		}

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		ValidatorFeeTokenController::update_conversion_rate_controller(
			msg_sender,
			cr_controller.into(),
		)
		.map_err(|_| {
			revert("ValidatorFeeTokenController: default token conversion rate cannot be updated")
		})?;

		handle.record_log_costs_manual(2, 64)?;
		log2(
			handle.context().address,
			SELECTOR_LOG_VALIDATOR_CONTROLLER_CHANGED,
			msg_sender,
			solidity::encode_event_data(cr_controller),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("conversionRateController(address)")]
	#[precompile::view]
	fn conversion_rate_controller(
		handle: &mut impl PrecompileHandle,
		validator: Address,
	) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		Ok(ValidatorFeeTokenController::conversion_rate_controller(validator.into()).into())
	}

	#[precompile::public("updateDefaultController(address)")]
	fn update_default_controller(
		handle: &mut impl PrecompileHandle,
		controller: Address,
	) -> EvmResult<()> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		let msg_sender = handle.context().caller;

		if Self::owner(handle)? != Address(msg_sender) {
			return Err(revert(
				"ValidatorFeeTokenController: sender is not the owner",
			));
		}

		ValidatorFeeTokenController::update_default_controller(controller.into()).map_err(
			|_| revert("ValidatorFeeTokenController: failed to update default controller"),
		)?;
		Ok(())
	}

	#[precompile::public("defaultController()")]
	#[precompile::view]
	fn default_controller(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ValidatorFeeTokenController::conversion_rate_controller(H160::zero()).into())
	}

	#[precompile::public("owner()")]
	#[precompile::view]
	fn owner(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(OwnerStorage::<Instance, DefaultOwner>::get().into())
	}

	#[precompile::public("pendingOwner()")]
	#[precompile::view]
	fn pending_owner(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(ClaimableOwnerStorage::<Instance>::get().into())
	}

	#[precompile::public("transferOwnership(address)")]
	fn transfer_ownership(handle: &mut impl PrecompileHandle, new_owner: Address) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let msg_sender = handle.context().caller;
		let owner = OwnerStorage::<Instance, DefaultOwner>::get();

		if msg_sender != owner {
			return Err(revert("ValidatorFeeTokenController: Sender is not owner"));
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
				"ValidatorFeeTokenController: Target owner is not the claimer",
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
