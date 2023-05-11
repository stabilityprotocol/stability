#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait EthExtensionRpcApi {
		fn convert_delegated_transaction(transaction: fp_ethereum::Transaction, meta_trx_nonce : u64, meta_trx_sponsor: H160, meta_trx_sponsor_signature: Vec<u8>) -> <Block as BlockT>::Extrinsic;
		fn get_sponsor_nonce(address: H160) -> u64;
	}
}
