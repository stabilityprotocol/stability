use scale_codec::Codec;
// Substrate
use sc_executor::WasmExecutor;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay};

use crate::eth::EthCompatRuntimeApiCollection;

/// Full backend.
pub type FullBackend<B> = sc_service::TFullBackend<B>;
/// Full client.
pub type FullClient<B, RA, HF> = sc_service::TFullClient<B, RA, WasmExecutor<HF>>;

/// A set of APIs that every runtime must implement.
pub trait BaseRuntimeApiCollection<Block: BlockT>:
	sp_api::ApiExt<Block>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
	+ moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block>
{
}

impl<Block, Api> BaseRuntimeApiCollection<Block> for Api
where
	Block: BlockT,
	Api: sp_api::ApiExt<Block>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
		+ moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block>,
{
}

/// A set of APIs that template runtime must implement.
pub trait RuntimeApiCollection<
	Block: BlockT,
	AuraId: Codec,
	AccountId: Codec,
	Nonce: Codec,
	Balance: Codec + MaybeDisplay,
>:
	BaseRuntimeApiCollection<Block>
	+ EthCompatRuntimeApiCollection<Block>
	+ sp_consensus_aura::AuraApi<Block, stbl_core_primitives::aura::Public>
	+ sp_consensus_grandpa::GrandpaApi<Block>
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
	+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>
{
}

impl<Block, AuraId, AccountId, Nonce, Balance, Api>
	RuntimeApiCollection<Block, AuraId, AccountId, Nonce, Balance> for Api
where
	Block: BlockT,
	AuraId: Codec,
	AccountId: Codec,
	Nonce: Codec,
	Balance: Codec + MaybeDisplay,
	Api: BaseRuntimeApiCollection<Block>
		+ EthCompatRuntimeApiCollection<Block>
		+ sp_consensus_aura::AuraApi<Block, stbl_core_primitives::aura::Public>
		+ sp_consensus_grandpa::GrandpaApi<Block>
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>,
{
}
