#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	#[api_version(1)]
	pub trait ZeroGasTransactionApi {
		fn convert_zero_gas_transaction(transaction: fp_ethereum::Transaction, validator_signature: Vec<u8>) -> <Block as BlockT>::Extrinsic;
	}
}
