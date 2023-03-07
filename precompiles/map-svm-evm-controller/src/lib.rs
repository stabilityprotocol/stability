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

use codec::Encode;

use sp_core::{H256, U256};

use precompile_utils::prelude::*;

use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use sp_std::marker::PhantomData;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_ACCOUNT_UNLINKED: [u8; 32] = keccak256!("AccountUnlinked(address,bytes32)");

pub struct MapSvmEvmControllerPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> MapSvmEvmControllerPrecompile<Runtime>
where
	Runtime: pallet_map_svm_evm::Config + pallet_timestamp::Config + pallet_evm::Config,
	Runtime::RuntimeCall: From<pallet_map_svm_evm::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	#[precompile::public("linkOf(address)")]
	#[precompile::view]
	fn link_of(handle: &mut impl PrecompileHandle, address: Address) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let substrate_account =
			pallet_map_svm_evm::Pallet::<Runtime>::get_linked_substrate_account(address.into())
				.ok_or(revert("EVM account not linked"))?;

		Ok(H256::from_slice(substrate_account.encode().as_slice()))
	}

	#[precompile::public("unLink()")]
	fn un_link(handle: &mut impl PrecompileHandle) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		let sender = handle.context().caller;

		let account_id =
			pallet_map_svm_evm::Pallet::<Runtime>::unlink_account_from_evm_account(sender)
				.map_err(|_| revert("Unlink failed"))?;

		log2(
			handle.context().address,
			SELECTOR_LOG_ACCOUNT_UNLINKED,
			handle.context().caller,
			EvmDataWriter::new()
				.write(H256::from_slice(account_id.encode().as_slice()))
				.build(),
		)
		.record(handle)?;

		Ok(true)
	}
}
