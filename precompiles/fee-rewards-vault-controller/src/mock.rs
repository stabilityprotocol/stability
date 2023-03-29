use super::*;

use frame_support::{parameter_types, weights::Weight};
use frame_support::traits::{ConstU32, ConstU64, Contains, GenesisBuild};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_version::RuntimeVersion;
use precompile_utils::precompile_set::*;
use pallet_evm::{EnsureAddressRoot, EnsureAddressNever, AddressMapping};
use sp_std::vec;
use std::collections::BTreeMap;
use hex::FromHex;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

parameter_types! {
	pub DefaultOwner: H160 = H160::from_str("0xa58482131a8d67725e996af72D91A849AcC0F4A1").expect("invalid address");
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<AddressU64<1>, FeeRewardsVaultControllerPrecompile<R, DefaultOwner>>,
>;

pub type PCall = FeeRewardsVaultControllerPrecompileCall<Test, DefaultOwner>;

parameter_types! {
	pub Version: RuntimeVersion = RuntimeVersion {
		spec_name: sp_version::create_runtime_str!("test"),
		impl_name: sp_version::create_runtime_str!("system-test"),
		authoring_version: 1,
		spec_version: 1,
		impl_version: 1,
		apis: sp_version::create_apis_vec!([]),
		transaction_version: 1,
		state_version: 1,
	};
}

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}


parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}


pub struct NumberAddressMapping;

impl AddressMapping<u64> for NumberAddressMapping {
	fn into_account_id(address: H160) -> u64 {
		let address_bytes: [u8; 8]  = (*address.as_fixed_bytes())[12..].try_into().unwrap();
		u64::from_be_bytes(address_bytes)
	}
}


parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Test> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<u64>;
	type WithdrawOrigin = EnsureAddressNever<u64>;
	type AddressMapping = NumberAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

pub type Balance = u128;

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_fee_rewards_vault::Config for Test {
}

frame_support::construct_runtime!(
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic, {
			System: frame_system,
			FeeRewardsVault: pallet_fee_rewards_vault,
			Timestamp: pallet_timestamp,
			EVM: pallet_evm,
			Balances: pallet_balances,
		}
);

parameter_types! {
    pub OwnerOfDapp: H160 = H160::from_str("0x2681896781c7b49A68dCBaB833D5544501794D78").expect("invalid address");
    pub NotOwner: H160 = H160::from_str("0xd5A80D493a784883Dbf7f0fd0FED4D3156C0235F").expect("invalid address");
	pub SmartContractWithOwner: H160 = H160::from_str("0x6f533d42ade638B8c3dBE3F6822697Ccd2662615").expect("invalid address");
    pub SmartContractWithOwnerCode: Vec<u8> = Vec::<u8>::from_hex("608060405234801561001057600080fd5b50600436106100415760003560e01c8063715018a6146100465780638da5cb5b14610050578063f2fde38b1461006e575b600080fd5b61004e61008a565b005b61005861009e565b60405161006591906102d5565b60405180910390f35b61008860048036038101906100839190610321565b6100c7565b005b61009261014a565b61009c60006101c8565b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b6100cf61014a565b600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff160361013e576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610135906103d1565b60405180910390fd5b610147816101c8565b50565b61015261028c565b73ffffffffffffffffffffffffffffffffffffffff1661017061009e565b73ffffffffffffffffffffffffffffffffffffffff16146101c6576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016101bd9061043d565b60405180910390fd5b565b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff169050816000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b60006102bf82610294565b9050919050565b6102cf816102b4565b82525050565b60006020820190506102ea60008301846102c6565b92915050565b600080fd5b6102fe816102b4565b811461030957600080fd5b50565b60008135905061031b816102f5565b92915050565b600060208284031215610337576103366102f0565b5b60006103458482850161030c565b91505092915050565b600082825260208201905092915050565b7f4f776e61626c653a206e6577206f776e657220697320746865207a65726f206160008201527f6464726573730000000000000000000000000000000000000000000000000000602082015250565b60006103bb60268361034e565b91506103c68261035f565b604082019050919050565b600060208201905081810360008301526103ea816103ae565b9050919050565b7f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572600082015250565b600061042760208361034e565b9150610432826103f1565b602082019050919050565b600060208201905081810360008301526104568161041a565b905091905056fea26469706673582212200e08f3fccbab50f6112138f1246c3fb25133b5a676d018004e82ab88882a33d764736f6c63430008120033").expect("invalid address");
    pub SmartContratWithoutOwner: H160= H160::from_str("0x85D30C2b6273E7d3b6e5B9E3c00860D42e3C25bd").expect("invalid address");
    pub SmartContratWithoutOwnerCode: Vec<u8> = Vec::<u8>::from_hex("6080604052348015600f57600080fd5b506004361060285760003560e01c8063cb7acdd914602d575b600080fd5b60336035565b005b56fea264697066735822122080cab20cbc1d3f2f36c19e83c8902d7ba237ffaac3a82bb14feb8f8aa8cda4f064736f6c63430008120033").expect("invalid address");
}
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(u64, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

	let evm_config = pallet_evm::GenesisConfig {
		accounts: {
			let mut map = BTreeMap::new();
			map.insert(
				SmartContractWithOwner::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: SmartContractWithOwnerCode::get(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map.insert(
				SmartContratWithoutOwner::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: SmartContratWithoutOwnerCode::get(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map
		},
	};
	<pallet_evm::GenesisConfig as GenesisBuild<Test>>::assimilate_storage(&evm_config, &mut t).unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}