#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait StabilityRpcApi {
		fn get_supported_tokens() -> Vec<H160>;

		fn get_delegated_transaction_current_nonce() -> u64;

		fn get_validator_list() -> Vec<H160>;
	}
}
