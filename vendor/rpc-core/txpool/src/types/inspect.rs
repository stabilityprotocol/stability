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

use crate::GetT;
use ethereum::{TransactionAction, TransactionV2 as EthereumTransaction};
use ethereum_types::{H160, H256, U256};
use serde::{Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct Summary {
	pub to: Option<H160>,
	pub value: U256,
	pub gas: U256,
	pub gas_price: U256,
}

impl Serialize for Summary {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let res = format!(
			"0x{:x}: {} wei + {} gas x {} wei",
			self.to.unwrap_or_default(),
			self.value,
			self.gas,
			self.gas_price
		);
		serializer.serialize_str(&res)
	}
}

impl GetT for Summary {
	fn get(_hash: H256, _from_address: H160, txn: &EthereumTransaction) -> Self {
		let (action, value, gas_price, gas_limit) = match txn {
			EthereumTransaction::Legacy(t) => (t.action, t.value, t.gas_price, t.gas_limit),
			EthereumTransaction::EIP2930(t) => (t.action, t.value, t.gas_price, t.gas_limit),
			EthereumTransaction::EIP1559(t) => (t.action, t.value, t.max_fee_per_gas, t.gas_limit),
		};
		Self {
			to: match action {
				TransactionAction::Call(to) => Some(to),
				_ => None,
			},
			value,
			gas_price,
			gas: gas_limit,
		}
	}
}
