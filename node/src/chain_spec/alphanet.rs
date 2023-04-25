use super::{base_genesis, get_authority_from_pubkeys, ChainSpec};
use sc_service::ChainType;
use sp_application_crypto::Ss58Codec;
use sp_core::ecdsa;
use sp_runtime::traits::{IdentifyAccount, Verify};
use stability_runtime::{AccountId, Signature, WASM_BINARY};
use std::vec;

type AccountPublic = <Signature as Verify>::Signer;

fn get_account_id_from_public(pubkey: &str) -> AccountId {
	AccountPublic::from(ecdsa::Public::from_string(pubkey).unwrap()).into_account()
}

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
					get_authority_from_pubkeys(
						"KWECfQF69Vr61qop6NVpesYrnw5WRS4M816286K7NUuVAn2zd",
						"5FviP577ihFCP4n8jCnrd38dQDCn2VeM5DAoYNEHbPy7JtWz",
					),
					get_authority_from_pubkeys(
						"KW5B2djwfWnVUPjZALW9NjKPkYc5wA1LSXmYD7HB2QeNoyBX1",
						"5FqDv66PJL7TtC49CitcfGNokKJjbL3nwDRmYnQ66BJttUrw",
					),
					get_authority_from_pubkeys(
						"KWAefjXz8rjkX23DYt1tdjUoz9E8PPPQaA5SDULYc1mPpyg6i",
						"5GUVATr2DwH51tnqaUEuBtuHA4bLnoWBAkauDxDafMukpZAZ",
					),
					get_authority_from_pubkeys(
						"KWBJnoEoDMniHfxEC2iMv5xLQNwVHV7GdZPCD31eiiLi4niHt",
						"5FiEnbnj7VV5CWAtbJXtZjCkiQTBBRqyb8MgXEkydW1SfLiJ",
					),
					get_authority_from_pubkeys(
						"KW7fmVoR3DnYBEX8DwBfPZR2QBLf4uTQvXNm7zweVRWvXqJyt",
						"5GzRrcmG4kztd31FPWEcr51B3Jd2GZPh6ZjpxwzymopHuViN",
					),
				],
				vec![get_account_id_from_public(
					"KWECfQF69Vr61qop6NVpesYrnw5WRS4M816286K7NUuVAn2zd",
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
