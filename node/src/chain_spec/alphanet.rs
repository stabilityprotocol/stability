// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use super::{base_genesis, get_authority_from_pubkeys, ChainSpec};
use sc_service::ChainType;
use stability_runtime::WASM_BINARY;
use std::vec;

pub fn alphanet_config() -> ChainSpec {
	ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
		.with_name("Alphanet")
		.with_id("alphanet")
		.with_chain_type(ChainType::Live)
		.with_genesis_config_patch(base_genesis(
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
				get_authority_from_pubkeys(
					"KWBJUVzDvXYKakKX1wuHKqxxH5qg751fHwtaG15KYpuPReU9x",
					"5FqhzMYnwsDx4uZYiGVXxJWqvtGwTgMEgYvziEkihE6tfov7",
				),
			],
			vec![crate::chain_spec::get_account_id_from_public(
				"KWECfQF69Vr61qop6NVpesYrnw5WRS4M816286K7NUuVAn2zd",
			)],
			20180427,
			false,
		))
		.build()
}
