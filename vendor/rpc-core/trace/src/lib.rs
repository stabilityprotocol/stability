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

use ethereum_types::H160;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use moonbeam_client_evm_tracing::types::block::TransactionTrace;
use moonbeam_rpc_core_types::RequestBlockId;
use serde::Deserialize;

#[rpc(server)]
#[jsonrpsee::core::async_trait]
pub trait Trace {
	#[method(name = "trace_filter")]
	async fn filter(&self, filter: FilterRequest) -> RpcResult<Vec<TransactionTrace>>;
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
	/// (optional?) From this block.
	pub from_block: Option<RequestBlockId>,

	/// (optional?) To this block.
	pub to_block: Option<RequestBlockId>,

	/// (optional) Sent from these addresses.
	pub from_address: Option<Vec<H160>>,

	/// (optional) Sent to these addresses.
	pub to_address: Option<Vec<H160>>,

	/// (optional) The offset trace number
	pub after: Option<u32>,

	/// (optional) Integer number of traces to display in a batch.
	pub count: Option<u32>,
}
