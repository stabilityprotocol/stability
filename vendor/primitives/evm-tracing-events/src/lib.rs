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

//! A Proxy in this context is an environmental trait implementor meant to be used for capturing
//! EVM trace events sent to a Host function from the Runtime. Works like:
//! - Runtime Api call `using` environmental.
//! - Runtime calls a Host function with some scale-encoded Evm event.
//! - Host function emits an additional event to this Listener.
//! - Proxy listens for the event and format the actual trace response.
//!
//! There are two proxy types: `Raw` and `CallList`.
//! - `Raw` - used for opcode-level traces.
//! - `CallList` - used for block tracing (stack of call stacks) and custom tracing outputs.
//!
//! The EVM event types may contain references and not implement Encode/Decode.
//! This module provide mirror types and conversion into them from the original events.

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

pub mod evm;
pub mod gasometer;
pub mod runtime;

pub use self::evm::EvmEvent;
pub use gasometer::GasometerEvent;
pub use runtime::RuntimeEvent;

use ethereum_types::{H160, U256};
use parity_scale_codec::{Decode, Encode};
use sp_runtime_interface::pass_by::PassByCodec;

environmental::environmental!(listener: dyn Listener + 'static);

pub fn using<R, F: FnOnce() -> R>(l: &mut (dyn Listener + 'static), f: F) -> R {
	listener::using(l, f)
}

/// Allow to configure which data of the Step event
/// we want to keep or discard. Not discarding the data requires cloning the data
/// in the runtime which have a significant cost for each step.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Default, PassByCodec)]
pub struct StepEventFilter {
	pub enable_stack: bool,
	pub enable_memory: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
	Evm(evm::EvmEvent),
	Gasometer(gasometer::GasometerEvent),
	Runtime(runtime::RuntimeEvent),
	CallListNew(),
}

impl Event {
	/// Access the global reference and call it's `event` method, passing the `Event` itself as
	/// argument.
	///
	/// This only works if we are `using` a global reference to a `Listener` implementor.
	pub fn emit(self) {
		listener::with(|listener| listener.event(self));
	}
}

/// Main trait to proxy emitted messages.
/// Used 2 times :
/// - Inside the runtime to proxy the events through the host functions
/// - Inside the client to forward those events to the client listener.
pub trait Listener {
	fn event(&mut self, event: Event);

	/// Allow the runtime to know which data should be discarded and not cloned.
	/// WARNING: It is only called once when the runtime tracing is instantiated to avoid
	/// performing many ext calls.
	fn step_event_filter(&self) -> StepEventFilter;
}

pub fn step_event_filter() -> Option<StepEventFilter> {
	let mut filter = None;
	listener::with(|listener| filter = Some(listener.step_event_filter()));
	filter
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Context {
	/// Execution address.
	pub address: H160,
	/// Caller of the EVM.
	pub caller: H160,
	/// Apparent value of the EVM.
	pub apparent_value: U256,
}

impl From<evm_runtime::Context> for Context {
	fn from(i: evm_runtime::Context) -> Self {
		Self {
			address: i.address,
			caller: i.caller,
			apparent_value: i.apparent_value,
		}
	}
}
