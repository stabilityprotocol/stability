use super::{authority_keys_from_seed, base_genesis, get_account_id_from_seed, ChainSpec};
use sc_chain_spec::Properties;
use sc_service::ChainType;
use stability_runtime::{SS58Prefix, WASM_BINARY};

fn properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("ss58Format".into(), SS58Prefix::get().into());
	properties
}

pub fn development_config(enable_manual_seal: bool) -> ChainSpec {
	ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
		.with_name("Development")
		.with_id("dev")
		.with_chain_type(ChainType::Development)
		.with_properties(properties())
		.with_genesis_config_patch(base_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![get_account_id_from_seed::<sp_core::ecdsa::Public>("Alice")],
			20180428,
			enable_manual_seal,
		))
		.build()
}
