#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait StabilityRpcApi {
		fn get_supported_tokens() -> Vec<H160>;

		fn get_validator_list() -> Vec<H160>;

		fn get_active_validator_list() -> Vec<H160>;

		fn convert_sponsored_transaction(transaction: fp_ethereum::Transaction, meta_trx_sponsor: H160, meta_trx_sponsor_signature: Vec<u8>) -> <Block as BlockT>::Extrinsic;
	}
}
