use super::*;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, weights::Weight};
use mock::{
    new_test_ext, AllowedAccountId, Logger, LoggerCall, NotAllowedAccountId, RootController,
    RuntimeCall, RuntimeOrigin,
};

#[test]
fn test_setup_works() {
    new_test_ext().execute_with(|| {
        assert!(Logger::i32_log().is_empty());
        assert!(Logger::account_log().is_empty());
    });
}

#[test]
fn test_dispatch_as_root_signed_from_allowed_origin() {
    new_test_ext().execute_with(|| {
        let call = Box::new(RuntimeCall::Logger(LoggerCall::privileged_i32_log {
            i: 42,
            weight: Weight::from_ref_time(1_000),
        }));
        assert_ok!(RootController::dispatch_as_root(
            RuntimeOrigin::signed(AllowedAccountId::get()),
            call
        ));
        assert_eq!(Logger::i32_log(), vec![42i32]);
    });
}

#[test]
fn test_dispatch_as_root_signed_from_not_allowec() {
    new_test_ext().execute_with(|| {
        let call = Box::new(RuntimeCall::Logger(LoggerCall::privileged_i32_log {
            i: 42,
            weight: Weight::from_ref_time(1_000),
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
