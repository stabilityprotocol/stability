// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	mock::{DelegatedTransaction, ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime},
	DelegatedTransactionPrecompile,
};
use evm::ExitReason;
use fp_evm::{ExitRevert, ExitSucceed};
use libsecp256k1::{sign, Message, SecretKey};
use precompile_utils::{costs::call_cost, encoded_revert, prelude::*, testing::*};
use sp_core::{H160, H256, U256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn dispatch_cost() -> u64 {
	DelegatedTransactionPrecompile::<Runtime>::dispatch_inherent_cost()
}

#[test]
fn selectors() {
	assert!(PCall::dispatch_selectors().contains(&0xb5ea0966));
	assert!(PCall::nonces_selectors().contains(&0x7ecebe00));
	assert!(PCall::domain_separator_selectors().contains(&0x3644e515));
}

#[test]
fn modifiers() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let mut tester = PrecompilesModifierTester::new(precompiles(), CryptoAlith, DelegatedTransaction);

			tester.test_default_modifier(PCall::dispatch_selectors());
			tester.test_view_modifier(PCall::nonces_selectors());
			tester.test_view_modifier(PCall::domain_separator_selectors());
		});
}

#[test]
fn valid_delegated_transaction_returns() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();
			let permit = DelegatedTransactionPrecompile::<Runtime>::generate_delegated_transaction(
				DelegatedTransaction.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.into(),
						gas_limit,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas,
						is_static,
						context,
					} = subcall;

					// Called on the behalf of the permit maker.
					assert_eq!(context.caller, CryptoAlith.into());
					assert_eq!(address, Bob.into());
					assert_eq!(is_static, false);
					assert_eq!(target_gas, Some(100_000), "forward requested gas");

					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, CryptoAlith.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 42u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 42u8.into());

					assert_eq!(&input, b"Test");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: b"TEST".to_vec(),
						cost: 13,
						logs: vec![log1(Bob, H256::repeat_byte(0x11), vec![])],
					}
				})
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
				.expect_log(log1(Bob, H256::repeat_byte(0x11), vec![]))
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(b"TEST"))
						.build(),
				);
		})
}

#[test]
fn valid_delegated_transaction_reverts() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = DelegatedTransactionPrecompile::<Runtime>::generate_delegated_transaction(
				DelegatedTransaction.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.into(),
						gas_limit,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas,
						is_static,
						context,
					} = subcall;

					// Called on the behalf of the permit maker.
					assert_eq!(context.caller, CryptoAlith.into());
					assert_eq!(address, Bob.into());
					assert_eq!(is_static, false);
					assert_eq!(target_gas, Some(100_000), "forward requested gas");

					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, CryptoAlith.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 42u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 42u8.into());

					assert_eq!(&input, b"Test");

					SubcallOutput {
						reason: ExitReason::Revert(ExitRevert::Reverted),
						output: encoded_revert(b"TEST"),
						cost: 13,
						logs: vec![],
					}
				})
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
				.expect_no_logs()
				.execute_reverts(|x| x == b"TEST".to_vec());
		})
}

#[test]
fn invalid_delegated_transaction_nonce() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 1u8.into(); // WRONG NONCE
			let deadline: U256 = 1_000u32.into();

			let delegation = DelegatedTransactionPrecompile::<Runtime>::generate_delegated_transaction(
				DelegatedTransaction.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&delegation);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.into(),
						gas_limit,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(dispatch_cost())
				.execute_reverts(|x| x == b"Invalid delegated transaction");
		})
}

#[test]
fn invalid_delegated_transaction_gas_limit_too_low() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = DelegatedTransactionPrecompile::<Runtime>::generate_delegated_transaction(
				DelegatedTransaction.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.into(),
						gas_limit,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(call_cost + 99_999 + dispatch_cost()))
				.expect_cost(dispatch_cost())
				.execute_reverts(|x| x == b"Gaslimit is too low to dispatch provided call");
		})
}

#[test]
fn invalid_delegated_transaction_gas_limit_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = u64::MAX;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let delegated_transaction = DelegatedTransactionPrecompile::<Runtime>::generate_delegated_transaction(
				DelegatedTransaction.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(delegated_transaction));

			let secret_key = SecretKey::parse(&alith_secret_key()).unwrap();
			let message = Message::parse(&delegated_transaction);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.into(),
						gas_limit,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(100_000 + dispatch_cost()))
				.expect_cost(dispatch_cost())
				.execute_reverts(|x| x == b"Call requires too much gas (uint64 overflow)");
		})
}

#[test]
fn valid_delegated_transaction_returns_with_metamask_signed_data() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 2000)])
		.build()
		.execute_with(|| {
			let from: H160 = CryptoAlith.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = hex_literal::hex!("abcdefed").to_vec();
			let gas_limit = 100_000u64;
			let deadline: U256 = 1_000u32.into();

			// Made with MetaMask
			let rsv = hex_literal::hex!(
				"56b497d556cb1b57a16aac6e8d53f3cbf1108df467ffcb937a3744369a27478f608de05
				34b8e0385e55ffd97cbafcfeac12ab52d0b74a2dea582bc8de46f257d1c"
			)
			.as_slice();
			let (r, sv) = rsv.split_at(32);
			let (s, v) = sv.split_at(32);
			let v_real = v[0];
			let r_real: [u8; 32] = r.try_into().unwrap();
			let s_real: [u8; 32] = s.try_into().unwrap();

			precompiles()
				.prepare_test(
					CryptoAlith,
					DelegatedTransaction,
					PCall::nonces {
						owner: Address(CryptoAlith.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					DelegatedTransaction,
					PCall::dispatch {
						from: Address(from),
						to: Address(to),
						value,
						data: data.clone().into(),
						gas_limit,
						deadline,
						v: v_real,
						r: r_real.into(),
						s: s_real.into(),
					},
				)
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas,
						is_static,
						context,
					} = subcall;

					// Called on the behalf of the permit maker.
					assert_eq!(context.caller, CryptoAlith.into());
					assert_eq!(address, Bob.into());
					assert_eq!(is_static, false);
					assert_eq!(target_gas, Some(100_000), "forward requested gas");

					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, CryptoAlith.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 42u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 42u8.into());

					assert_eq!(&input, &data);

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: b"TEST".to_vec(),
						cost: 13,
						logs: vec![log1(Bob, H256::repeat_byte(0x11), vec![])],
					}
				})
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
				.expect_log(log1(Bob, H256::repeat_byte(0x11), vec![]))
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(b"TEST"))
						.build(),
				);
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["DelegatedTransaction.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}