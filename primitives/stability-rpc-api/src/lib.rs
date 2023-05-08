#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait StabilityRpcApi {
		fn get_supported_tokens() -> Vec<H160>;

		fn get_validator_list() -> Vec<H160>;

		fn get_user_fee_token(account: H160) -> Result<H160, sp_runtime::DispatchError>;

		fn set_user_fee_token(token: H160) -> Result<(), sp_runtime::DispatchError>;
	}
}
