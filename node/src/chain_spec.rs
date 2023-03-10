use std::{collections::BTreeMap, str::FromStr, vec};

use serde::{Deserialize, Serialize};
// Substrate
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
	bytes::from_hex, crypto::Ss58Codec, sr25519, storage::Storage, Pair, Public, H160, H256, U256,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
// Frontier
use stability_runtime::{
	opaque::SessionKeys, AccountId, EnableManualSeal, GenesisConfig, Precompiles, Signature,
	WASM_BINARY,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
	/// Genesis config.
	genesis_config: GenesisConfig,
	/// The flag that if enable manual-seal mode.
	enable_manual_seal: Option<bool>,
}

impl sp_runtime::BuildStorage for DevGenesisExt {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		BasicExternalities::execute_with_storage(storage, || {
			if let Some(enable_manual_seal) = &self.enable_manual_seal {
				EnableManualSeal::set(enable_manual_seal);
			}
		});
		self.genesis_config.assimilate_storage(storage)
	}
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

fn session_keys(aura: AuraId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys {
		aura,
		grandpa,
		im_online,
	}
}

fn get_key_sr(pubkey: &str) -> sr25519::Public {
	match sr25519::Public::from_str(pubkey) {
		Ok(sr_pubkey) => sr_pubkey,
		Err(_) => panic!("sr pubkey bad formatted"),
	}
}

fn get_authority_from_pubkeys(
	sr_pubkey: &str,
	ed_pubkey: &str,
) -> (AccountId, AuraId, GrandpaId, ImOnlineId) {
	(
		AccountId::from_string(sr_pubkey).expect("bad formatted sr pubkey"),
		AuraId::from_string(sr_pubkey).expect("bad formatted sr pubkey"),
		GrandpaId::from_string(ed_pubkey).expect("bad formatted ed pubkey"),
		ImOnlineId::from_string(sr_pubkey).expect("bad formatted ed pubkey"),
	)
}

pub fn development_config(enable_manual_seal: Option<bool>) -> DevChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM not available");

	DevChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			DevGenesisExt {
				genesis_config: testnet_genesis(
					wasm_binary,
					// Sudo account
					// Pre-funded accounts
					vec![
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					],
					// Initial PoA authorities
					vec![authority_keys_from_seed("Alice")],
					vec![(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						H160::from_str("0x76d2265F02C763Dc71A02f62B1Dce4d58b3bb7e2")
							.expect("Bad account id format"),
					)],
					vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
					42,
				),
				enable_manual_seal,
			}
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	)
}

pub fn local_testnet_config() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM not available");

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						H160::from_str("0x76d2265F02C763Dc71A02f62B1Dce4d58b3bb7e2")
							.expect("Bad account id format"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						H160::from_str("0x6e63A65a87E24046167a2ce1971B6b765BaD3293")
							.expect("Bad account id format"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				],
				20180427,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	)
}

pub fn alphanet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let pubkey_sr = get_key_sr("5FC9q4Nu51s48cJ9RqTj78zyFhpi2wpC1jzt3hXLAiqkfAbs");
	let main_account = AccountPublic::from(pubkey_sr).into_account();

	Ok(ChainSpec::from_genesis(
		// Name
		"Alphanet",
		// ID
		"alphanet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// Pre-funded accounts
				vec![main_account.clone()],
				vec![
					get_authority_from_pubkeys(
						"5FC9q4Nu51s48cJ9RqTj78zyFhpi2wpC1jzt3hXLAiqkfAbs",
						"5FviP577ihFCP4n8jCnrd38dQDCn2VeM5DAoYNEHbPy7JtWz",
					),
					get_authority_from_pubkeys(
						"5G1wTFx3iLZCWejfaSGfxQtdKHRzDJ4ga7iabWVS1a9DND2L",
						"5FqDv66PJL7TtC49CitcfGNokKJjbL3nwDRmYnQ66BJttUrw",
					),
					get_authority_from_pubkeys(
						"5Dnet7dgiMJBuAyizMH5EW9JYSXttNKFveQH5Miekrwb4GxJ",
						"5GUVATr2DwH51tnqaUEuBtuHA4bLnoWBAkauDxDafMukpZAZ",
					),
					get_authority_from_pubkeys(
						"5H639iD2JYtZbQN5sNNVRhVtpvzHyhXp5MwGBgW1FLz71zkp",
						"5FiEnbnj7VV5CWAtbJXtZjCkiQTBBRqyb8MgXEkydW1SfLiJ",
					),
					get_authority_from_pubkeys(
						"5DtBoSGDHwH4aLJUA2LVYwprziGEyDopXc4YvdU8LRB1rzdv",
						"5GzRrcmG4kztd31FPWEcr51B3Jd2GZPh6ZjpxwzymopHuViN",
					),
				],
				vec![
					(
						AccountId::from_string("5FC9q4Nu51s48cJ9RqTj78zyFhpi2wpC1jzt3hXLAiqkfAbs")
							.expect("Bad account id format"),
						H160::from_str("0x76d2265F02C763Dc71A02f62B1Dce4d58b3bb7e2")
							.expect("Bad account id format"),
					),
					(
						AccountId::from_string("5G1wTFx3iLZCWejfaSGfxQtdKHRzDJ4ga7iabWVS1a9DND2L")
							.expect("Bad account id format"),
						H160::from_str("0x6e63A65a87E24046167a2ce1971B6b765BaD3293")
							.expect("Bad account id format"),
					),
					(
						AccountId::from_string("5Dnet7dgiMJBuAyizMH5EW9JYSXttNKFveQH5Miekrwb4GxJ")
							.expect("Bad account id format"),
						H160::from_str("0x3dbCC2fbe676037579A59fD2412AA4DcAe11db97")
							.expect("Bad account id format"),
					),
					(
						AccountId::from_string("5H639iD2JYtZbQN5sNNVRhVtpvzHyhXp5MwGBgW1FLz71zkp")
							.expect("Bad account id format"),
						H160::from_str("0x1db0Fd9B3Abf83C802869D0De696157e60b136e5")
							.expect("Bad account id format"),
					),
					(
						AccountId::from_string("5DtBoSGDHwH4aLJUA2LVYwprziGEyDopXc4YvdU8LRB1rzdv")
							.expect("Bad account id format"),
						H160::from_str("0xf349108C149a99C133c662A6779E07d0e68340d6")
							.expect("Bad account id format"),
					),
				],
				vec![
					AccountId::from_string("5FC9q4Nu51s48cJ9RqTj78zyFhpi2wpC1jzt3hXLAiqkfAbs")
						.expect("Bad account id format"),
				],
				20180427,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId, ImOnlineId)>,
	linked_accounts: Vec<(AccountId, H160)>,
	members: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	use stability_runtime::{
		AuraConfig, BalancesConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, MapSvmEvmConfig, ImOnlineConfig,
		SupportedTokensManagerConfig, SystemConfig, TechCommitteeCollectiveConfig, ValidatorSetConfig, SessionConfig
	};
	let initial_default_token =
		H160::from_str("0xDc2B93f3291030F3F7a6D9363ac37757f7AD5C43").expect("invalid address");
	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		// Monetary
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		transaction_payment: Default::default(),

		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.1.clone(), x.2.clone(), x.3.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		// Consensus
		aura: AuraConfig {
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		tech_committee_collective: TechCommitteeCollectiveConfig {
			phantom: Default::default(),
			members: members.clone(),
		},

		map_svm_evm: MapSvmEvmConfig {
			linked_accounts: linked_accounts.clone(),
		},
		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				// Stability testing account
				map.insert(
					H160::from_str("A38395b264f232ffF4bb294b5947092E359dDE88")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						nonce: U256::from(1),
						balance: U256::from(1_000_000_000_000_000_000_000_000u128),
						storage: Default::default(),
						code: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address for benchmark usage
					H160::from_str("1000000000000000000000000000000000000001")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						nonce: U256::from(1),
						balance: U256::from(1_000_000_000_000_000_000_000_000u128),
						storage: Default::default(),
						code: vec![0x00],
					},
				);
				let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
				Precompiles::used_addresses()
					.into_iter()
					.for_each(|addr: H160| {
						map.insert(
							addr,
							fp_evm::GenesisAccount {
								nonce: Default::default(),
								balance: Default::default(),
								storage: Default::default(),
								code: revert_bytecode.clone(),
							},
						);
					});
				map.insert(initial_default_token, fp_evm::GenesisAccount {
					nonce: Default::default(),
					balance: Default::default(),
					storage: {
						let mut storage = BTreeMap::new();
						let initial_default_token_balance = H256::from_str("0x444f4c5200000000000000000000000000000000000000000000000000000008").expect("invalid hex storage value");
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").expect("invalid hex storage key"), initial_default_token_balance);
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000003").expect("invalid hex storage key"), H256::from_str("0x444f4c5200000000000000000000000000000000000000000000000000000008").expect("invalid hex storage value"));
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000004").expect("invalid hex storage key"), H256::from_str("0x444f4c5200000000000000000000000000000000000000000000000000000008").expect("invalid hex storage value"));
						storage.insert(H256::from_str("e5ddd19dddbc957f8947ecfc88aaec9fa58987adbcaa4d981451357aaddf59ea").expect("invalid hex storage key"), initial_default_token_balance);
						storage
					},
					code: from_hex("0x608060405234801561001057600080fd5b50600436106100a95760003560e01c80633950935111610071578063395093511461016857806370a082311461019857806395d89b41146101c8578063a457c2d7146101e6578063a9059cbb14610216578063dd62ed3e14610246576100a9565b806306fdde03146100ae578063095ea7b3146100cc57806318160ddd146100fc57806323b872dd1461011a578063313ce5671461014a575b600080fd5b6100b6610276565b6040516100c39190610d20565b60405180910390f35b6100e660048036038101906100e19190610b6a565b610308565b6040516100f39190610d05565b60405180910390f35b61010461032b565b6040516101119190610e22565b60405180910390f35b610134600480360381019061012f9190610b17565b610335565b6040516101419190610d05565b60405180910390f35b610152610364565b60405161015f9190610e3d565b60405180910390f35b610182600480360381019061017d9190610b6a565b61036d565b60405161018f9190610d05565b60405180910390f35b6101b260048036038101906101ad9190610aaa565b6103a4565b6040516101bf9190610e22565b60405180910390f35b6101d06103ec565b6040516101dd9190610d20565b60405180910390f35b61020060048036038101906101fb9190610b6a565b61047e565b60405161020d9190610d05565b60405180910390f35b610230600480360381019061022b9190610b6a565b6104f5565b60405161023d9190610d05565b60405180910390f35b610260600480360381019061025b9190610ad7565b610518565b60405161026d9190610e22565b60405180910390f35b60606003805461028590610f52565b80601f01602080910402602001604051908101604052809291908181526020018280546102b190610f52565b80156102fe5780601f106102d3576101008083540402835291602001916102fe565b820191906000526020600020905b8154815290600101906020018083116102e157829003601f168201915b5050505050905090565b60008061031361059f565b90506103208185856105a7565b600191505092915050565b6000600254905090565b60008061034061059f565b905061034d858285610772565b6103588585856107fe565b60019150509392505050565b60006012905090565b60008061037861059f565b905061039981858561038a8589610518565b6103949190610e74565b6105a7565b600191505092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b6060600480546103fb90610f52565b80601f016020809104026020016040519081016040528092919081815260200182805461042790610f52565b80156104745780601f1061044957610100808354040283529160200191610474565b820191906000526020600020905b81548152906001019060200180831161045757829003601f168201915b5050505050905090565b60008061048961059f565b905060006104978286610518565b9050838110156104dc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104d390610e02565b60405180910390fd5b6104e982868684036105a7565b60019250505092915050565b60008061050061059f565b905061050d8185856107fe565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610617576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161060e90610de2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610687576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161067e90610d62565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040516107659190610e22565b60405180910390a3505050565b600061077e8484610518565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146107f857818110156107ea576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016107e190610d82565b60405180910390fd5b6107f784848484036105a7565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561086e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161086590610dc2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156108de576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108d590610d42565b60405180910390fd5b6108e9838383610a76565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490508181101561096f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161096690610da2565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610a5d9190610e22565b60405180910390a3610a70848484610a7b565b50505050565b505050565b505050565b600081359050610a8f816111fb565b92915050565b600081359050610aa481611212565b92915050565b600060208284031215610ac057610abf610fe2565b5b6000610ace84828501610a80565b91505092915050565b60008060408385031215610aee57610aed610fe2565b5b6000610afc85828601610a80565b9250506020610b0d85828601610a80565b9150509250929050565b600080600060608486031215610b3057610b2f610fe2565b5b6000610b3e86828701610a80565b9350506020610b4f86828701610a80565b9250506040610b6086828701610a95565b9150509250925092565b60008060408385031215610b8157610b80610fe2565b5b6000610b8f85828601610a80565b9250506020610ba085828601610a95565b9150509250929050565b610bb381610edc565b82525050565b6000610bc482610e58565b610bce8185610e63565b9350610bde818560208601610f1f565b610be781610fe7565b840191505092915050565b6000610bff602383610e63565b9150610c0a82610ff8565b604082019050919050565b6000610c22602283610e63565b9150610c2d82611047565b604082019050919050565b6000610c45601d83610e63565b9150610c5082611096565b602082019050919050565b6000610c68602683610e63565b9150610c73826110bf565b604082019050919050565b6000610c8b602583610e63565b9150610c968261110e565b604082019050919050565b6000610cae602483610e63565b9150610cb98261115d565b604082019050919050565b6000610cd1602583610e63565b9150610cdc826111ac565b604082019050919050565b610cf081610f08565b82525050565b610cff81610f12565b82525050565b6000602082019050610d1a6000830184610baa565b92915050565b60006020820190508181036000830152610d3a8184610bb9565b905092915050565b60006020820190508181036000830152610d5b81610bf2565b9050919050565b60006020820190508181036000830152610d7b81610c15565b9050919050565b60006020820190508181036000830152610d9b81610c38565b9050919050565b60006020820190508181036000830152610dbb81610c5b565b9050919050565b60006020820190508181036000830152610ddb81610c7e565b9050919050565b60006020820190508181036000830152610dfb81610ca1565b9050919050565b60006020820190508181036000830152610e1b81610cc4565b9050919050565b6000602082019050610e376000830184610ce7565b92915050565b6000602082019050610e526000830184610cf6565b92915050565b600081519050919050565b600082825260208201905092915050565b6000610e7f82610f08565b9150610e8a83610f08565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115610ebf57610ebe610f84565b5b828201905092915050565b6000610ed582610ee8565b9050919050565b60008115159050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b600060ff82169050919050565b60005b83811015610f3d578082015181840152602081019050610f22565b83811115610f4c576000848401525b50505050565b60006002820490506001821680610f6a57607f821691505b60208210811415610f7e57610f7d610fb3565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b600080fd5b6000601f19601f8301169050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b61120481610eca565b811461120f57600080fd5b50565b61121b81610f08565b811461122657600080fd5b5056fea26469706673582212209daa28e8827f7929dd57c8cbf97f9bf1107593dbe1d6e3c5d442c610f132f07e64736f6c63430008070033").expect("invalid hex"),
				});
				map
			},
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		supported_tokens_manager: SupportedTokensManagerConfig {
			initial_default_token,
			initial_default_token_slot: H256::zero(),
		},
	}
}
