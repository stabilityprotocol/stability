use super::{
	authority_keys_from_seed, base_genesis, get_account_id_from_seed, DevChainSpec, DevGenesisExt,
};
use sc_service::ChainType;
use sp_core::ecdsa;
use stability_runtime::WASM_BINARY;

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
				genesis_config: base_genesis(
					wasm_binary,
					// Sudo account
					// Pre-funded accounts
					vec![
						get_account_id_from_seed::<ecdsa::Public>("Alice"),
						get_account_id_from_seed::<ecdsa::Public>("Bob"),
						get_account_id_from_seed::<ecdsa::Public>("Alice//stash"),
						get_account_id_from_seed::<ecdsa::Public>("Bob//stash"),
					],
					// Initial PoA authorities
					vec![authority_keys_from_seed("Alice")],
					vec![get_account_id_from_seed::<ecdsa::Public>("Alice")],
					20180428,
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
