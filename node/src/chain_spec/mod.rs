use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use stability_runtime::{AccountId, GenesisConfig, Precompiles};
use std::{collections::BTreeMap, str::FromStr, vec};
// Substrate
use sp_core::{crypto::Ss58Codec, sr25519, storage::Storage, Pair, Public};
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
		get_account_id_from_seed::<sr25519::Public>(s),
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

pub fn get_key_sr(pubkey: &str) -> sr25519::Public {
	match sr25519::Public::from_str(pubkey) {
		Ok(sr_pubkey) => sr_pubkey,
		Err(_) => panic!("sr pubkey bad formatted"),
	}
}

pub fn get_authority_from_pubkeys(
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

/// Configure initial storage state for FRAME modules.
pub fn base_genesis(
	wasm_binary: &[u8],
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId, ImOnlineId)>,
	members: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	use stability_runtime::{
		AuraConfig, BalancesConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, ImOnlineConfig,
		SessionConfig, SystemConfig, TechCommitteeCollectiveConfig, ValidatorSetConfig,
	};

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
		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from(1_000_000_000_000_000_000_000_000u128),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				// Stability testing account
				map.insert(
					H160::from_str("A38395b264f232ffF4bb294b5947092E359dDE88")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						nonce: Default::default(),
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
				map
			},
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}
