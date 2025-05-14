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

//! Substrate EVM tracing.
//!
//! The purpose of this crate is enable tracing the EVM opcode execution and will be used by
//! both Dapp developers - to get a granular view on their transactions - and indexers to access
//! the EVM callstack (internal transactions).
//!
//! Proxies EVM messages to the host functions.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod tracer {
	use evm_tracing_events::{EvmEvent, GasometerEvent, RuntimeEvent, StepEventFilter};
	use parity_scale_codec::Encode;

	use evm::tracing::{using as evm_using, EventListener as EvmListener};
	use evm_gasometer::tracing::{using as gasometer_using, EventListener as GasometerListener};
	use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener};
	use sp_std::{cell::RefCell, rc::Rc};

	struct ListenerProxy<T>(pub Rc<RefCell<T>>);
	impl<T: GasometerListener> GasometerListener for ListenerProxy<T> {
		fn event(&mut self, event: evm_gasometer::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
		fn event(&mut self, event: evm_runtime::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	impl<T: EvmListener> EvmListener for ListenerProxy<T> {
		fn event(&mut self, event: evm::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	pub struct EvmTracer {
		step_event_filter: StepEventFilter,
	}

	impl EvmTracer {
		pub fn new() -> Self {
			Self {
				step_event_filter: moonbeam_primitives_ext::moonbeam_ext::step_event_filter(),
			}
		}

		/// Setup event listeners and execute provided closure.
		///
		/// Consume the tracer and return it alongside the return value of
		/// the closure.
		pub fn trace<R, F: FnOnce() -> R>(self, f: F) {
			let wrapped = Rc::new(RefCell::new(self));

			let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
			let mut runtime = ListenerProxy(Rc::clone(&wrapped));
			let mut evm = ListenerProxy(Rc::clone(&wrapped));

			// Each line wraps the previous `f` into a `using` call.
			// Listening to new events results in adding one new line.
			// Order is irrelevant when registering listeners.
			let f = || runtime_using(&mut runtime, f);
			let f = || gasometer_using(&mut gasometer, f);
			let f = || evm_using(&mut evm, f);
			f();
		}

		pub fn emit_new() {
			moonbeam_primitives_ext::moonbeam_ext::call_list_new();
		}
	}

	impl EvmListener for EvmTracer {
		/// Proxies `evm::tracing::Event` to the host.
		fn event(&mut self, event: evm::tracing::Event) {
			let event: EvmEvent = event.into();
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::evm_event(message);
		}
	}

	impl GasometerListener for EvmTracer {
		/// Proxies `evm_gasometer::tracing::Event` to the host.
		fn event(&mut self, event: evm_gasometer::tracing::Event) {
			let event: GasometerEvent = event.into();
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::gasometer_event(message);
		}
	}

	impl RuntimeListener for EvmTracer {
		/// Proxies `evm_runtime::tracing::Event` to the host.
		fn event(&mut self, event: evm_runtime::tracing::Event) {
			let event = RuntimeEvent::from_evm_event(event, self.step_event_filter);
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::runtime_event(message);
		}
	}
}
