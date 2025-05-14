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

use super::blockscout::BlockscoutCallInner as CallInner;
use crate::listeners::call_list::Listener;
use crate::types::{
	block::{
		TransactionTrace, TransactionTraceAction, TransactionTraceOutput, TransactionTraceResult,
	},
	CallResult, CreateResult, CreateType,
};
use ethereum_types::H256;

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
	type Listener = Listener;
	type Response = Vec<TransactionTrace>;

	fn format(mut listener: Listener) -> Option<Vec<TransactionTrace>> {
		// Remove empty BTreeMaps pushed to `entries`.
		// I.e. InvalidNonce or other pallet_evm::runner exits
		listener.entries.retain(|x| !x.is_empty());
		let mut traces = Vec::new();
		for (eth_tx_index, entry) in listener.entries.iter().enumerate() {
			let mut tx_traces: Vec<_> = entry
				.into_iter()
				.map(|(_, trace)| match trace.inner.clone() {
					CallInner::Call {
						input,
						to,
						res,
						call_type,
					} => TransactionTrace {
						action: TransactionTraceAction::Call {
							call_type,
							from: trace.from,
							gas: trace.gas,
							input,
							to,
							value: trace.value,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: match res {
							CallResult::Output(output) => {
								TransactionTraceOutput::Result(TransactionTraceResult::Call {
									gas_used: trace.gas_used,
									output,
								})
							}
							CallResult::Error(error) => TransactionTraceOutput::Error(error),
						},
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
					CallInner::Create { init, res } => {
						TransactionTrace {
							action: TransactionTraceAction::Create {
								creation_method: CreateType::Create,
								from: trace.from,
								gas: trace.gas,
								init,
								value: trace.value,
							},
							// Can't be known here, must be inserted upstream.
							block_hash: H256::default(),
							// Can't be known here, must be inserted upstream.
							block_number: 0,
							output: match res {
								CreateResult::Success {
									created_contract_address_hash,
									created_contract_code,
								} => {
									TransactionTraceOutput::Result(TransactionTraceResult::Create {
										gas_used: trace.gas_used,
										code: created_contract_code,
										address: created_contract_address_hash,
									})
								}
								CreateResult::Error { error } => {
									TransactionTraceOutput::Error(error)
								}
							},
							subtraces: trace.subtraces,
							trace_address: trace.trace_address.clone(),
							// Can't be known here, must be inserted upstream.
							transaction_hash: H256::default(),
							transaction_position: eth_tx_index as u32,
						}
					}
					CallInner::SelfDestruct { balance, to } => TransactionTrace {
						action: TransactionTraceAction::Suicide {
							address: trace.from,
							balance,
							refund_address: to,
						},
						// Can't be known here, must be inserted upstream.
						block_hash: H256::default(),
						// Can't be known here, must be inserted upstream.
						block_number: 0,
						output: TransactionTraceOutput::Result(TransactionTraceResult::Suicide),
						subtraces: trace.subtraces,
						trace_address: trace.trace_address.clone(),
						// Can't be known here, must be inserted upstream.
						transaction_hash: H256::default(),
						transaction_position: eth_tx_index as u32,
					},
				})
				.collect();

			traces.append(&mut tx_traces);
		}
		Some(traces)
	}
}
