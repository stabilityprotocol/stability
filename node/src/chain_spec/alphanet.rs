use super::{base_genesis, get_authority_from_pubkeys, get_key_sr, ChainSpec};
use sc_service::ChainType;
use sp_core::{crypto::Ss58Codec, H160};
use sp_runtime::traits::{IdentifyAccount, Verify};
use stability_runtime::{AccountId, Signature, WASM_BINARY};
use std::{str::FromStr, vec};

type AccountPublic = <Signature as Verify>::Signer;

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
