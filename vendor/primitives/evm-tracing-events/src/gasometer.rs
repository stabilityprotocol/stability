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

use parity_scale_codec::{Decode, Encode};

#[derive(Debug, Default, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Snapshot {
	pub gas_limit: u64,
	pub memory_gas: u64,
	pub used_gas: u64,
	pub refunded_gas: i64,
}

impl Snapshot {
	pub fn gas(&self) -> u64 {
		self.gas_limit - self.used_gas - self.memory_gas
	}
}

#[cfg(feature = "evm-tracing")]
impl From<Option<evm_gasometer::Snapshot>> for Snapshot {
	fn from(i: Option<evm_gasometer::Snapshot>) -> Self {
		if let Some(i) = i {
			Self {
				gas_limit: i.gas_limit,
				memory_gas: i.memory_gas,
				used_gas: i.used_gas,
				refunded_gas: i.refunded_gas,
			}
		} else {
			Default::default()
		}
	}
}

#[derive(Debug, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub enum GasometerEvent {
	RecordCost {
		cost: u64,
		snapshot: Snapshot,
	},
	RecordRefund {
		refund: i64,
		snapshot: Snapshot,
	},
	RecordStipend {
		stipend: u64,
		snapshot: Snapshot,
	},
	RecordDynamicCost {
		gas_cost: u64,
		memory_gas: u64,
		gas_refund: i64,
		snapshot: Snapshot,
	},
	RecordTransaction {
		cost: u64,
		snapshot: Snapshot,
	},
}

#[cfg(feature = "evm-tracing")]
impl From<evm_gasometer::tracing::Event> for GasometerEvent {
	fn from(i: evm_gasometer::tracing::Event) -> Self {
		match i {
			evm_gasometer::tracing::Event::RecordCost { cost, snapshot } => Self::RecordCost {
				cost,
				snapshot: snapshot.into(),
			},
			evm_gasometer::tracing::Event::RecordRefund { refund, snapshot } => {
				Self::RecordRefund {
					refund,
					snapshot: snapshot.into(),
				}
			}
			evm_gasometer::tracing::Event::RecordStipend { stipend, snapshot } => {
				Self::RecordStipend {
					stipend,
					snapshot: snapshot.into(),
				}
			}
			evm_gasometer::tracing::Event::RecordDynamicCost {
				gas_cost,
				memory_gas,
				gas_refund,
				snapshot,
			} => Self::RecordDynamicCost {
				gas_cost,
				memory_gas,
				gas_refund,
				snapshot: snapshot.into(),
			},
			evm_gasometer::tracing::Event::RecordTransaction { cost, snapshot } => {
				Self::RecordTransaction {
					cost,
					snapshot: snapshot.into(),
				}
			}
		}
	}
}
