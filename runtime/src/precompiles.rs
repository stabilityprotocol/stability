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

use core::str::FromStr;

use frame_support::parameter_types;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_fee_rewards_vault_controller::FeeRewardsVaultControllerPrecompile;
use precompile_fee_token_selector::FeeTokenPrecompile;
use precompile_supported_tokens_manager::SupportedTokensManagerPrecompile;
use precompile_upgrade_runtime_controller::UpgradeRuntimeControllerPrecompile;
use precompile_utils::precompile_set::*;
use precompile_validator_controller::ValidatorControllerPrecompile;
use precompile_validator_fee_selector::ValidatorFeeManagerPrecompile;
use sp_core::H160;

use crate::{
	stability_config::{DEFAULT_FEE_TOKEN, DEFAULT_OWNER},
	FeeController as StabilityFeeController,
};

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as foreign
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as local
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX;
	pub LocalAssetPrefix: &'static [u8] = LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX;
	pub DefaultOwner:H160 = H160::from_str(DEFAULT_OWNER).expect("invalid address");
	pub DefaultToken:H160 = H160::from_str(DEFAULT_FEE_TOKEN).expect("invalid address");
}

type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

// Set of Stability precompiles
pub type StabilityPrecompiles<R, FeeController> = PrecompileSetBuilder<
	R,
	(
		// Skip precompiles if out of range.
		PrecompilesInRangeInclusive<
			(AddressU64<1>, AddressU64<4095>),
			(
				// Ethereum precompiles:
				// We allow DELEGATECALL to stay compliant with Ethereum behavior.
				PrecompileAt<AddressU64<1>, ECRecover, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<2>, Sha256, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<3>, Ripemd160, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<4>, Identity, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<5>, Modexp, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<6>, Bn128Add, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<7>, Bn128Mul, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<8>, Bn128Pairing, EthereumPrecompilesChecks>,
				PrecompileAt<AddressU64<9>, Blake2F, EthereumPrecompilesChecks>,
				// Non-Stability specific nor Ethereum precompiles
				PrecompileAt<AddressU64<1024>, Sha3FIPS256>,
				PrecompileAt<AddressU64<1026>, ECRecoverPublicKey>,
				PrecompileAt<
					AddressU64<2049>,
					SupportedTokensManagerPrecompile<
						R,
						<FeeController as StabilityFeeController>::Token,
						DefaultOwner,
					>,
					EthereumPrecompilesChecks,
				>,
				PrecompileAt<
					AddressU64<2050>,
					ValidatorFeeManagerPrecompile<
						R,
						<FeeController as StabilityFeeController>::Validator,
						DefaultOwner,
					>,
					(), // Only from EOA
				>,
				PrecompileAt<
					AddressU64<2051>,
					FeeTokenPrecompile<R, <FeeController as StabilityFeeController>::User>,
					(), // Only from EOA
				>,
				PrecompileAt<
					AddressU64<2053>,
					ValidatorControllerPrecompile<R, DefaultOwner>,
					(), // Only from EOA
				>,
				PrecompileAt<
					AddressU64<2054>,
					UpgradeRuntimeControllerPrecompile<R, DefaultOwner>,
					EthereumPrecompilesChecks,
				>,
				PrecompileAt<
					AddressU64<2055>,
					FeeRewardsVaultControllerPrecompile<R, DefaultOwner>,
					(
						AcceptDelegateCall,
						CallableByContract,
						CallableByPrecompile,
						SubcallWithMaxNesting<1>,
					),
				>,
			),
		>,
	),
>;
