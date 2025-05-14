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

use crate::listeners::raw::Listener;
use crate::types::single::TransactionTrace;

pub struct Formatter;

impl super::ResponseFormatter for Formatter {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn format(listener: Listener) -> Option<TransactionTrace> {
		if listener.remaining_memory_usage.is_none() {
			None
		} else {
			Some(TransactionTrace::Raw {
				struct_logs: listener.struct_logs,
				gas: listener.final_gas.into(),
				return_value: listener.return_value,
			})
		}
	}
}
