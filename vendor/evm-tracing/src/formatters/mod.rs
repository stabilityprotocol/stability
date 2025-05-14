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

pub mod blockscout;
pub mod call_tracer;
pub mod raw;
pub mod trace_filter;

pub use blockscout::Formatter as Blockscout;
pub use call_tracer::Formatter as CallTracer;
pub use raw::Formatter as Raw;
pub use trace_filter::Formatter as TraceFilter;

use evm_tracing_events::Listener;
use serde::Serialize;

pub trait ResponseFormatter {
	type Listener: Listener;
	type Response: Serialize;

	fn format(listener: Self::Listener) -> Option<Self::Response>;
}
