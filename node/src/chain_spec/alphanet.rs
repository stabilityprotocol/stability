use super::{base_genesis, ChainSpec};
use sc_service::ChainType;
use sp_runtime::traits::Verify;
use stability_runtime::{AccountId, Signature, WASM_BINARY};
use std::{str::FromStr, vec};

pub fn alphanet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Alphanet",
		// ID
		"alphanet",
		ChainType::Live,
		move || {
			base_genesis(
				wasm_binary,
				vec![
					AccountId::from_str("0x76d2265F02C763Dc71A02f62B1Dce4d58b3bb7e2")
						.expect("Bad account id format"),
					AccountId::from_str("0x6e63A65a87E24046167a2ce1971B6b765BaD3293")
						.expect("Bad account id format"),
					AccountId::from_str("0x3dbCC2fbe676037579A59fD2412AA4DcAe11db97")
						.expect("Bad account id format"),
					AccountId::from_str("0x1db0Fd9B3Abf83C802869D0De696157e60b136e5")
						.expect("Bad account id format"),
					AccountId::from_str("0xf349108C149a99C133c662A6779E07d0e68340d6")
						.expect("Bad account id format"),
				],
				vec![
					AccountId::from_str("0x76d2265F02C763Dc71A02f62B1Dce4d58b3bb7e2")
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
