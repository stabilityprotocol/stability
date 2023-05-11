use serde::{Deserialize, Serialize};
use sp_core::{bytes::from_hex, ecdsa, H160, H256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use stability_runtime::{AccountId, GenesisConfig, Precompiles, ValidatorFeeSelectorConfig};
use std::{collections::BTreeMap, str::FromStr, vec};
// Substrate
use sp_core::{crypto::Ss58Codec, storage::Storage, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
// Frontier
use stability_runtime::{opaque::SessionKeys, EnableManualSeal, Signature};

pub mod alphanet;
pub mod dev;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

pub type AuraId = stbl_core_primitives::aura::Public;
pub type ImOnlineId = stbl_core_primitives::imonline::Public;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
	/// Genesis config.
	pub genesis_config: GenesisConfig,
	/// The flag that if enable manual-seal mode.
	pub enable_manual_seal: Option<bool>,
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
		get_account_id_from_seed::<ecdsa::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
	)
}

pub fn session_keys(aura: AuraId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys {
		aura,
		grandpa,
		im_online,
	}
}

pub fn get_authority_from_pubkeys(
	ecdsa_key_str: &str,
	ed_pubkey: &str,
) -> (AccountId, AuraId, GrandpaId, ImOnlineId) {
	let ecdsa_pubkey = ecdsa::Public::from_string(ecdsa_key_str).unwrap();
	(
		(account::EthereumSigner::from(ecdsa_pubkey).into_account()),
		AuraId::from_string(ecdsa_key_str).unwrap(),
		GrandpaId::from_string(ed_pubkey).unwrap(),
		ImOnlineId::from_string(ecdsa_key_str).unwrap(),
	)
}

/// Configure initial storage state for FRAME modules.
pub fn base_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId, ImOnlineId)>,
	members: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	use stability_runtime::{
		AuraConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, ImOnlineConfig, SessionConfig,
		SupportedTokensManagerConfig, SystemConfig, TechCommitteeCollectiveConfig,
		ValidatorSetConfig,
	};
	let initial_default_token =
		H160::from_str("0xDc2B93f3291030F3F7a6D9363ac37757f7AD5C43").expect("invalid address");
	let initial_default_conversion_rate_controller =
		H160::from_str("0x444212d6E4827893A70d19921E383130281Cda4a").expect("invalid address");
	let main_account =
		H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5").expect("invalid address");
	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
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
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
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
						let initial_default_token_balance = H256::from_str("0x000000000000000000000000000000000000000000084595161401484a000000").expect("invalid hex storage value"); // 10M
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").expect("invalid hex storage key"), initial_default_token_balance);
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000003").expect("invalid hex storage key"), H256::from_str("0x53746162696c697479205465737400000000000000000000000000000000001c").expect("invalid hex storage value")); // Name
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000004").expect("invalid hex storage key"), H256::from_str("0x5354424c00000000000000000000000000000000000000000000000000000008").expect("invalid hex storage value")); // Symbol
						storage.insert(stbl_tools::eth::get_storage_address_for_mapping(main_account, H256::from_low_u64_be(0)), initial_default_token_balance);
						storage
					},
					code: from_hex("0x608060405234801561001057600080fd5b50600436106100a95760003560e01c80633950935111610071578063395093511461016857806370a082311461019857806395d89b41146101c8578063a457c2d7146101e6578063a9059cbb14610216578063dd62ed3e14610246576100a9565b806306fdde03146100ae578063095ea7b3146100cc57806318160ddd146100fc57806323b872dd1461011a578063313ce5671461014a575b600080fd5b6100b6610276565b6040516100c39190610d20565b60405180910390f35b6100e660048036038101906100e19190610b6a565b610308565b6040516100f39190610d05565b60405180910390f35b61010461032b565b6040516101119190610e22565b60405180910390f35b610134600480360381019061012f9190610b17565b610335565b6040516101419190610d05565b60405180910390f35b610152610364565b60405161015f9190610e3d565b60405180910390f35b610182600480360381019061017d9190610b6a565b61036d565b60405161018f9190610d05565b60405180910390f35b6101b260048036038101906101ad9190610aaa565b6103a4565b6040516101bf9190610e22565b60405180910390f35b6101d06103ec565b6040516101dd9190610d20565b60405180910390f35b61020060048036038101906101fb9190610b6a565b61047e565b60405161020d9190610d05565b60405180910390f35b610230600480360381019061022b9190610b6a565b6104f5565b60405161023d9190610d05565b60405180910390f35b610260600480360381019061025b9190610ad7565b610518565b60405161026d9190610e22565b60405180910390f35b60606003805461028590610f52565b80601f01602080910402602001604051908101604052809291908181526020018280546102b190610f52565b80156102fe5780601f106102d3576101008083540402835291602001916102fe565b820191906000526020600020905b8154815290600101906020018083116102e157829003601f168201915b5050505050905090565b60008061031361059f565b90506103208185856105a7565b600191505092915050565b6000600254905090565b60008061034061059f565b905061034d858285610772565b6103588585856107fe565b60019150509392505050565b60006012905090565b60008061037861059f565b905061039981858561038a8589610518565b6103949190610e74565b6105a7565b600191505092915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b6060600480546103fb90610f52565b80601f016020809104026020016040519081016040528092919081815260200182805461042790610f52565b80156104745780601f1061044957610100808354040283529160200191610474565b820191906000526020600020905b81548152906001019060200180831161045757829003601f168201915b5050505050905090565b60008061048961059f565b905060006104978286610518565b9050838110156104dc576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016104d390610e02565b60405180910390fd5b6104e982868684036105a7565b60019250505092915050565b60008061050061059f565b905061050d8185856107fe565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff161415610617576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161060e90610de2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415610687576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161067e90610d62565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925836040516107659190610e22565b60405180910390a3505050565b600061077e8484610518565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff81146107f857818110156107ea576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016107e190610d82565b60405180910390fd5b6107f784848484036105a7565b5b50505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561086e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161086590610dc2565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156108de576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108d590610d42565b60405180910390fd5b6108e9838383610a76565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490508181101561096f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161096690610da2565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef84604051610a5d9190610e22565b60405180910390a3610a70848484610a7b565b50505050565b505050565b505050565b600081359050610a8f816111fb565b92915050565b600081359050610aa481611212565b92915050565b600060208284031215610ac057610abf610fe2565b5b6000610ace84828501610a80565b91505092915050565b60008060408385031215610aee57610aed610fe2565b5b6000610afc85828601610a80565b9250506020610b0d85828601610a80565b9150509250929050565b600080600060608486031215610b3057610b2f610fe2565b5b6000610b3e86828701610a80565b9350506020610b4f86828701610a80565b9250506040610b6086828701610a95565b9150509250925092565b60008060408385031215610b8157610b80610fe2565b5b6000610b8f85828601610a80565b9250506020610ba085828601610a95565b9150509250929050565b610bb381610edc565b82525050565b6000610bc482610e58565b610bce8185610e63565b9350610bde818560208601610f1f565b610be781610fe7565b840191505092915050565b6000610bff602383610e63565b9150610c0a82610ff8565b604082019050919050565b6000610c22602283610e63565b9150610c2d82611047565b604082019050919050565b6000610c45601d83610e63565b9150610c5082611096565b602082019050919050565b6000610c68602683610e63565b9150610c73826110bf565b604082019050919050565b6000610c8b602583610e63565b9150610c968261110e565b604082019050919050565b6000610cae602483610e63565b9150610cb98261115d565b604082019050919050565b6000610cd1602583610e63565b9150610cdc826111ac565b604082019050919050565b610cf081610f08565b82525050565b610cff81610f12565b82525050565b6000602082019050610d1a6000830184610baa565b92915050565b60006020820190508181036000830152610d3a8184610bb9565b905092915050565b60006020820190508181036000830152610d5b81610bf2565b9050919050565b60006020820190508181036000830152610d7b81610c15565b9050919050565b60006020820190508181036000830152610d9b81610c38565b9050919050565b60006020820190508181036000830152610dbb81610c5b565b9050919050565b60006020820190508181036000830152610ddb81610c7e565b9050919050565b60006020820190508181036000830152610dfb81610ca1565b9050919050565b60006020820190508181036000830152610e1b81610cc4565b9050919050565b6000602082019050610e376000830184610ce7565b92915050565b6000602082019050610e526000830184610cf6565b92915050565b600081519050919050565b600082825260208201905092915050565b6000610e7f82610f08565b9150610e8a83610f08565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115610ebf57610ebe610f84565b5b828201905092915050565b6000610ed582610ee8565b9050919050565b60008115159050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b600060ff82169050919050565b60005b83811015610f3d578082015181840152602081019050610f22565b83811115610f4c576000848401525b50505050565b60006002820490506001821680610f6a57607f821691505b60208210811415610f7e57610f7d610fb3565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b600080fd5b6000601f19601f8301169050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b61120481610eca565b811461120f57600080fd5b50565b61121b81610f08565b811461122657600080fd5b5056fea26469706673582212209daa28e8827f7929dd57c8cbf97f9bf1107593dbe1d6e3c5d442c610f132f07e64736f6c63430008070033").expect("invalid hex"),
				});
				map.insert(
					initial_default_conversion_rate_controller,
					fp_evm::GenesisAccount {
						nonce: Default::default(),
						balance: Default::default(),
						storage: {
							let mut storage = BTreeMap::new();
							storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hex storage key"), main_account.into());
							storage
						},
						code: from_hex("0x608060405234801561001057600080fd5b506004361061004c5760003560e01c80638da5cb5b14610051578063c5c3ab451461006f578063d9e1fc141461008b578063ff47b5a6146100bc575b600080fd5b6100596100ed565b6040516100669190610407565b60405180910390f35b61008960048036038101906100849190610373565b610111565b005b6100a560048036038101906100a091906102f3565b61020f565b6040516100b3929190610442565b60405180910390f35b6100d660048036038101906100d19190610320565b610233565b6040516100e4929190610442565b60405180910390f35b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff161461019f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161019690610422565b60405180910390fd5b604051806040016040528083815260200182815250600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000820151816000015560208201518160010155905050505050565b60016020528060005260406000206000915090508060000154908060010154905082565b6000806000600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206040518060400160405290816000820154815260200160018201548152505090506000808260200151149050806102b757816000015182602001516102bb565b6001805b935093505050935093915050565b6000813590506102d88161050c565b92915050565b6000813590506102ed81610523565b92915050565b600060208284031215610309576103086104b8565b5b6000610317848285016102c9565b91505092915050565b600080600060608486031215610339576103386104b8565b5b6000610347868287016102c9565b9350506020610358868287016102c9565b9250506040610369868287016102c9565b9150509250925092565b60008060006060848603121561038c5761038b6104b8565b5b600061039a868287016102c9565b93505060206103ab868287016102de565b92505060406103bc868287016102de565b9150509250925092565b6103cf8161047c565b82525050565b60006103e260218361046b565b91506103ed826104bd565b604082019050919050565b610401816104ae565b82525050565b600060208201905061041c60008301846103c6565b92915050565b6000602082019050818103600083015261043b816103d5565b9050919050565b600060408201905061045760008301856103f8565b61046460208301846103f8565b9392505050565b600082825260208201905092915050565b60006104878261048e565b9050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b600080fd5b7f436f6e76657273696f6e526174654d616e616765723a204f4e4c595f4f574e4560008201527f5200000000000000000000000000000000000000000000000000000000000000602082015250565b6105158161047c565b811461052057600080fd5b50565b61052c816104ae565b811461053757600080fd5b5056fea2646970667358221220b2777c2a4814eaee2b96fedfd2cabf6a8cf8b3cd19bbf9d9255533266aee23d564736f6c63430008070033").expect("invalid hex"),
					},
				);
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
		validator_fee_selector: ValidatorFeeSelectorConfig {
			initial_default_conversion_rate_controller,
		},
		dnt_fee_controller: Default::default(),
	}
}
