// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

//! A collection of node-specific RPC methods.

use std::sync::Arc;

use futures::channel::mpsc;
use jsonrpsee::RpcModule;
// Substrate
use sc_client_api::{
	backend::{Backend, StorageProvider},
	client::BlockchainEvents,
	AuxStore, UsageProvider,
};
use sc_consensus_manual_seal::rpc::EngineCommand;
use sc_rpc::SubscriptionTaskExecutor;
use sc_rpc_api::DenyUnsafe;
use sc_service::TransactionPool;
use sc_transaction_pool::ChainApi;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::H256;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::traits::Header as HeaderT;
use stbl_primitives_zero_gas_transactions_api::ZeroGasTransactionApi;
// Runtime
use stability_runtime::{AccountId, Balance, Hash, Nonce};
use tracing::RpcRequesters;

mod eth;
pub use self::eth::{create_eth, EthDeps};

pub mod tracing;
pub use self::tracing::*;

/// Full client dependencies.
pub struct FullDeps<B: BlockT, C, P, A: ChainApi, CT, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// Manual seal command sink
	pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
	/// Ethereum-compatibility specific dependencies.
	pub eth: EthDeps<B, C, P, A, CT, CIDP>,
}

pub struct TracingConfig {
	pub tracing_requesters: RpcRequesters,
	pub trace_filter_max_count: u32,
}

pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<B, C, BE> fc_rpc::EthConfig<B, C> for DefaultEthConfig<C, BE>
where
	B: BlockT<Hash = H256>,
	C: StorageProvider<B, BE> + Sync + Send + 'static,
	BE: Backend<B> + 'static,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride =
		fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<B, C, BE>;
}

/// Instantiate all Full RPC extensions.
pub fn create_full<B, C, P, BE, A, CT, CIDP>(
	deps: FullDeps<B, C, P, A, CT, CIDP>,
	subscription_task_executor: SubscriptionTaskExecutor,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<B>,
		>,
	>,
	optional_tracing_config: Option<TracingConfig>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	B: BlockT<Hash = H256>,
	B::Header: HeaderT<Number = u32>,
	C: CallApiAt<B> + ProvideRuntimeApi<B>,
	C::Api: sp_block_builder::BlockBuilder<B>,
	C::Api: sp_consensus_aura::AuraApi<B, stbl_core_primitives::aura::Public>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<B, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<B, Balance>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<B>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
	C::Api: ZeroGasTransactionApi<B>,
	C::Api: stability_rpc::StabilityRpcRuntimeApi<B>,
	C::Api: moonbeam_rpc_primitives_debug::DebugRuntimeApi<B>,
	C::Api: moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<B>,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE>,
	BE: Backend<B> + 'static,
	P: TransactionPool<Block = B> + 'static,
	A: ChainApi<Block = B> + 'static,
	CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
	CT: fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
	use stability_rpc::{StabilityRpc, StabilityRpcEndpointsServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		command_sink,
		eth,
	} = deps;

	io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	io.merge(StabilityRpc::new(client.clone(), pool.clone()).into_rpc())?;

	if let Some(command_sink) = command_sink {
		io.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}

	// Ethereum compatibility RPCs
	let io = create_eth::<_, _, _, _, _, _, _, DefaultEthConfig<C, BE>>(
		io,
		eth,
		subscription_task_executor,
		pubsub_notification_sinks,
		optional_tracing_config,
	)?;

	Ok(io)
}
