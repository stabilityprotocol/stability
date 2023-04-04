use super::{
	authority_keys_from_seed, base_genesis, get_account_id_from_seed, DevChainSpec, DevGenesisExt,
};
use sc_service::ChainType;
use sp_core::{sr25519, H160};
use stability_runtime::WASM_BINARY;
use std::str::FromStr;

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
