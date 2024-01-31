use serde::{Deserialize, Serialize};
use sp_core::{bytes::from_hex, ecdsa, H160, H256, U256};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use stability_runtime::{AccountId, GenesisConfig, Precompiles, ValidatorFeeSelectorConfig};
use std::{collections::BTreeMap, str::FromStr, vec};
// Substrate
use sp_core::{crypto::Ss58Codec, storage::Storage, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
// Frontier
use stability_runtime::{opaque::SessionKeys, EnableManualSeal, Signature};

pub mod alphanet;
pub mod betanet;
pub mod dev;
pub mod testnet;
pub mod mainnet;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt>;

pub type AuraId = stbl_core_primitives::aura::Public;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
	/// Genesis config.
	pub genesis_config: GenesisConfig,
	/// The flag that if enable manual-seal mode.
	pub enable_manual_seal: Option<bool>,
}

impl sp_runtime::BuildStorage for DevGenesisExt {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		BasicExternalities::execute_with_storage(storage, || {
			if let Some(enable_manual_seal) = &self.enable_manual_seal {
				EnableManualSeal::set(enable_manual_seal);
			}
		});
		self.genesis_config.assimilate_storage(storage)
	}
}

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
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<ecdsa::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub fn get_authority_from_pubkeys(
	ecdsa_key_str: &str,
	ed_pubkey: &str,
) -> (AccountId, AuraId, GrandpaId) {
	let ecdsa_pubkey = ecdsa::Public::from_string(ecdsa_key_str).unwrap();
	(
		(account::EthereumSigner::from(ecdsa_pubkey).into_account()),
		AuraId::from_string(ecdsa_key_str).unwrap(),
		GrandpaId::from_string(ed_pubkey).unwrap(),
	)
}

/// Configure initial storage state for FRAME modules.
pub fn base_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	members: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	use stability_runtime::{
		AuraConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, SessionConfig,
		SupportedTokensManagerConfig, SystemConfig, TechCommitteeCollectiveConfig,
		ValidatorSetConfig,
	};
	let initial_default_token =
		H160::from_str("0x261FB2d971eFBBFd027A9C9Cebb8548Cf7d0d2d5").expect("invalid address");
	let initial_default_conversion_rate_controller =
		H160::from_str("0x444212d6E4827893A70d19921E383130281Cda4a").expect("invalid address");
	let main_account =
		H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5").expect("invalid address");
	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		transaction_payment: Default::default(),

		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.1.clone(), x.2.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| x.0).collect(),
			max_epochs_missed: 5.into(),
		},
		// Consensus
		aura: AuraConfig {
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		tech_committee_collective: TechCommitteeCollectiveConfig {
			phantom: Default::default(),
			members: members.clone(),
		},
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
				Precompiles::used_addresses()
					.into_iter()
					.for_each(|addr| {
						map.insert(
							H160(addr.0),
							fp_evm::GenesisAccount {
								nonce: Default::default(),
								balance: Default::default(),
								storage: Default::default(),
								code: revert_bytecode.clone(),
							},
						);
					});
				map.insert(initial_default_token, fp_evm::GenesisAccount {
					nonce: Default::default(),
					balance: Default::default(),
					storage: {
						let mut storage = BTreeMap::new();
						let initial_default_token_balance = H256::from_str("0x00000000000000000000000000000000000000000000d3c21bcecceda1000000").expect("invalid hex storage value"); // 10M
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000002").expect("invalid hex storage key"), initial_default_token_balance); // Total Supply
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000003").expect("invalid hex storage key"), H256::from_str("0x53746162696c6974792047617320546f6b656e00000000000000000000000026").expect("invalid hex storage value")); // Name
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000004").expect("invalid hex storage key"), H256::from_str("0x5347540000000000000000000000000000000000000000000000000000000006").expect("invalid hex storage value")); // Symbol
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000005").expect("invalid hex storage key"), H256::from_str("0x000000000000000000000000af537bd156c7e548d0bf2cd43168dabf7af2feb5").expect("invalid hex storage value")); // Owner
						storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000006").expect("invalid hex storage key"), H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hex storage value")); // Transfers Block Flag
						storage.insert(stbl_tools::eth::get_storage_address_for_mapping(main_account, H256::from_low_u64_be(0)), initial_default_token_balance);
						storage
					},
					code: from_hex("0x608060405234801561001057600080fd5b506004361061012c5760003560e01c806379ba5097116100ad578063a457c2d711610071578063a457c2d71461030d578063a9059cbb1461033d578063dd62ed3e1461036d578063e30c39781461039d578063f2fde38b146103bb5761012c565b806379ba50971461028d5780637dcfd3d5146102975780638da5cb5b146102b557806395d89b41146102d35780639dc29fac146102f15761012c565b806339509351116100f457806339509351146101eb5780633a5500481461021b57806340c10f191461023757806370a0823114610253578063715018a6146102835761012c565b806306fdde0314610131578063095ea7b31461014f57806318160ddd1461017f57806323b872dd1461019d578063313ce567146101cd575b600080fd5b6101396103d7565b6040516101469190611371565b60405180910390f35b6101696004803603810190610164919061142c565b610469565b6040516101769190611487565b60405180910390f35b61018761048c565b60405161019491906114b1565b60405180910390f35b6101b760048036038101906101b291906114cc565b610496565b6040516101c49190611487565b60405180910390f35b6101d56104c5565b6040516101e2919061153b565b60405180910390f35b6102056004803603810190610200919061142c565b6104ce565b6040516102129190611487565b60405180910390f35b61023560048036038101906102309190611582565b610505565b005b610251600480360381019061024c919061142c565b61052a565b005b61026d600480360381019061026891906115af565b610540565b60405161027a91906114b1565b60405180910390f35b61028b610588565b005b61029561059c565b005b61029f610629565b6040516102ac9190611487565b60405180910390f35b6102bd61063c565b6040516102ca91906115eb565b60405180910390f35b6102db610666565b6040516102e89190611371565b60405180910390f35b61030b6004803603810190610306919061142c565b6106f8565b005b6103276004803603810190610322919061142c565b61070e565b6040516103349190611487565b60405180910390f35b6103576004803603810190610352919061142c565b610785565b6040516103649190611487565b60405180910390f35b61038760048036038101906103829190611606565b6107a8565b60405161039491906114b1565b60405180910390f35b6103a561082f565b6040516103b291906115eb565b60405180910390f35b6103d560048036038101906103d091906115af565b610859565b005b6060600380546103e690611675565b80601f016020809104026020016040519081016040528092919081815260200182805461041290611675565b801561045f5780601f106104345761010080835404028352916020019161045f565b820191906000526020600020905b81548152906001019060200180831161044257829003601f168201915b5050505050905090565b6000806104746109cc565b90506104818185856109d4565b600191505092915050565b6000600254905090565b6000806104a16109cc565b90506104ae858285610b9d565b6104b9858585610c29565b60019150509392505050565b60006012905090565b6000806104d96109cc565b90506104fa8185856104eb85896107a8565b6104f591906116d5565b6109d4565b600191505092915050565b61050d610c8f565b80600660146101000a81548160ff02191690831515021790555050565b610532610c8f565b61053c8282610d0d565b5050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050919050565b610590610c8f565b61059a6000610e63565b565b60006105a66109cc565b90508073ffffffffffffffffffffffffffffffffffffffff166105c761082f565b73ffffffffffffffffffffffffffffffffffffffff161461061d576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016106149061177b565b60405180910390fd5b61062681610e63565b50565b600660149054906101000a900460ff1681565b6000600560009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b60606004805461067590611675565b80601f01602080910402602001604051908101604052809291908181526020018280546106a190611675565b80156106ee5780601f106106c3576101008083540402835291602001916106ee565b820191906000526020600020905b8154815290600101906020018083116106d157829003601f168201915b5050505050905090565b610700610c8f565b61070a8282610e94565b5050565b6000806107196109cc565b9050600061072782866107a8565b90508381101561076c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016107639061180d565b60405180910390fd5b61077982868684036109d4565b60019250505092915050565b6000806107906109cc565b905061079d818585610c29565b600191505092915050565b6000600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b6000600660009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b610861610c8f565b80600660006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508073ffffffffffffffffffffffffffffffffffffffff166108c161063c565b73ffffffffffffffffffffffffffffffffffffffff167f38d16b8cac22d99fc7c124b9cd0de2d3fa1faef420bfe791d8c362d765e2270060405160405180910390a350565b6000600560009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905081600560006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055508173ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff167f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e060405160405180910390a35050565b600033905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1603610a43576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610a3a9061189f565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610ab2576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610aa990611931565b60405180910390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92583604051610b9091906114b1565b60405180910390a3505050565b6000610ba984846107a8565b90507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8114610c235781811015610c15576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610c0c9061199d565b60405180910390fd5b610c2284848484036109d4565b5b50505050565b60001515600660149054906101000a900460ff16151514610c7f576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610c7690611a2f565b60405180910390fd5b610c8a838383611061565b505050565b610c976109cc565b73ffffffffffffffffffffffffffffffffffffffff16610cb561063c565b73ffffffffffffffffffffffffffffffffffffffff1614610d0b576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610d0290611a9b565b60405180910390fd5b565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610d7c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610d7390611b07565b60405180910390fd5b610d88600083836112d7565b8060026000828254610d9a91906116d5565b92505081905550806000808473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508173ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef83604051610e4b91906114b1565b60405180910390a3610e5f600083836112dc565b5050565b600660006101000a81549073ffffffffffffffffffffffffffffffffffffffff0219169055610e9181610906565b50565b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1603610f03576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610efa90611b99565b60405180910390fd5b610f0f826000836112d7565b60008060008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905081811015610f95576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610f8c90611c2b565b60405180910390fd5b8181036000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000208190555081600260008282540392505081905550600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8460405161104891906114b1565b60405180910390a361105c836000846112dc565b505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16036110d0576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016110c790611cbd565b60405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff160361113f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161113690611d4f565b60405180910390fd5b61114a8383836112d7565b60008060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020549050818110156111d0576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016111c790611de1565b60405180910390fd5b8181036000808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002081905550816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040516112be91906114b1565b60405180910390a36112d18484846112dc565b50505050565b505050565b505050565b600081519050919050565b600082825260208201905092915050565b60005b8381101561131b578082015181840152602081019050611300565b60008484015250505050565b6000601f19601f8301169050919050565b6000611343826112e1565b61134d81856112ec565b935061135d8185602086016112fd565b61136681611327565b840191505092915050565b6000602082019050818103600083015261138b8184611338565b905092915050565b600080fd5b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b60006113c382611398565b9050919050565b6113d3816113b8565b81146113de57600080fd5b50565b6000813590506113f0816113ca565b92915050565b6000819050919050565b611409816113f6565b811461141457600080fd5b50565b60008135905061142681611400565b92915050565b6000806040838503121561144357611442611393565b5b6000611451858286016113e1565b925050602061146285828601611417565b9150509250929050565b60008115159050919050565b6114818161146c565b82525050565b600060208201905061149c6000830184611478565b92915050565b6114ab816113f6565b82525050565b60006020820190506114c660008301846114a2565b92915050565b6000806000606084860312156114e5576114e4611393565b5b60006114f3868287016113e1565b9350506020611504868287016113e1565b925050604061151586828701611417565b9150509250925092565b600060ff82169050919050565b6115358161151f565b82525050565b6000602082019050611550600083018461152c565b92915050565b61155f8161146c565b811461156a57600080fd5b50565b60008135905061157c81611556565b92915050565b60006020828403121561159857611597611393565b5b60006115a68482850161156d565b91505092915050565b6000602082840312156115c5576115c4611393565b5b60006115d3848285016113e1565b91505092915050565b6115e5816113b8565b82525050565b600060208201905061160060008301846115dc565b92915050565b6000806040838503121561161d5761161c611393565b5b600061162b858286016113e1565b925050602061163c858286016113e1565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b6000600282049050600182168061168d57607f821691505b6020821081036116a05761169f611646565b5b50919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006116e0826113f6565b91506116eb836113f6565b9250828201905080821115611703576117026116a6565b5b92915050565b7f4f776e61626c6532537465703a2063616c6c6572206973206e6f74207468652060008201527f6e6577206f776e65720000000000000000000000000000000000000000000000602082015250565b60006117656029836112ec565b915061177082611709565b604082019050919050565b6000602082019050818103600083015261179481611758565b9050919050565b7f45524332303a2064656372656173656420616c6c6f77616e63652062656c6f7760008201527f207a65726f000000000000000000000000000000000000000000000000000000602082015250565b60006117f76025836112ec565b91506118028261179b565b604082019050919050565b60006020820190508181036000830152611826816117ea565b9050919050565b7f45524332303a20617070726f76652066726f6d20746865207a65726f2061646460008201527f7265737300000000000000000000000000000000000000000000000000000000602082015250565b60006118896024836112ec565b91506118948261182d565b604082019050919050565b600060208201905081810360008301526118b88161187c565b9050919050565b7f45524332303a20617070726f766520746f20746865207a65726f20616464726560008201527f7373000000000000000000000000000000000000000000000000000000000000602082015250565b600061191b6022836112ec565b9150611926826118bf565b604082019050919050565b6000602082019050818103600083015261194a8161190e565b9050919050565b7f45524332303a20696e73756666696369656e7420616c6c6f77616e6365000000600082015250565b6000611987601d836112ec565b915061199282611951565b602082019050919050565b600060208201905081810360008301526119b68161197a565b9050919050565b7f53746162696c697479476173546f6b656e3a205472616e73666572732061726560008201527f20626c6f636b6564000000000000000000000000000000000000000000000000602082015250565b6000611a196028836112ec565b9150611a24826119bd565b604082019050919050565b60006020820190508181036000830152611a4881611a0c565b9050919050565b7f4f776e61626c653a2063616c6c6572206973206e6f7420746865206f776e6572600082015250565b6000611a856020836112ec565b9150611a9082611a4f565b602082019050919050565b60006020820190508181036000830152611ab481611a78565b9050919050565b7f45524332303a206d696e7420746f20746865207a65726f206164647265737300600082015250565b6000611af1601f836112ec565b9150611afc82611abb565b602082019050919050565b60006020820190508181036000830152611b2081611ae4565b9050919050565b7f45524332303a206275726e2066726f6d20746865207a65726f2061646472657360008201527f7300000000000000000000000000000000000000000000000000000000000000602082015250565b6000611b836021836112ec565b9150611b8e82611b27565b604082019050919050565b60006020820190508181036000830152611bb281611b76565b9050919050565b7f45524332303a206275726e20616d6f756e7420657863656564732062616c616e60008201527f6365000000000000000000000000000000000000000000000000000000000000602082015250565b6000611c156022836112ec565b9150611c2082611bb9565b604082019050919050565b60006020820190508181036000830152611c4481611c08565b9050919050565b7f45524332303a207472616e736665722066726f6d20746865207a65726f20616460008201527f6472657373000000000000000000000000000000000000000000000000000000602082015250565b6000611ca76025836112ec565b9150611cb282611c4b565b604082019050919050565b60006020820190508181036000830152611cd681611c9a565b9050919050565b7f45524332303a207472616e7366657220746f20746865207a65726f206164647260008201527f6573730000000000000000000000000000000000000000000000000000000000602082015250565b6000611d396023836112ec565b9150611d4482611cdd565b604082019050919050565b60006020820190508181036000830152611d6881611d2c565b9050919050565b7f45524332303a207472616e7366657220616d6f756e742065786365656473206260008201527f616c616e63650000000000000000000000000000000000000000000000000000602082015250565b6000611dcb6026836112ec565b9150611dd682611d6f565b604082019050919050565b60006020820190508181036000830152611dfa81611dbe565b905091905056fea264697066735822122067edfa4a59d6e71c5723c5c965bf4ca5cde4893d8992042582714ea5d36b55ce64736f6c63430008120033").expect("invalid hex"),
				});
				map.insert(
					main_account,
					fp_evm::GenesisAccount {
						nonce: U256::from(1),
						balance: Default::default(),
						storage: Default::default(),
						code: Default::default(),
					},
				);
				map.insert(
					initial_default_conversion_rate_controller,
					fp_evm::GenesisAccount {
						nonce: Default::default(),
						balance: Default::default(),
						storage: {
							let mut storage = BTreeMap::new();
							storage.insert(H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").expect("invalid hex storage key"), main_account.into());
							storage
						},
						code: from_hex("0x608060405234801561001057600080fd5b506004361061004c5760003560e01c80638da5cb5b14610051578063c5c3ab451461006f578063d9e1fc141461008b578063ff47b5a6146100bc575b600080fd5b6100596100ed565b6040516100669190610407565b60405180910390f35b61008960048036038101906100849190610373565b610111565b005b6100a560048036038101906100a091906102f3565b61020f565b6040516100b3929190610442565b60405180910390f35b6100d660048036038101906100d19190610320565b610233565b6040516100e4929190610442565b60405180910390f35b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff161461019f576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161019690610422565b60405180910390fd5b604051806040016040528083815260200182815250600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000820151816000015560208201518160010155905050505050565b60016020528060005260406000206000915090508060000154908060010154905082565b6000806000600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206040518060400160405290816000820154815260200160018201548152505090506000808260200151149050806102b757816000015182602001516102bb565b6001805b935093505050935093915050565b6000813590506102d88161050c565b92915050565b6000813590506102ed81610523565b92915050565b600060208284031215610309576103086104b8565b5b6000610317848285016102c9565b91505092915050565b600080600060608486031215610339576103386104b8565b5b6000610347868287016102c9565b9350506020610358868287016102c9565b9250506040610369868287016102c9565b9150509250925092565b60008060006060848603121561038c5761038b6104b8565b5b600061039a868287016102c9565b93505060206103ab868287016102de565b92505060406103bc868287016102de565b9150509250925092565b6103cf8161047c565b82525050565b60006103e260218361046b565b91506103ed826104bd565b604082019050919050565b610401816104ae565b82525050565b600060208201905061041c60008301846103c6565b92915050565b6000602082019050818103600083015261043b816103d5565b9050919050565b600060408201905061045760008301856103f8565b61046460208301846103f8565b9392505050565b600082825260208201905092915050565b60006104878261048e565b9050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b600080fd5b7f436f6e76657273696f6e526174654d616e616765723a204f4e4c595f4f574e4560008201527f5200000000000000000000000000000000000000000000000000000000000000602082015250565b6105158161047c565b811461052057600080fd5b50565b61052c816104ae565b811461053757600080fd5b5056fea2646970667358221220b2777c2a4814eaee2b96fedfd2cabf6a8cf8b3cd19bbf9d9255533266aee23d564736f6c63430008070033").expect("invalid hex"),
					},
				);
				map
			},
		},
		ethereum: Default::default(),
		base_fee: Default::default(),
		supported_tokens_manager: SupportedTokensManagerConfig {
			initial_default_token,
			initial_default_token_slot: H256::zero(),
		},
		validator_fee_selector: ValidatorFeeSelectorConfig {
			initial_default_conversion_rate_controller,
		},
		dnt_fee_controller: Default::default(),
	}
}
