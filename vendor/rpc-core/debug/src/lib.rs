// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/blob/master/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.
use ethereum::AccessListItem;
use ethereum_types::{H160, H256, U256};
use fc_rpc_core::types::Bytes;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use moonbeam_client_evm_tracing::types::single;
use moonbeam_rpc_core_types::RequestBlockId;
use serde::Deserialize;

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceParams {
	pub disable_storage: Option<bool>,
	pub disable_memory: Option<bool>,
	pub disable_stack: Option<bool>,
	/// Javascript tracer (we just check if it's Blockscout tracer string)
	pub tracer: Option<String>,
	pub timeout: Option<String>,
	pub tracer_config: Option<single::TraceCallConfig>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallParams {
	/// Sender
	pub from: Option<H160>,
	/// Recipient
	pub to: H160,
	/// Gas Price, legacy.
	pub gas_price: Option<U256>,
	/// Max BaseFeePerGas the user is willing to pay.
	pub max_fee_per_gas: Option<U256>,
	/// The miner's tip.
	pub max_priority_fee_per_gas: Option<U256>,
	/// Gas
	pub gas: Option<U256>,
	/// Value of transaction in wei
	pub value: Option<U256>,
	/// Additional data sent with transaction
	pub data: Option<Bytes>,
	/// Nonce
	pub nonce: Option<U256>,
	/// EIP-2930 access list
	pub access_list: Option<Vec<AccessListItem>>,
	/// EIP-2718 type
	#[serde(rename = "type")]
	pub transaction_type: Option<U256>,
}

#[rpc(server)]
#[jsonrpsee::core::async_trait]
pub trait Debug {
	#[method(name = "debug_traceTransaction")]
	async fn trace_transaction(
		&self,
		transaction_hash: H256,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace>;
	#[method(name = "debug_traceCall")]
	async fn trace_call(
		&self,
		call_params: TraceCallParams,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<single::TransactionTrace>;
	#[method(name = "debug_traceBlockByNumber", aliases = ["debug_traceBlockByHash"])]
	async fn trace_block(
		&self,
		id: RequestBlockId,
		params: Option<TraceParams>,
	) -> RpcResult<Vec<single::TransactionTrace>>;
}
