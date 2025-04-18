use super::{base_genesis, get_authority_from_pubkeys, ChainSpec};
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use stability_runtime::WASM_BINARY;
use std::{str::FromStr, vec};

pub fn gtn_config() -> ChainSpec {
	ChainSpec::builder(WASM_BINARY.expect("WASM not available"), Default::default())
		.with_name("GTN")
		.with_id("gtn")
		.with_chain_type(ChainType::Live)
		.with_genesis_config_patch(base_genesis(
			vec![
				get_authority_from_pubkeys(
					"KWCNZHPzYGqHqGaWHV5y49LFRxKjzQKjs6ipjsgMErdhHTESC",
					"5GkCaqZz6vg943Yp2fSM3SZhcu8bAvrH2fHuZCioMY16uRUA",
				),
				get_authority_from_pubkeys(
					"KWEHqG66UQsPoa1c5SFjkC4jcSuUwwqbAPbx1SrmhWaTHrQD2",
					"5Fyow7im5QHts5DuZ6tVipF6XEDhWgJxV8atCVaDNuqiGDht",
				),
				get_authority_from_pubkeys(
					"KWDM9bJLqQktNr4TSRfdh7iXTxLhiyTBjNAcdFNSeueTCd8dH",
					"5FNiZMiDSg2Emn4wLGLUqSyX7z2R482VktKMW4NXWuuQBMxf",
				),
				get_authority_from_pubkeys(
					"KW4hgPpFGdmtsfVTThf1UMCTBq9oZhSZXLQ7GpBZdGZmGxXgR",
					"5E2n24284g6gEjToh7ugY1fjoutHwTeTfQUpZZdsLw7w8LBM",
				),
				get_authority_from_pubkeys(
					"KW6wyS4h9snoJFSomxRESiKTysuPqm8CyxqpbU5oyX2UnCXbV",
					"5DLdFNVbWduLqZ68K8GJdnQiWk34KxVJvu7W7veu6kdKUvTU",
				),
				get_authority_from_pubkeys(
					"KW8BkuxEdR5AAZWJVUHGL2gmeXTCHot2hzukvxpvhqarNvBSS",
					"5EkLmLw4sPrzrt2JhimMrggzoyn2jLWaYbAh1yUw65D6xRDZ",
				),
				get_authority_from_pubkeys(
					"KWDQPKqrZY1kUGRsBWDme3hrWaPXExFkxZrc7Eh5SRxUGXYyK",
					"5He4DiUGM1Tp1nDtTVFsGD7DrarMkuMki7HCBy2TRPc3X3R3",
				),
			],
			vec![crate::chain_spec::get_account_id_from_public(
				"KWAWnNYYgJzqvZQ2LLYmTVya95oeyAXX76PZN86DjbQKp7Czw",
			)],
			101010,
			false
		))
		.with_boot_nodes(
			vec![
			MultiaddrWithPeerId::from_str("/dns4/s0.gtn.stabilityprotocol.com/tcp/30333/p2p/12D3KooWNGnx5ZbeCkea9HVR5TTMjA5hyPqhLCBdM9KejPXD7GvQ").unwrap(),
			MultiaddrWithPeerId::from_str("/dns4/s1.gtn.stabilityprotocol.com/tcp/30333/p2p/12D3KooWAcZMqS6TjCNXFaZ6a3u6sob87cG3f4E3yUhAwNmzCAWF").unwrap(),
			MultiaddrWithPeerId::from_str("/dns4/s2.gtn.stabilityprotocol.com/tcp/30333/p2p/12D3KooWC9QrhPSewpysp3LPiXxo9Go25a8VgGsKCFwhBrccJ21n").unwrap()
		]
		)
		.build()
}
