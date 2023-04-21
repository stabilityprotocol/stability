#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait TokensApi {
		fn get_supported_tokens() -> Vec<H160>;
	}
}
