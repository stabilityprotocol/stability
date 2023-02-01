use std::{collections::BTreeMap, str::FromStr};

use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

use frontier_template_runtime::{AccountId, GenesisConfig, Signature, WASM_BINARY};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
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

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
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

fn get_key_sr(pubkey: &str) -> sr25519::Public {
    match sr25519::Public::from_str(pubkey) {
        Ok(sr_pubkey) => sr_pubkey,
        Err(_) => panic!("sr pubkey bad formatted"),
    }
}

fn get_authority_from_pubkeys(sr_pubkey: &str, ed_pubkey: &str) -> (AuraId, GrandpaId) {
    (
        AuraId::from_string(sr_pubkey).expect("bad formatted sr pubkey"),
        GrandpaId::from_string(ed_pubkey).expect("bad formatted ed pubkey"),
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
                // Sudo account
                main_account.clone(),
                // Pre-funded accounts
                vec![main_account.clone()],
                vec![get_authority_from_pubkeys(
                    "5FC9q4Nu51s48cJ9RqTj78zyFhpi2wpC1jzt3hXLAiqkfAbs",
                    "5FviP577ihFCP4n8jCnrd38dQDCn2VeM5DAoYNEHbPy7JtWz",
                )],
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
    sudo_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    chain_id: u64,
) -> GenesisConfig {
    use frontier_template_runtime::{
        AuraConfig, BalancesConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, SudoConfig,
        SystemConfig,
    };

    GenesisConfig {
        // System
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(sudo_key),
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

        // Consensus
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
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
                map.insert(
                    H160::from_str("a58482131a8d67725e996af72D91A849AcC0F4A1")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        nonce: Default::default(),
                        balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                        storage: Default::default(),
                        code: vec![0x00],
                    },
                );
                map
            },
        },
        ethereum: Default::default(),
        dynamic_fee: Default::default(),
        base_fee: Default::default(),
    }
}
