use scale_codec::Codec;
// Substrate
use sc_executor::{NativeExecutionDispatch, NativeVersion, WasmExecutor};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
// Local
use crate::eth::EthCompatRuntimeApiCollection;
use sp_runtime::traits::MaybeDisplay;
use stability_runtime::{opaque::Block, AccountId, Balance, Index, RuntimeApi};

/// Full backend.
pub type FullBackend = sc_service::TFullBackend<Block>;
/// Full client.
pub type FullClient = sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;

/// Only enable the benchmarking host functions when we actually want to benchmark.
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);
/// Otherwise we use empty host functions for ext host functions.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = moonbeam_primitives_ext::moonbeam_ext::HostFunctions;

pub struct TemplateRuntimeExecutor;
impl NativeExecutionDispatch for TemplateRuntimeExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		stability_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		stability_runtime::native_version()
	}
}

/// A set of APIs that every runtimes must implement.
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
	+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
	+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>
	+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>
	+ moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block>
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
		+ frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ stbl_primitives_fee_compatible_api::CompatibleFeeApi<Block, AccountId>
		+ stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi<Block>
		+ moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block>
		+ moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block>,
{
}
