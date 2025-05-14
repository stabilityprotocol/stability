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

mod content;
mod inspect;

use ethereum::TransactionV2 as EthereumTransaction;
use ethereum_types::{H160, H256, U256};
use serde::Serialize;
use std::collections::HashMap;

pub use self::content::Transaction;
pub use self::inspect::Summary;

pub type TransactionMap<T> = HashMap<H160, HashMap<U256, T>>;

#[derive(Debug, Clone, Serialize)]
pub struct TxPoolResult<T: Serialize> {
	pub pending: T,
	pub queued: T,
}

pub trait Get {
	fn get(hash: H256, from_address: H160, txn: &EthereumTransaction) -> Self;
}
