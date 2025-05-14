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

//! Testing utilities.
#![cfg(test)]

use core::str::FromStr;

use fp_evm::FeeCalculator;
use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};
use hex::FromHex;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use sp_core::{ConstU32, H160, H256, U256};
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use std::collections::BTreeMap;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub struct MockUserFeeTokenController;
impl pallet_user_fee_selector::UserFeeTokenController for MockUserFeeTokenController {
	type Error = ();

	fn balance_of(_token: H160) -> sp_core::U256 {
		return U256::from(1000000);
	}

	fn get_user_fee_token(_account: H160) -> H160 {
		return ERC20SlotZero::get();
	}

	fn set_user_fee_token(_account: H160, _token: H160) -> Result<(), Self::Error> {
		Ok(())
	}

	fn transfer(_from: H160, _to: H160, _value: U256) -> Result<(), Self::Error> {
		Ok(())
	}
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
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
	type RuntimeTask = ();
	type Nonce = u64;
	type Block = Block;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub ERC20SlotZero: H160 = H160::from_str("0x22D598E0a9a1b474CdC7c6fBeA0B4F83E12046a9").unwrap();
	pub ZeroSlot : H256 = H256::from_low_u64_be(0);
	pub const GasLimitPovSizeRatio: u64 = 15;
	pub const SuicideQuickClearLimit: u32 = 64;
}
pub struct FixedBaseFee;
impl FeeCalculator for FixedBaseFee {
	fn min_gas_price() -> (U256, Weight) {
		(
			U256::from(1_000_000_000),
			Weight::from_parts(1_000_000_000, 0),
		)
	}
}
impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedBaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = pallet_evm::HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

parameter_types! {
	pub const PostBlockAndTxnHashes: pallet_ethereum::PostLogContent = pallet_ethereum::PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		Ethereum: pallet_ethereum,
		UserFeeSelector: pallet_user_fee_selector,
	}
);

pub struct MockSupportedTokensManager;

impl pallet_supported_tokens_manager::SupportedTokensManager for MockSupportedTokensManager {
	type Error = ();

	fn get_supported_tokens() -> Vec<H160> {
		Default::default()
	}

	fn add_supported_token(_token: H160, _slot: H256) -> Result<(), Self::Error> {
		Ok(())
	}

	fn is_supported_token(_token: H160) -> bool {
		false
	}

	fn remove_supported_token(_token: H160) -> Result<(), Self::Error> {
		Ok(())
	}

	fn get_token_balance_slot(_token: H160) -> Option<H256> {
		None
	}

	fn get_default_token() -> H160 {
		Default::default()
	}

	fn set_default_token(_token: H160) -> Result<(), Self::Error> {
		Ok(())
	}
}

pub struct MockERC20Manager;
impl pallet_erc20_manager::ERC20Manager for MockERC20Manager {
	type Error = ();

	fn balance_of(_token: H160, payer: H160) -> U256 {
		if payer.eq(&FundedAccount::get()) {
			U256::MAX
		} else {
			U256::from(0)
		}
	}

	fn withdraw_amount(_token: H160, _payer: H160, _amount: U256) -> Result<U256, Self::Error> {
		Ok(Default::default())
	}

	fn deposit_amount(_token: H160, _payer: H160, _amount: U256) -> Result<U256, Self::Error> {
		Ok(Default::default())
	}
}

impl pallet_user_fee_selector::Config for Runtime {
	type SupportedTokensManager = MockSupportedTokensManager;
	type ERC20Manager = MockERC20Manager;
}

// Test parameters
parameter_types! {
	pub NoFundsAccount: H160 = H160::from_str("A38395b264f232ffF4bb294b5947092E359dDE88").expect("invalid address");
	pub FundedAccount: H160 = H160::from_str("0x2dEA828C816cC4D7CF195E0D220CB75354f47F2F").expect("invalid address");
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let acc = H160::from_str("A38395b264f232ffF4bb294b5947092E359dDE88")
		.expect("internal H160 is valid; qed");
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_evm::GenesisConfig::<Runtime> {
		_marker: Default::default(),
		accounts: {
			let mut map = BTreeMap::new();
			map.insert(
				acc,
				fp_evm::GenesisAccount {
					balance: U256::from(1_000_000_000_000_000_000u128),
					code: Default::default(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map.insert(
				ERC20SlotZero::get(),
				fp_evm::GenesisAccount {
					balance: Default::default(),
					code: Vec::<u8>::from_hex("608060405234801561001057600080fd5b50600436106100a95760003560e01c80633950935111610071578063395093511461016857806370a082311461019857806395d89b41146101c8578063a457c2d7146101e6578063a9059cbb14610216578063dd62ed3e14610246576100a9565b806306fdde03146100ae578063095ea7b3146100cc57806318160ddd146100fc57806323b872dd1461011a578063313ce5671461014a575b600080fd5b6100b6610276565b6040516100c39190610d20565b60405180910390f35b6100e660048036038101906100e19190610b6a565b610308565b6040516100f39190610d05565b60405180910390f35b61010461032b565b6040516101119190610e22565b60405180910390f35b610134600480360381019061012f9190610b17565b610335565b6040516101419190610d05565b60405180910390f35b610152610364565b60405161015f9190610e3d565b60405180910390f35b610182600480360381019061017d9190610b6a565b61036d565b60405161018f9190610d05565b60405180910390f35b6101b260048036038101906101ad9190610aaa565b6103a4565b6040516101bf9190610e22565b60405180910390f35b6101d06103ec565b6040516101dd9190610d20565b60405180910390f35b61020060048036038101906101fb9190610b6a565b61047e565b60405161020d9190610d05565b60405180910390f35b610230600480360381019061022b9190610b6a565b6104f5565b60405161023d9190610d05565b60405180910390f35b610260600480360381019061025b9190610ad7565b610518565b60405161026d9190610e22565b60405180910390f35b60606003805461028590610f52565b80601f01602080910402602001604051908101604052809291908181526020018280546102b190610f52565b80156102fe5780601f106102d3576101008083540402835291602001916102fe565b820191906000526020600020905b8154815290600101906020018083116102e157829003601f168201915b5050505050905090565b60008061031361059f565b90506103208185856105a7565b600191505092915050565b6000600254905090565b60008061034061059f565b905061034d858285610772565b6103588585856107fe565b60019150509392505050565b60006012905090565b60008061037861059f565b905061039981858561038a8589610518565b6103949190610e74565b6105a7565b600191505092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b6060600480546103fb90610f52565b80601f016020809104026020016040519081016040528092919081815260200182805461042790610f52565b80156104745780601f1061044957610100808354040283529160200191610474565b820191906000526020600020905b81548152906001019060200180831161045757829003601f168201915b5050505050905090565b60008061048961059f565b905060006104978286610518565b9050838110156104dc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104d390610e02565b60405180910390fd5b6104e982868684036105a7565b60019250505092915050565b60008061050061059f565b905061050d8185856107fe565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610617576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161060e90610de2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610687576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161067e90610d62565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040516107659190610e22565b60405180910390a3505050565b600061077e8484610518565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146107f857818110156107ea576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016107e190610d82565b60405180910390fd5b6107f784848484036105a7565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561086e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161086590610dc2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156108de576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108d590610d42565b60405180910390fd5b6108e9838383610a76565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490508181101561096f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161096690610da2565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610a5d9190610e22565b60405180910390a3610a70848484610a7b565b50505050565b505050565b505050565b600081359050610a8f816111fb565b92915050565b600081359050610aa481611212565b92915050565b600060208284031215610ac057610abf610fe2565b5b6000610ace84828501610a80565b91505092915050565b60008060408385031215610aee57610aed610fe2565b5b6000610afc85828601610a80565b9250506020610b0d85828601610a80565b9150509250929050565b600080600060608486031215610b3057610b2f610fe2565b5b6000610b3e86828701610a80565b9350506020610b4f86828701610a80565b9250506040610b6086828701610a95565b9150509250925092565b60008060408385031215610b8157610b80610fe2565b5b6000610b8f85828601610a80565b9250506020610ba085828601610a95565b9150509250929050565b610bb381610edc565b82525050565b610bc482610e58565b610bce8185610e63565b9350610bde818560208601610f1f565b610be781610fe7565b840191505092915050565b6000610bff602383610e63565b9150610c0a82610ff8565b604082019050919050565b6000610c22602283610e63565b9150610c2d82611047565b604082019050919050565b6000610c45601d83610e63565b9150610c5082611096565b602082019050919050565b6000610c68602683610e63565b9150610c73826110bf565b604082019050919050565b6000610c8b602583610e63565b9150610c968261110e565b604082019050919050565b6000610cae602483610e63565b9150610cb98261115d565b604082019050919050565b6000610cd1602583610e63565b9150610cdc826111ac565b604082019050919050565b610cf081610f08565b82525050565b610cff81610f12565b82525050565b6000602082019050610d1a6000830184610baa565b92915050565b60006020820190508181036000830152610d3a8184610bb9565b905092915050565b60006020820190508181036000830152610d5b81610bf2565b9050919050565b60006020820190508181036000830152610d7b81610c15565b9050919050565b60006020820190508181036000830152610d9b81610c38565b9050919050565b60006020820190508181036000830152610dbb81610c5b565b9050919050565b60006020820190508181036000830152610ddb81610c7e565b9050919050565b60006020820190508181036000830152610dfb81610ca1565b9050919050565b60006020820190508181036000830152610e1b81610cc4565b9050919050565b6000602082019050610e376000830184610ce7565b92915050565b6000602082019050610e526000830184610cf6565b92915050565b600081519050919050565b600082825260208201905092915050565b6000610e7f82610f08565b9150610e8a83610f08565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115610ebf57610ebe610f84565b5b828201905092915050565b6000610ed582610ee8565b9050919050565b60008115159050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b600060ff82169050919050565b60005b83811015610f3d578082015181840152602081019050610f22565b83811115610f4c576000848401525b50505050565b60006002820490506001821680610f6a57607f821691505b60208210811415610f7e57610f7d610fb3565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b600080fd5b6000601f19601f8301169050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b61120481610eca565b811461120f57600080fd5b50565b61121b81610f08565b811461122657600080fd5b5056fea264697066735822122081a6d1cd8c33b7a44c114f115269b06839560a6fa8d1b6eeaad02e10566b67ca64736f6c63430008070033").unwrap(),
					nonce: Default::default(),
					storage: {
						let mut storage = BTreeMap::new();
						let initial_default_token_balance = H256::from_str("0x000000000000000000000000000000000000000000084595161401484a000000").expect("invalid hex storage value");
						storage.insert(stbl_tools::eth::get_storage_address_for_mapping(NoFundsAccount::get(), H256::from_low_u64_be(0)), initial_default_token_balance);
						storage.insert(stbl_tools::eth::get_storage_address_for_mapping(FundedAccount::get(), H256::from_low_u64_be(0)), initial_default_token_balance);
						storage
					},
				},
			);
			map
		},
	}.assimilate_storage(&mut t)
	.expect("pallet_evm GenesisConfig failed");

	t.into()
}
