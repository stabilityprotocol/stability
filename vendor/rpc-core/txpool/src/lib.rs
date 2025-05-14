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

use ethereum_types::U256;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

mod types;

pub use crate::types::{Get as GetT, Summary, Transaction, TransactionMap, TxPoolResult};

#[rpc(server)]
pub trait TxPool {
	#[method(name = "txpool_content")]
	fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Transaction>>>;

	#[method(name = "txpool_inspect")]
	fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Summary>>>;

	#[method(name = "txpool_status")]
	fn status(&self) -> RpcResult<TxPoolResult<U256>>;
}
