// Copyright 2019-2022 Stability Solutions.
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

//! Precompile to interact with pallet_balances instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use core::str::FromStr;

use fp_evm::PrecompileHandle;
use frame_support::parameter_types;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	storage::types::{StorageMap, ValueQuery},
	traits::StorageInstance,
	Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};

use precompile_utils::prelude::*;
use sp_core::{Get, H256};
use sp_core::{H160, U256};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_FEE_CHANGED: [u8; 32] = keccak256!("FeeTokenChanged(address,address)");

/// Associates pallet Instance to a prefix used for the Approves storage.
/// This trait is implemented for () and the 16 substrate Instance.
pub trait FeePrefixes {
	/// Prefix used for the Approves storage.
	type FeeTokenPrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($instance:ident, $name:literal) => {
		// Using `paste!` we generate a dedicated module to avoid collisions
		// between each instance `Approves` struct.
		paste::paste! {
			mod [<_impl_prefix_ $instance:snake>] {
				use super::*;

				pub struct FeeToken;

				impl StorageInstance for FeeToken {
					const STORAGE_PREFIX: &'static str = "FeeToken";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}


				impl FeePrefixes for $instance {
					type FeeTokenPrefix = FeeToken;
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

parameter_types! {
	pub ZeroAddress:H160 = H160::from_str("0x0000000000000000000000000000000000000000").expect("invalid address");
}

/// Storage type used to store EIP2612 nonces.
pub type FeeTokenStorage<Instance, DefaultFeeToken> = StorageMap<
	<Instance as FeePrefixes>::FeeTokenPrefix,
	// Owner
	Blake2_128Concat,
	H160,
	// Nonce
	H160,
	ValueQuery,
	DefaultFeeToken,
>;

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct FeeTokenPrecompile<Runtime, DefaultToken: Get<H160> + 'static, Instance: 'static = ()>(
	PhantomData<(Runtime, DefaultToken, Instance)>,
);

#[precompile_utils::precompile]
impl<Runtime, DefaultFeeToken, Instance> FeeTokenPrecompile<Runtime, DefaultFeeToken, Instance>
where
	DefaultFeeToken: Get<H160> + 'static,
	Instance: FeePrefixes + 'static,
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	#[precompile::public("setFeeToken(address)")]
	fn set_fee_token(handle: &mut impl PrecompileHandle, token_address: Address) -> EvmResult {
		let msg_sender = handle.context().caller;

		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		handle.record_log_costs_manual(2, 32)?;

		FeeTokenStorage::<Instance, DefaultFeeToken>::mutate(msg_sender, |token| {
			*token = token_address.into();
		});

		log2(
			handle.context().address,
			SELECTOR_LOG_FEE_CHANGED,
			msg_sender,
			EvmDataWriter::new()
				.write::<H256>(Into::<H160>::into(token_address).into())
				.build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("getFeeToken(address)")]
	#[precompile::view]
	fn get_fee_token(handle: &mut impl PrecompileHandle, address: Address) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let token_address: H160 =
			FeeTokenStorage::<Instance, DefaultFeeToken>::get::<H160>(address.into());

		Ok(Into::<H256>::into(token_address))
	}
}
