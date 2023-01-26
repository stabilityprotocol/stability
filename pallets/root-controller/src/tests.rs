use super::*;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use mock::{
    new_test_ext, AllowedAccountId, Logger, LoggerCall, NotAllowedAccountId, RootController,
    RuntimeCall, RuntimeEvent as TestEvent, RuntimeOrigin, System,
};

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
