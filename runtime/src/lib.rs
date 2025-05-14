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

// Please see the Stability Open License Agreement for more
// information.

//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::new_without_default, clippy::or_fun_call)]
#![cfg_attr(feature = "runtime-benchmarks", deny(unused_crate_dependencies))]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use account::AccountId20;
use account::EthereumSigner;
use core::str::FromStr;
#[cfg(feature = "std")]
pub use fp_evm::GenesisAccount;
use frame_support::pallet_prelude::EnsureOrigin;
use frame_support::pallet_prelude::InvalidTransaction;
use frame_system::EnsureRoot;
use frame_system::RawOrigin;
use opaque::SessionKeys;
use pallet_balances::Instance1;
use pallet_custom_balances::AccountIdMapping;
use pallet_supported_tokens_manager::SupportedTokensManager as OtherSupportedTokensManager;
use pallet_transaction_payment::OnChargeTransaction;
use pallet_user_fee_selector::UserFeeTokenController;
use pallet_validator_fee_selector::ValidatorFeeTokenController;
use parity_scale_codec::{Decode, Encode};
use runner::OnChargeDecentralizedNativeTokenFee;
use sp_api::impl_runtime_apis;
use sp_core::{
	crypto::{ByteArray, KeyTypeId},
	OpaqueMetadata, H160, H256, U256,
};
use sp_genesis_builder::PresetId;
use sp_runtime::traits::Convert;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::IdentityLookup;
use sp_runtime::{
	create_runtime_str, generic,
	generic::Era,
	impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, Get, NumberFor, OpaqueKeys,
		PostDispatchInfoOf, UniqueSaturatedInto, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
	ApplyExtrinsicResult, Permill, SaturatedConversion,
};
use sp_std::{marker::PhantomData, prelude::*};
use sp_version::RuntimeVersion;
use stability_config::{MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO};
use stbl_core_primitives::aura::Public as AuraId;
use stbl_tools::custom_fee::CustomFeeInfo;
use stbl_transaction_validator::FallbackTransactionValidator;
// Substrate FRAME
#[cfg(feature = "with-paritydb-weights")]
use frame_support::weights::constants::ParityDbWeight as RuntimeDbWeight;
#[cfg(feature = "with-rocksdb-weights")]
use frame_support::weights::constants::RocksDbWeight as RuntimeDbWeight;
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
// Frontier
use fp_rpc::TransactionStatus;
use pallet_ethereum::{Call::transact, PostLogContent, Transaction as EthereumTransaction};
use pallet_evm::{Account as EVMAccount, FeeCalculator, GasWeightMapping, Runner};
use pallet_sponsored_transactions::Call::send_sponsored_transaction;
use pallet_validator_set::SessionBlockManager;
extern crate moonbeam_rpc_primitives_txpool;
// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{
		ConstBool, ConstU32, ConstU8, EitherOfDiverse, FindAuthor, KeyOwnerProofSystem, OnFinalize,
		OnTimestampSet, Randomness,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight},
		ConstantMultiplier, IdentityFee, Weight,
	},
	ConsensusEngineId, StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_timestamp::Call as TimestampCall;

use pallet_user_fee_selector;

mod stability_config;
use stability_config::{
	COUNCIL_MAX_MEMBERS, COUNCIL_MAX_PROPOSALS, COUNCIL_MOTION_MINUTES_DURATION,
	DEFAULT_ELASTICITY, DEFAULT_FEE_TOKEN, EXISTENTIAL_DEPOSIT, GAS_BASE_FEE, MAXIMUM_BLOCK_LENGTH,
	MILLISECS_PER_BLOCK, SESSION_MINUTES_DURATION, VALIDATOR_SET_MIN_VALIDATORS,
};

mod precompiles;
use precompiles::StabilityPrecompiles;

pub trait FeeController {
	type Token: pallet_supported_tokens_manager::SupportedTokensManager;
	type Validator: pallet_validator_fee_selector::ValidatorFeeTokenController;
	type User: pallet_user_fee_selector::UserFeeTokenController;
}

pub struct StabilityFeeController;

impl FeeController for StabilityFeeController {
	type Token = pallet_supported_tokens_manager::Pallet<Runtime>;
	type Validator = pallet_validator_fee_selector::Pallet<Runtime>;
	type User = pallet_user_fee_selector::Pallet<Runtime>;
}

pub type Precompiles = StabilityPrecompiles<Runtime, StabilityFeeController>;

use runner::Runner as StabilityRunner;

/// Type of block number.
pub type BlockNumber = stbl_core_primitives::BlockNumber;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = stbl_core_primitives::Signature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = stbl_core_primitives::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = stbl_core_primitives::AccountIndex;

/// Balance of an account.
pub type Balance = stbl_core_primitives::Balance;

/// Index of a transaction in the chain.
pub type Index = stbl_core_primitives::Index;

pub type Nonce = Index;

/// A hash of some data used by the chain.
pub type Hash = stbl_core_primitives::Hash;

/// Digest item type.
pub type DigestItem = stbl_core_primitives::DigestItem;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node-stability"),
	impl_name: create_runtime_str!("node-stability"),
	authoring_version: 1,
	spec_version: 5,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
	sp_version::NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 256;
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max(MAXIMUM_BLOCK_LENGTH);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The index type for blocks.
	type Block = Block;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The aggregated RuntimeTask type.
	type RuntimeTask = RuntimeTask;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RuntimeDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const MaxAuthorities: u32 = 100;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = MaxAuthorities;
	type DisabledValidators = ();
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = frame_support::traits::ConstU64<SLOT_DURATION>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxSetIdSessionEntries = ();
	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
	type MaxNominators = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	pub storage EnableManualSeal: bool = false;
}

pub struct ConsensusOnTimestampSet<T>(PhantomData<T>);
impl<T: pallet_aura::Config> OnTimestampSet<T::Moment> for ConsensusOnTimestampSet<T> {
	fn on_timestamp_set(moment: T::Moment) {
		if EnableManualSeal::get() {
			return;
		}
		<pallet_aura::Pallet<T> as OnTimestampSet<T::Moment>>::on_timestamp_set(moment)
	}
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ConsensusOnTimestampSet<Self>;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

pub struct StbleOnChargeTransaction;
impl<T: pallet_transaction_payment::Config> OnChargeTransaction<T> for StbleOnChargeTransaction
where
	account::AccountId20: From<T::AccountId>,
{
	type Balance = Balance;
	type LiquidityInfo = Balance;

	fn withdraw_fee(
		who: &T::AccountId,
		_call: &T::RuntimeCall,
		_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		_fee: Self::Balance,
		_tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		let from: H160 = Into::<AccountId20>::into((*who).clone()).into();
		let token = DNTFeeController::get_transaction_fee_token(from);
		let validator = EVM::find_author();
		let conversion_rate =
			DNTFeeController::get_transaction_conversion_rate(from, validator, token);
		let fee = U256::from(_fee.saturated_into::<u128>());

		DNTFeeController::withdraw_fee(from, token, conversion_rate, fee).map_err(|_x| {
			TransactionValidityError::Invalid(
				frame_support::pallet_prelude::InvalidTransaction::Payment,
			)
		})?;

		Ok(_fee)
	}

	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		_corrected_fee: Self::Balance,
		_tip: Self::Balance,
		_already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		let from: H160 = Into::<AccountId20>::into((*who).clone()).into();
		let corrected_fee = U256::from(_corrected_fee.saturated_into::<u128>());
		let already_withdrawn = U256::from(_already_withdrawn.saturated_into::<u128>());
		let validator = EVM::find_author();

		let token = DNTFeeController::get_transaction_fee_token(from);
		let conversion_rate =
			DNTFeeController::get_transaction_conversion_rate(from, validator, token);

		DNTFeeController::correct_fee(
			from,
			token,
			conversion_rate,
			already_withdrawn,
			corrected_fee,
		)
		.map_err(|_x| {
			TransactionValidityError::Invalid(
				frame_support::pallet_prelude::InvalidTransaction::Payment,
			)
		})?;

		DNTFeeController::pay_fees(token, conversion_rate, corrected_fee, validator, None)
			.map_err(|_x| {
				TransactionValidityError::Invalid(
					frame_support::pallet_prelude::InvalidTransaction::Payment,
				)
			})?;

		Ok(())
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = StbleOnChargeTransaction;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = ();
}

impl pallet_evm_chain_id::Config for Runtime {}

pub struct FindAuthorLinkedOrTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorLinkedOrTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id =
				pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();

			let bytes: [u8; 33] = authority_id.as_slice().try_into().unwrap();
			let signer: EthereumSigner = sp_core::ecdsa::Public::from(bytes).into();
			return Some(signer.into_account().into());
		}
		None
	}
}

pub struct FindBlockAuthorityId<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<AccountId> for FindBlockAuthorityId<F> {
	fn find_author<'a, I>(digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id =
				pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();

			let bytes: [u8; 33] = authority_id.as_slice().try_into().unwrap();
			let signer: EthereumSigner = sp_core::ecdsa::Public::from(bytes).into();
			return Some(signer.into_account().into());
		}
		None
	}
}

pub struct IdentityAddressMapping;
impl pallet_evm::AddressMapping<AccountId> for IdentityAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		address.into()
	}
}

pub struct AccountIdToH160Mapping;
impl pallet_custom_balances::AccountIdMapping<Runtime> for AccountIdToH160Mapping {
	fn into_evm_address(address: &AccountId) -> H160 {
		(*address).into()
	}
}

impl pallet_custom_balances::Config for Runtime {
	type AccountIdMapping = AccountIdToH160Mapping;
	type UserFeeTokenController = <StabilityFeeController as FeeController>::User;
}

pub struct EnsureAddressLinkedOrTruncated;

impl<OuterOrigin> pallet_evm::EnsureAddressOrigin<OuterOrigin> for EnsureAddressLinkedOrTruncated
where
	OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>> + From<RawOrigin<AccountId>>,
{
	type Success = AccountId;

	fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId, OuterOrigin> {
		origin.into().and_then(|o| match o {
			RawOrigin::Signed(who) if Into::<H160>::into(who).eq(address) => Ok(who),
			r => Err(OuterOrigin::from(r)),
		})
	}
}

const WEIGHT_PER_GAS: u64 = 20_000;

parameter_types! {
	pub PrecompilesValue: StabilityPrecompiles<Runtime, StabilityFeeController> = StabilityPrecompiles::<_, StabilityFeeController>::new();
	pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
	pub const GasLimitPovSizeRatio: u64 = 4;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = BaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressLinkedOrTruncated;
	type WithdrawOrigin = EnsureAddressLinkedOrTruncated;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = StabilityPrecompiles<Self, StabilityFeeController>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EVMChainId;
	type BlockGasLimit = BlockGasLimit;
	type Runner = StabilityRunner<Self, DNTFeeController, pallet_user_fee_selector::Pallet<Self>>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type FindAuthor = FindAuthorLinkedOrTruncated<Aura>;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
	type SuicideQuickClearLimit = ConstU32<0>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

impl pallet_dnt_fee_controller::Config for Runtime {
	type ERC20Manager = pallet_erc20_manager::Pallet<Self>;
	type UserFeeTokenController = pallet_user_fee_selector::Pallet<Self>;
	type ValidatorTokenController = pallet_validator_fee_selector::Pallet<Self>;
}

impl pallet_erc20_manager::Config for Runtime {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
}

parameter_types! {
	pub DefaultFeeToken: H160 = H160::from_str(DEFAULT_FEE_TOKEN).expect("invalid address");
}
impl pallet_user_fee_selector::Config for Runtime {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
	type ERC20Manager = pallet_erc20_manager::Pallet<Self>;
}
impl pallet_validator_fee_selector::Config for Runtime {
	type SupportedTokensManager = pallet_supported_tokens_manager::Pallet<Self>;
	type SimulatorRunner = pallet_evm::runner::stack::Runner<Self>;
}

impl pallet_supported_tokens_manager::Config for Runtime {}

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = U256::from(GAS_BASE_FEE);
	pub DefaultElasticity: Permill = DEFAULT_ELASTICITY;
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = BaseFeeThreshold;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

impl pallet_hotfix_sufficients::Config for Runtime {
	type AddressMapping = IdentityAddressMapping;
	type WeightInfo = pallet_hotfix_sufficients::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = COUNCIL_MOTION_MINUTES_DURATION * MINUTES;
	pub const CouncilMaxProposals: u32 = COUNCIL_MAX_PROPOSALS;
	pub const CouncilMaxMembers: u32 = COUNCIL_MAX_MEMBERS;
	pub const MaxProposalWeight: Weight = MAXIMUM_BLOCK_WEIGHT;
}

type TechCommitteeInstance = pallet_collective::Instance1;

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Self>;
	type SetMembersOrigin = EnsureRootOrHalfTechCommittee;
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_root_controller::Config for Runtime {
	type ControlOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const MinAuthorities: u32 = VALIDATOR_SET_MIN_VALIDATORS;
}

type EnsureRootOrHalfTechCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>,
>;

pub struct PeriodicSessionBlockManager;
impl SessionBlockManager<BlockNumber> for PeriodicSessionBlockManager {
	fn session_start_block(session_index: sp_staking::SessionIndex) -> BlockNumber {
		return session_index * Period::get() + Offset::get();
	}
}

pub struct AccountIdOfValidator;
impl Convert<AuraId, AccountId> for AccountIdOfValidator {
	fn convert(authority_id: AuraId) -> AccountId {
		let bytes: [u8; 33] = authority_id.as_slice().try_into().unwrap();
		let signer: EthereumSigner = sp_core::ecdsa::Public::from(bytes).into();
		return signer.into_account().into();
	}
}

impl pallet_validator_set::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddRemoveOrigin = EnsureRootOrHalfTechCommittee;
	type MinAuthorities = MinAuthorities;
	type SessionBlockManager = PeriodicSessionBlockManager;
	type FindAuthor = FindBlockAuthorityId<Aura>;
	type AuthorityId = AuraId;
	type MaxKeys = MaxKeys;
	type AccountIdOfValidator = AccountIdOfValidator;
}

pub struct SessionKeysBuilder;
impl pallet_validator_keys_controller::SessionKeysBuilder<AuraId, GrandpaId, SessionKeys>
	for SessionKeysBuilder
{
	fn new(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
		SessionKeys { aura, grandpa }
	}
}
pub struct ValidatorIdMapping;
impl Convert<AuraId, AccountId20> for ValidatorIdMapping {
	fn convert(a: AuraId) -> AccountId20 {
		AccountIdOfValidator::convert(a.clone())
	}
}
impl pallet_validator_keys_controller::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type FinalizationId = GrandpaId;
	type SessionKeysBuilder = SessionKeysBuilder;
	type ValidatorIdOfValidation = ValidatorIdMapping;
}

parameter_types! {
	pub const Period: u32 = SESSION_MINUTES_DURATION * MINUTES;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_validator_set::ValidatorOf<Self>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = ValidatorSet;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		let tip = 0;
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			.saturating_sub(1);
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			stbl_transaction_validator::check_nonce::StbleCheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			frame_metadata_hash_extension::CheckMetadataHash::<Runtime>::new(false),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = account;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature.into(), extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

parameter_types! {
	pub const MaxSizeOfCode: u32 = 10 * 1024 * 1024; // 10 MB
}

pub struct EnsureMemberOfTechCollective<AccountId>(sp_std::marker::PhantomData<AccountId>);
impl<
		O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>,
		AccountId: core::clone::Clone,
	> EnsureOrigin<O> for EnsureMemberOfTechCollective<AccountId>
where
	account::AccountId20: From<AccountId>,
{
	type Success = ();
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(account) => {
				if <pallet_collective::Pallet<Runtime, Instance1>>::is_member(
					&account.clone().into(),
				) {
					Ok(())
				} else {
					Err(O::from(RawOrigin::Signed(account.clone())))
				}
			}
			r => Err(O::from(r)),
		})
	}
}

type EnsureRootOrMemberOfTechCollective =
	EitherOfDiverse<EnsureRoot<AccountId>, EnsureMemberOfTechCollective<AccountId>>;

impl pallet_upgrade_runtime_proposal::Config for Runtime {
	type ControlOrigin = EnsureRootOrMemberOfTechCollective;
	type MaxSizeOfCode = MaxSizeOfCode;
}

impl pallet_fee_rewards_vault::Config for Runtime {}

#[frame_support::pallet]
pub mod pallet_manual_seal {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T> {
		pub enable: bool,
		#[serde(skip)]
		pub _config: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			EnableManualSeal::set(&self.enable);
		}
	}
}

impl pallet_manual_seal::Config for Runtime {}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		ValidatorSet: pallet_validator_set,
		ValidatorKeysController: pallet_validator_keys_controller,
		Session: pallet_session,
		Aura: pallet_aura,
		Grandpa: pallet_grandpa,
		Balances: pallet_custom_balances,
		TransactionPayment: pallet_transaction_payment,
		TechCommitteeCollective: pallet_collective::<Instance1>,
		RootController: pallet_root_controller,
		Ethereum: pallet_ethereum,
		EVM: pallet_evm,
		EVMChainId: pallet_evm_chain_id,
		BaseFee: pallet_base_fee,
		HotfixSufficients: pallet_hotfix_sufficients,
		UserFeeSelector: pallet_user_fee_selector,
		ValidatorFeeSelector: pallet_validator_fee_selector,
		SupportedTokensManager: pallet_supported_tokens_manager,
		ERC20Manager: pallet_erc20_manager,
		DNTFeeController: pallet_dnt_fee_controller,
		UpgradeRuntimeProposal: pallet_upgrade_runtime_proposal,
		FeeRewardsVault: pallet_fee_rewards_vault,
		MetaTransactions: pallet_sponsored_transactions,
		ZeroGasTransactions: pallet_zero_gas_transactions,
		ManualSeal: pallet_manual_seal,
	}
);

#[derive(Clone)]
pub struct TransactionConverter<B>(PhantomData<B>);

impl<B> Default for TransactionConverter<B> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<B: BlockT> fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> for TransactionConverter<B> {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> <B as BlockT>::Extrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		<B as BlockT>::Extrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

/// The address format for describing accounts.
pub type Address = stbl_core_primitives::Address;
/// Block header type as expected by this runtime.
pub type Header = stbl_core_primitives::Header;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	stbl_transaction_validator::check_nonce::StbleCheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
	fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call
				.validate_self_contained(info, dispatch_info, len)
				.map(|result| match result {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Payment)) => {
						FallbackTransactionValidator::check_actual_balance(info, call)
					}
					_ => result,
				}),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call
				.pre_dispatch_self_contained(info, dispatch_info, len)
				.map(|result| match result {
					Err(TransactionValidityError::Invalid(InvalidTransaction::Payment)) => {
						FallbackTransactionValidator::check_actual_balance(info, call).map(|_| ())
					}
					_ => result,
				}),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				)))
			}
			_ => None,
		}
	}
}

impl pallet_sponsored_transactions::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type ERC20Manager = ERC20Manager;
	type DNTFeeController = DNTFeeController;
}

impl pallet_zero_gas_transactions::Config for Runtime {
	type RuntimeCall = RuntimeCall;
}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);

	pub BlockGasLimit : U256 = {
		let max_normal_block_usage = BlockWeights::get().get(DispatchClass::Normal).max_total.expect("invalid max_extrinsic").ref_time();
		U256::from(max_normal_block_usage / WEIGHT_PER_GAS)
	};

}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!([pallet_evm, EVM]);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().to_vec()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			EVMAccount { nonce: pallet_evm::Pallet::<Runtime>::account_basic(&address).0.nonce, balance: <StabilityFeeController as FeeController>::User::balance_of(address) }
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());

			// Estimated encoded transaction size must be based on the heaviest transaction
			// type (EIP1559Transaction) to be compatible with all transaction types.
			let mut estimated_transaction_len = data.len() +
			// pallet ethereum index: 1
			// transact call index: 1
			// Transaction enum variant: 1
			// chain_id 8 bytes
			// nonce: 32
			// max_priority_fee_per_gas: 32
			// max_fee_per_gas: 32
			// gas_limit: 32
			// action: 21 (enum varianrt + call address)
			// value: 32
			// access_list: 1 (empty vec size)
			// 65 bytes signature
			258;

		if access_list.is_some() {
			estimated_transaction_len += access_list.encoded_size();
		}

		let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();
		let without_base_extrinsic_weight = true;

		let (weight_limit, proof_size_base_cost) =
			match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
				gas_limit,
				without_base_extrinsic_weight
			) {
				weight_limit if weight_limit.proof_size() > 0 => {
					(Some(weight_limit), Some(estimated_transaction_len as u64))
				}
				_ => (None, None),
			};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			let is_transactional = false;
			let validate = true;
			let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());

			let mut estimated_transaction_len = data.len() +
			20 + // from
			32 + // value
			32 + // gas_limit
			32 + // nonce
			1 + // TransactionAction
			8 + // chain id
			65; // signature

		if max_fee_per_gas.is_some() {
			estimated_transaction_len += 32;
		}
		if max_priority_fee_per_gas.is_some() {
			estimated_transaction_len += 32;
		}
		if access_list.is_some() {
			estimated_transaction_len += access_list.encoded_size();
		}

		let gas_limit = if gas_limit > U256::from(u64::MAX) {
			u64::MAX
		} else {
			gas_limit.low_u64()
		};
		let without_base_extrinsic_weight = true;

		let (weight_limit, proof_size_base_cost) =
			match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
				gas_limit,
				without_base_extrinsic_weight
			) {
				weight_limit if weight_limit.proof_size() > 0 => {
					(Some(weight_limit), Some(estimated_transaction_len as u64))
				}
				_ => (None, None),
			};


			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,
				validate,
				weight_limit,
				proof_size_base_cost,
				evm_config,
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<EthereumTransaction>>()
		}

		fn elasticity() -> Option<Permill> {
			Some(pallet_base_fee::Elasticity::<Runtime>::get())
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn initialize_pending_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header);
		}
	}

	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			traced_transaction: &pallet_ethereum::Transaction,
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			use moonbeam_evm_tracer::tracer::EvmTracer;

			// We need to follow the order when replaying the transactions.
			// Block initialize happens first then apply_extrinsic.
			Executive::initialize_block(header);

			// Apply the a subset of extrinsics: all the substrate-specific or ethereum
			// transactions that preceded the requested transaction.
			for ext in extrinsics.into_iter() {
				let _ = match &ext.0.function {
					RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
						if transaction == traced_transaction {
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							return Ok(());
						} else {
							Executive::apply_extrinsic(ext)
						}
					},
					RuntimeCall::MetaTransactions(pallet_sponsored_transactions::Call::send_sponsored_transaction { transaction, .. }) => {
						if transaction == traced_transaction {
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							return Ok(());
						} else {
							Executive::apply_extrinsic(ext)
						}
					},
					RuntimeCall::ZeroGasTransactions(pallet_zero_gas_transactions::Call::send_zero_gas_transaction { transaction, .. }) => {
						if transaction == traced_transaction {
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
							return Ok(());
						} else {
							Executive::apply_extrinsic(ext)
						}
					},
					_ => Executive::apply_extrinsic(ext),
				};
			}
			Err(sp_runtime::DispatchError::Other(
				"Failed to find Ethereum transaction among the extrinsics.",
			))
		}

		fn trace_block(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			known_transactions: Vec<H256>,
			header: &<Block as BlockT>::Header,
		) -> Result<
			(),
			sp_runtime::DispatchError,
		> {
			use moonbeam_evm_tracer::tracer::EvmTracer;


			// We need to follow the order when replaying the transactions.
			// Block initialize happens first then apply_extrinsic.
			Executive::initialize_block(header);

			let mut config = <Runtime as pallet_evm::Config>::config().clone();
			config.estimate = true;

			// Apply all extrinsics. Ethereum extrinsics are traced.
			for ext in extrinsics.into_iter() {
				match &ext.0.function {
					RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => {
						if known_transactions.contains(&transaction.hash()) {
							// Each known extrinsic is a new call stack.
							EvmTracer::emit_new();
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
						} else {
							let _ = Executive::apply_extrinsic(ext);
						}
					}
					RuntimeCall::MetaTransactions(pallet_sponsored_transactions::Call::send_sponsored_transaction { transaction, .. }) => {
						if known_transactions.contains(&transaction.hash()) {
							// Each known extrinsic is a new call stack.
							EvmTracer::emit_new();
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
						} else {
							let _ = Executive::apply_extrinsic(ext);
						}
					},
					RuntimeCall::ZeroGasTransactions(pallet_zero_gas_transactions::Call::send_zero_gas_transaction { transaction, .. }) => {
						if known_transactions.contains(&transaction.hash()) {
							// Each known extrinsic is a new call stack.
							EvmTracer::emit_new();
							EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
						} else {
							let _ = Executive::apply_extrinsic(ext);
						}
					},
					_ => {
						let _ = Executive::apply_extrinsic(ext);
					}
				};
			}

			Ok(())
		}

		fn trace_call(
			header: &<Block as BlockT>::Header,
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError> {
			use moonbeam_evm_tracer::tracer::EvmTracer;

			// We need to follow the order when replaying the transactions.
			// Block initialize happens first then apply_extrinsic.
			Executive::initialize_block(header);

			EvmTracer::new().trace(|| {
				let is_transactional = false;
				let validate = true;
				let without_base_extrinsic_weight = true;


				// Estimated encoded transaction size must be based on the heaviest transaction
				// type (EIP1559Transaction) to be compatible with all transaction types.
				let mut estimated_transaction_len = data.len() +
				// pallet ethereum index: 1
				// transact call index: 1
				// Transaction enum variant: 1
				// chain_id 8 bytes
				// nonce: 32
				// max_priority_fee_per_gas: 32
				// max_fee_per_gas: 32
				// gas_limit: 32
				// action: 21 (enum varianrt + call address)
				// value: 32
				// access_list: 1 (empty vec size)
				// 65 bytes signature
				258;

				if access_list.is_some() {
					estimated_transaction_len += access_list.encoded_size();
				}

				let gas_limit = gas_limit.min(u64::MAX.into()).low_u64();

				let (weight_limit, proof_size_base_cost) =
					match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
						gas_limit,
						without_base_extrinsic_weight
					) {
						weight_limit if weight_limit.proof_size() > 0 => {
							(Some(weight_limit), Some(estimated_transaction_len as u64))
						}
						_ => (None, None),
					};

				let _ = <Runtime as pallet_evm::Config>::Runner::call(
					from,
					to,
					data,
					value,
					gas_limit,
					max_fee_per_gas,
					max_priority_fee_per_gas,
					nonce,
					access_list.unwrap_or_default(),
					is_transactional,
					validate,
					weight_limit,
					proof_size_base_cost,
					<Runtime as pallet_evm::Config>::config(),
				);
			});
			Ok(())
		}
	}

	impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(
			xts_ready: Vec<<Block as BlockT>::Extrinsic>,
			xts_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> moonbeam_rpc_primitives_txpool::TxPoolResponse {
			moonbeam_rpc_primitives_txpool::TxPoolResponse {
				ready: xts_ready
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
				future: xts_future
					.into_iter()
					.filter_map(|xt| match xt.0.function {
						RuntimeCall::Ethereum(pallet_ethereum::Call::transact { transaction }) => Some(transaction),
						_ => None,
					})
					.collect(),
			}
		}
	}

	impl stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block> for Runtime {
		fn convert_zero_gas_transaction(transaction: EthereumTransaction, validator_signature: Vec<u8>) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_zero_gas_transactions::Call::<Runtime>::send_zero_gas_transaction { transaction, validator_signature }.into(),
			)
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}

		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId> for Runtime {
		fn is_compatible_fee(tx: <Block as BlockT>::Extrinsic, validator: AccountId) -> bool {
			match tx.0.function {
				RuntimeCall::Ethereum(transact { transaction }) | RuntimeCall::MetaTransactions(send_sponsored_transaction { transaction, .. }) => {
					let source_address_option = stbl_tools::eth::recover_signer(&transaction);

					if source_address_option.is_none() {
						return true;
					}

					let source_address = source_address_option.unwrap();
					let source_fee_token = <pallet_user_fee_selector::Pallet<Runtime>>::get_user_fee_token(source_address);
					let validator_conversion_rate = <pallet_validator_fee_selector::Pallet<Runtime>>::conversion_rate(source_address, validator.into(), source_fee_token);
					let fee = pallet_base_fee::BaseFeePerGas::<Runtime>::get();
					let custom_fee_info = CustomFeeInfo::new(fee, &transaction);

					if !custom_fee_info.match_validator_conversion_rate_limit(validator_conversion_rate) {
						return false;
					}

					<pallet_validator_fee_selector::Pallet<Runtime>>::validator_supports_fee_token(validator.into(), source_fee_token)
				}
				_ => true, // always return true for non-ethereum transactions
			}
		}
	}

	impl stability_rpc_api::StabilityRpcApi<Block> for Runtime {
		fn get_supported_tokens() -> Vec<H160> {
			<pallet_supported_tokens_manager::Pallet<Runtime> as OtherSupportedTokensManager>::get_supported_tokens()
		}

		fn get_validator_list() -> Vec<H160> {
			let validators = <pallet_validator_set::Pallet<Runtime>>::approved_validators();
			validators
			.iter()
			.map(|v| <Runtime as pallet_custom_balances::Config>::AccountIdMapping::into_evm_address(v))
			.collect()

		}


		fn get_active_validator_list() -> Vec<H160> {
			let validators = <pallet_validator_set::Pallet<Runtime>>::validators();
			validators
			.iter()
			.map(|v| <Runtime as pallet_custom_balances::Config>::AccountIdMapping::into_evm_address(v))
			.collect()

		}

		fn convert_sponsored_transaction(transaction: EthereumTransaction, meta_trx_sponsor: H160, meta_trx_sponsor_signature: Vec<u8>) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_sponsored_transactions::Call::<Runtime>::send_sponsored_transaction { transaction,  meta_trx_sponsor, meta_trx_sponsor_signature }.into(),
			)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use pallet_hotfix_sufficients::Pallet as PalletHotfixSufficients;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);
			list_benchmark!(list, extra, pallet_hotfix_sufficients, PalletHotfixSufficients::<Runtime>);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		fn dispatch_benchmark(
			stability_config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use pallet_evm::Pallet as PalletEvmBench;
			use pallet_hotfix_sufficients::Pallet as PalletHotfixSufficients;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&stability_config, &whitelist);

			add_benchmark!(params, batches, pallet_evm, PalletEvmBench::<Runtime>);
			add_benchmark!(params, batches, pallet_hotfix_sufficients, PalletHotfixSufficients::<Runtime>);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Runtime, WeightPerGas};
	#[test]
	fn configured_base_extrinsic_weight_is_evm_compatible() {
		let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
		let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.base_extrinsic;
		assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
	}
}
