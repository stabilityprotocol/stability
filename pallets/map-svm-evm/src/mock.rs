#![cfg(test)]

use super::*;
use crate as map_svm_evm;

use frame_support::pallet_prelude::Weight;
use frame_support::traits::{Contains, Everything, GenesisBuild};
use hex::FromHex;
use sp_core::{H256, U256};
use std::str::FromStr;

use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use std::collections::BTreeMap;

use frame_support::parameter_types;

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;
pub type BlockNumber = u32;

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

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

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm_chain_id::Config for Test {}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = EVMChainId;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

frame_support::construct_runtime!(
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic, {
			System: frame_system,
			Timestamp: pallet_timestamp,
			Balances: pallet_balances,
			EVM: pallet_evm,
			MapSvmEvm: map_svm_evm,
			EVMChainId: pallet_evm_chain_id
		}
);

parameter_types! {
	pub SmartcontractErc1271Success: H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
	pub SmartcontractErc1271Fails: H160 = H160::from_str("0xdf1dAbfc88Fdb4Bac77eDBcDd6608d1dAeEd02E0").unwrap();
	pub SmartcontractWithoutErc721: H160 = H160::from_str("0xba29c6A61bf8Ff7E1d77bF1B9858010cE6756725").unwrap();
	pub ChainId: u64 = 20180427;
}

pub fn new_test_ext(linked_accounts: Vec<(AccountId, H160)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	map_svm_evm::GenesisConfig::<Test> { linked_accounts }
		.assimilate_storage(&mut t)
		.unwrap();
	let evm_config = pallet_evm::GenesisConfig {
		accounts: {
			let mut map = BTreeMap::new();
			map.insert(
				SmartcontractErc1271Success::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: Vec::<u8>::from_hex("608060405234801561001057600080fd5b506004361061002b5760003560e01c80631626ba7e14610030575b600080fd5b61004a60048036038101906100459190610118565b610060565b60405161005791906101b3565b60405180910390f35b6000631626ba7e60e01b90509392505050565b600080fd5b600080fd5b6000819050919050565b6100908161007d565b811461009b57600080fd5b50565b6000813590506100ad81610087565b92915050565b600080fd5b600080fd5b600080fd5b60008083601f8401126100d8576100d76100b3565b5b8235905067ffffffffffffffff8111156100f5576100f46100b8565b5b602083019150836001820283011115610111576101106100bd565b5b9250929050565b60008060006040848603121561013157610130610073565b5b600061013f8682870161009e565b935050602084013567ffffffffffffffff8111156101605761015f610078565b5b61016c868287016100c2565b92509250509250925092565b60007fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b6101ad81610178565b82525050565b60006020820190506101c860008301846101a4565b9291505056fea26469706673582212203c5f8a369fefd1ff7c875dab43a5f33c16ebd53bf90cc1aea38ec2353e85792564736f6c63430008110033").unwrap(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map.insert(
				SmartcontractErc1271Fails::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: Vec::<u8>::from_hex("608060405234801561001057600080fd5b506004361061002b5760003560e01c80631626ba7e14610030575b600080fd5b61004a60048036038101906100459190610118565b610060565b60405161005791906101b3565b60405180910390f35b600063ffffffff60e01b90509392505050565b600080fd5b600080fd5b6000819050919050565b6100908161007d565b811461009b57600080fd5b50565b6000813590506100ad81610087565b92915050565b600080fd5b600080fd5b600080fd5b60008083601f8401126100d8576100d76100b3565b5b8235905067ffffffffffffffff8111156100f5576100f46100b8565b5b602083019150836001820283011115610111576101106100bd565b5b9250929050565b60008060006040848603121561013157610130610073565b5b600061013f8682870161009e565b935050602084013567ffffffffffffffff8111156101605761015f610078565b5b61016c868287016100c2565b92509250509250925092565b60007fffffffff0000000000000000000000000000000000000000000000000000000082169050919050565b6101ad81610178565b82525050565b60006020820190506101c860008301846101a4565b9291505056fea264697066735822122026ba584e6dfd7892b68718b0ad6cf7e76826fd04aa14f2db3df75e7ccfaf7aa164736f6c63430008110033").unwrap(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map.insert(
				SmartcontractWithoutErc721::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: Vec::<u8>::from_hex("6080604052348015600f57600080fd5b506004361060285760003560e01c80639476f92214602d575b600080fd5b60336047565b604051603e91906069565b60405180910390f35b60006001905090565b60008115159050919050565b6063816050565b82525050565b6000602082019050607c6000830184605c565b9291505056fea2646970667358221220a07ef20d077b5efde86c3f322a5c2ef6ec0a181b6b84a4f564d1485ec066f4b164736f6c63430008120033").unwrap(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);

			map
		},
	};
	<pallet_evm::GenesisConfig as GenesisBuild<Test>>::assimilate_storage(&evm_config, &mut t)
		.unwrap();

	let chain_id_config = pallet_evm_chain_id::GenesisConfig {
		chain_id: ChainId::get(),
	};

	<pallet_evm_chain_id::GenesisConfig as GenesisBuild<Test>>::assimilate_storage(
		&chain_id_config,
		&mut t,
	)
	.unwrap();

	t.into()
}
