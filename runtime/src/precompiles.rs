use core::str::FromStr;

use frame_support::parameter_types;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use precompile_fee_token_selector::FeeTokenPrecompile;
use precompile_map_svm_evm_controller::MapSvmEvmControllerPrecompile;
use precompile_supported_tokens_manager::SupportedTokensManagerPrecompile;
use precompile_utils::precompile_set::*;
use precompile_validator_fee_selector::ValidatorFeeManagerPrecompile;
use precompile_validator_controller::ValidatorControllerPrecompile;
use precompile_fee_rewards_vault_controller::FeeRewardsVaultControllerPrecompile;
use precompile_upgrade_runtime_controller::UpgradeRuntimeControllerPrecompile;
use sp_core::H160;

use crate::{
	stability_config::{DEFAULT_FEE_TOKEN, DEFAULT_OWNER},
	FeeController as StabilityFeeController,
};

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"DOLR"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"DOLR"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

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
				PrecompileAt<AddressU64<1>, ECRecover, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<2>, Sha256, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<3>, Ripemd160, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<4>, Identity, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<5>, Modexp, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<6>, Bn128Add, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<7>, Bn128Mul, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<8>, Bn128Pairing, ForbidRecursion, AllowDelegateCall>,
				PrecompileAt<AddressU64<9>, Blake2F, ForbidRecursion, AllowDelegateCall>,
				// Non-Stability specific nor Ethereum precompiles :
				PrecompileAt<AddressU64<1024>, Sha3FIPS256>,
				PrecompileAt<AddressU64<1026>, ECRecoverPublicKey>,
				// Stability specific precompiles:
				PrecompileAt<
					AddressU64<2048>,
					Erc20BalancesPrecompile<R, NativeErc20Metadata, DefaultOwner>,
				>,
				PrecompileAt<
					AddressU64<2049>,
					SupportedTokensManagerPrecompile<
						R,
						<FeeController as StabilityFeeController>::Token,
						DefaultOwner,
					>,
				>,
				PrecompileAt<
					AddressU64<2050>,
					ValidatorFeeManagerPrecompile<
						R,
						<FeeController as StabilityFeeController>::Validator,
					>,
				>,
				PrecompileAt<
					AddressU64<2051>,
					FeeTokenPrecompile<R, <FeeController as StabilityFeeController>::User>,
				>,
				PrecompileAt<AddressU64<2052>, MapSvmEvmControllerPrecompile<R>>,
				PrecompileAt<AddressU64<2053>, ValidatorControllerPrecompile<R, DefaultOwner>>,
				PrecompileAt<AddressU64<2054>, UpgradeRuntimeControllerPrecompile<R, DefaultOwner>>,
				PrecompileAt<AddressU64<2055>, FeeRewardsVaultControllerPrecompile<R, DefaultOwner>>

			),
		>,
	),
>;
