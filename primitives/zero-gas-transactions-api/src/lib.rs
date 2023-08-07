#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::Block as BlockT;

sp_api::decl_runtime_apis! {
	pub trait ZeroGasTransactionApi {
		fn convert_zero_gas_transaction(transaction: fp_ethereum::Transaction) -> <Block as BlockT>::Extrinsic;
	}
}
