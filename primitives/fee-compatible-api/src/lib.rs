#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::Block as BlockT;

sp_api::decl_runtime_apis! {
	pub trait CompatibleFeeApi<AccountId> where
		AccountId: codec::Codec,  {
		fn is_compatible_fee(tx: <Block as BlockT>::Extrinsic, validator: AccountId) -> bool;
	}
}
