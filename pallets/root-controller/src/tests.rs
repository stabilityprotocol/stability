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

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	new_test_ext, AllowedAccountId, Logger, LoggerCall, NotAllowedAccountId, RootController,
	RuntimeCall, RuntimeEvent as TestEvent, RuntimeOrigin, System,
};
use sp_runtime::DispatchError;

#[test]
fn test_setup_works() {
	new_test_ext().execute_with(|| {
		assert!(Logger::i32_log().is_empty());
	});
}

#[test]
fn test_dispatch_as_root_signed_from_allowed_origin() {
	new_test_ext().execute_with(|| {
		let call = Box::new(RuntimeCall::Logger(LoggerCall::privileged_i32_log {
			i: 42,
		}));
		assert_ok!(RootController::dispatch_as_root(
			RuntimeOrigin::signed(AllowedAccountId::get()),
			call
		));
		assert_eq!(Logger::i32_log(), vec![42i32]);
	});
}

#[test]
fn test_dispatch_as_root_signed_from_not_allowed() {
	new_test_ext().execute_with(|| {
		let call = Box::new(RuntimeCall::Logger(LoggerCall::privileged_i32_log {
			i: 42,
		}));
		assert_noop!(
			RootController::dispatch_as_root(
				RuntimeOrigin::signed(NotAllowedAccountId::get()),
				call
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_dispatch_as_root_emits_empty_when_dispatch_success() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let call = Box::new(RuntimeCall::Logger(LoggerCall::privileged_i32_log {
			i: 42,
		}));
		assert_ok!(RootController::dispatch_as_root(
			RuntimeOrigin::signed(AllowedAccountId::get()),
			call
		));

		System::assert_has_event(TestEvent::RootController(Event::DispatchAsRootOccurred {
			dispatch_result: Ok(()),
		}));
	});
}

#[test]
fn test_dispatch_as_root_emits_dispatch_error_when_dispatch_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let call = Box::new(RuntimeCall::Logger(LoggerCall::force_fail_log {}));
		assert_ok!(RootController::dispatch_as_root(
			RuntimeOrigin::signed(AllowedAccountId::get()),
			call
		));

		System::assert_has_event(TestEvent::RootController(Event::DispatchAsRootOccurred {
			dispatch_result: Err(DispatchError::BadOrigin),
		}));
	});
}
