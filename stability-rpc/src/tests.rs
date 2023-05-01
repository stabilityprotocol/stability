use super::*;
use crate::mock::*;

use sp_core::H160;
use std::str::FromStr;
use std::sync::Arc;
use substrate_test_runtime_client::{self, runtime::Block};

sp_api::mock_impl_runtime_apis! {
	impl stability_rpc_api::StabilityRpcApi<Block> for TestRuntimeApi {
		fn get_supported_tokens() -> Vec<H160> {
			vec![H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5").expect("Bad account id format")]
		}

		fn get_validator_list() -> Vec<H160> {
			vec![H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5").expect("Bad account id format"),
			H160::from_str("0xf25F864329C44b2aA103De1dFf6fA020b85D8C07").expect("Bad account id format")]
		}
	}
}

#[tokio::test]
async fn get_supported_tokens_should_return_expected_addr() {
	let client = Arc::new(TestApi {});
	let api = StabilityRpc::<TestApi, Block>::new(client);
	let result = api.get_validator_list(None);
	assert_eq!(true, result.is_ok());
	let result_unwrap = result.unwrap().value as Vec<H160>;
	let expected: Vec<H160> = vec![
		H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5")
			.expect("Bad account id format"),
		H160::from_str("0xf25F864329C44b2aA103De1dFf6fA020b85D8C07")
			.expect("Bad account id format"),
	];
	assert_eq!(expected, result_unwrap);
}

#[tokio::test]
async fn get_validator_list_should_return_a_validator_list() {
	let client = Arc::new(TestApi {});
	let api = StabilityRpc::<TestApi, Block>::new(client);
	let result = api.get_supported_tokens(None);
	assert_eq!(true, result.is_ok());
	let result_unwrap = result.unwrap().value as Vec<H160>;
	let expected: Vec<H160> = vec![H160::from_str("0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5")
		.expect("Bad account id format")];
	assert_eq!(expected, result_unwrap);
}
