#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait ValidatorHealth<AccountId>
	where
		AccountId: codec::Codec
	{
		fn convert_add_validator_again_transaction(
			validator: AccountId,
			signature: Vec<u8>,
		) -> <Block as BlockT>::Extrinsic;

		fn generate_validator_message(
			validator: AccountId,
		) -> Vec<u8>;
	}
}
