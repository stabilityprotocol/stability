use super::{
	authority_keys_from_seed, base_genesis, get_account_id_from_seed, DevChainSpec, DevGenesisExt,
};
use sc_service::ChainType;
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
					// Initial PoA authorities
					vec![authority_keys_from_seed("Alice")],
					vec![get_account_id_from_seed::<sp_core::ecdsa::Public>("Alice")],
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
