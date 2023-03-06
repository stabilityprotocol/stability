#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use evm::ExitReason;
use fp_evm::{Context, ExitRevert, PrecompileFailure, PrecompileHandle, Transfer};
use frame_support::{
	ensure,
	storage::types::{StorageMap, ValueQuery},
	traits::{ConstU32, Get, StorageInstance},
	Blake2_128Concat,
};
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, H256, U256};
use sp_io::hashing::keccak_256;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/* 
DOMAIN:

"Delegated Transaction": a meta transaction created by a delegator, and executed by a delegate.
"Delegator": the user who creates a delegated transaction.
"Delegate": the user who executes a delegated transaction.
 */

/// Storage prefix for nonces.
pub struct Nonces;

impl StorageInstance for Nonces {
	const STORAGE_PREFIX: &'static str = "Nonces";

	fn pallet_prefix() -> &'static str {
		"PrecompileDelegatedTransaction"
	}
}

/// Storage type used to store EIP2612 nonces.
pub type NoncesStorage = StorageMap<
	Nonces,
	// From
	Blake2_128Concat,
	H160,
	// Nonce
	U256,
	ValueQuery,
>;

/// EIP712 tx typehash.
pub const DELEGATED_TRANSACTION_TYPEHASH: [u8; 32] = keccak256!(
	"DelegatedTransaction(address from,address to,uint256 value,bytes data,uint64 gaslimit\
,uint256 nonce,uint256 deadline)"
);

/// EIP712 delegated transaction domain used to compute an individualized domain separator.
const DELEGATED_TRANSACTION_DOMAIN: [u8; 32] = keccak256!(
	"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

/// Precompile allowing to issue and dispatch delegated transactions for gasless transactions.
/// A user (delegator) can sign a transaction for a call that can be dispatched and paid for by another account (delegate).
pub struct DelegatedTransactionPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> DelegatedTransactionPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + pallet_timestamp::Config,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	fn compute_domain_separator(address: H160) -> [u8; 32] {
		let name: H256 = keccak_256(b"Delegated Transaction Precompile").into();
		let version: H256 = keccak256!("1").into();
		let chain_id: U256 = Runtime::ChainId::get().into();

		let domain_separator_inner = EvmDataWriter::new()
			.write(H256::from(DELEGATED_TRANSACTION_DOMAIN))
			.write(name)
			.write(version)
			.write(chain_id)
			.write(Address(address))
			.build();

		keccak_256(&domain_separator_inner).into()
	}

	pub fn generate_delegated_transaction(
		address: H160,
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gaslimit: u64,
		nonce: U256,
		deadline: U256,
	) -> [u8; 32] {
		let domain_separator = Self::compute_domain_separator(address);

		let delegated_transaction_content = EvmDataWriter::new()
			.write(H256::from(DELEGATED_TRANSACTION_TYPEHASH))
			.write(Address(from))
			.write(Address(to))
			.write(value)
			// bytes are encoded as the keccak_256 of the content
			.write(H256::from(keccak_256(&data)))
			.write(gaslimit)
			.write(nonce)
			.write(deadline)
			.build();
		let delegated_transaction_content = keccak_256(&delegated_transaction_content);
		let mut pre_digest = Vec::with_capacity(2 + 32 + 32);
		pre_digest.extend_from_slice(b"\x19\x01");
		pre_digest.extend_from_slice(&domain_separator);
		pre_digest.extend_from_slice(&delegated_transaction_content);
		keccak_256(&pre_digest)
	}

	pub fn dispatch_inherent_cost() -> u64 {
		3_000 // cost of ECRecover precompile for reference
			+ RuntimeHelper::<Runtime>::db_read_gas_cost() * 2 // we read nonce and timestamp
			+ RuntimeHelper::<Runtime>::db_write_gas_cost() // we write nonce
	}

	#[precompile::public(
		"dispatch(address,address,uint256,bytes,uint64,uint256,uint8,bytes32,bytes32)"
	)]
	fn dispatch(
		handle: &mut impl PrecompileHandle,
		from: Address,
		to: Address,
		value: U256,
		data: BoundedBytes<ConstU32<CALL_DATA_LIMIT>>,
		gas_limit: u64,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(Self::dispatch_inherent_cost())?;

		let from: H160 = from.into();
		let to: H160 = to.into();
		let data: Vec<u8> = data.into();

		// ENSURE GASLIMIT IS SUFFICIENT
		let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

		let total_cost = gas_limit
			.checked_add(call_cost)
			.ok_or_else(|| revert("Call requires too much gas (uint64 overflow)"))?;

		if total_cost > handle.remaining_gas() {
			return Err(revert("Gaslimit is too low to dispatch provided call"));
		}

		// VERIFY DELEGATED TRANSACTION

		// pallet_timestamp is in ms while Ethereum use second timestamps.
		let timestamp: U256 = (pallet_timestamp::Pallet::<Runtime>::get()).into() / 1000;
		ensure!(deadline >= timestamp, revert("Delegated transaction expired"));

		let nonce = NoncesStorage::get(from);

		let delegated_transaction = Self::generate_delegated_transaction(
			handle.context().address,
			from,
			to,
			value,
			data.clone(),
			gas_limit,
			nonce,
			deadline,
		);

		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(&r.as_bytes());
		sig[32..64].copy_from_slice(&s.as_bytes());
		sig[64] = v;

		let delegate = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &delegated_transaction)
			.map_err(|_| revert("Invalid delegated transaction"))?;
		let delegate = H160::from(H256::from_slice(keccak_256(&delegate).as_slice()));

		ensure!(
			delegate != H160::zero() && delegate == from,
			revert("Invalid delegated transaction")
		);

		NoncesStorage::insert(from, nonce + U256::one());

		// DISPATCH CALL
		let sub_context = Context {
			caller: from,
			address: to.clone(),
			apparent_value: value,
		};

		let transfer = if value.is_zero() {
			None
		} else {
			Some(Transfer {
				source: from,
				target: to.clone(),
				value,
			})
		};

		let (reason, output) =
			handle.call(to, transfer, data, Some(gas_limit), false, &sub_context);
		match reason {
			ExitReason::Error(exit_status) => Err(PrecompileFailure::Error { exit_status }),
			ExitReason::Fatal(exit_status) => Err(PrecompileFailure::Fatal { exit_status }),
			ExitReason::Revert(_) => Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output,
			}),
			ExitReason::Succeed(_) => Ok(output.into()),
		}
	}

	#[precompile::public("nonces(address)")]
	#[precompile::view]
	fn nonces(handle: &mut impl PrecompileHandle, owner: Address) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let owner: H160 = owner.into();

		let nonce = NoncesStorage::get(owner);

		Ok(nonce)
	}

	#[precompile::public("DOMAIN_SEPARATOR()")]
	#[precompile::view]
	fn domain_separator(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let domain_separator: H256 =
			Self::compute_domain_separator(handle.context().address).into();

		Ok(domain_separator)
	}
}