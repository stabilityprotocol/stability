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

//! Tests for the Validator Set pallet.

use super::*;
use crate::mock::{new_test_ext, FeeRewardsVault};
use core::str::FromStr;
use frame_support::{assert_noop, parameter_types};

parameter_types! {
	pub DappAddress:H160 = H160::from_str("0x5F4bf370dA7e059FAf4eE007233f48D5131F1983").expect("invalid address");
	pub DappAddress2:H160 = H160::from_str("0xa6F2b8883F00238de607Fd9f7c67c56A81fe3402").expect("invalid address");
	pub TokenAddress:H160 = H160::from_str("0xd70ceDDA8d51D3A70e406CFB6bEbf6a664702F04").expect("invalid address");
	pub TokenAddress2: H160 = H160::from_str("0xa11Ea5dd87c42f43265D189A93db49C703732FB1").expect("invalid address");
}

#[test]
fn simple_setup_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(FeeRewardsVault::is_whitelisted(DappAddress::get()), false);
	});
}

#[test]
fn set_whitelist_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(FeeRewardsVault::is_whitelisted(DappAddress::get()), false);
		assert_eq!(FeeRewardsVault::is_whitelisted(DappAddress2::get()), false);
		FeeRewardsVault::set_whitelist(DappAddress::get(), true);
		assert_eq!(FeeRewardsVault::is_whitelisted(DappAddress::get()), true);
		assert_eq!(FeeRewardsVault::is_whitelisted(DappAddress2::get()), false);
	});
}

#[test]
fn get_claimable_reward_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(DappAddress::get(), TokenAddress::get()),
			U256::zero()
		);
	});
}

#[test]
fn add_claimable_reward_should_work() {
	new_test_ext().execute_with(|| {
		let amount = U256::from(100);
		FeeRewardsVault::add_claimable_reward(DappAddress::get(), TokenAddress::get(), amount)
			.unwrap();
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(DappAddress::get(), TokenAddress::get()),
			amount
		);
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(DappAddress::get(), TokenAddress2::get()),
			U256::zero()
		);
	});
}

#[test]
fn add_claimable_reward_should_fail_if_overflow() {
	new_test_ext().execute_with(|| {
		let amount = U256::from(100);
		FeeRewardsVault::add_claimable_reward(DappAddress::get(), TokenAddress::get(), amount)
			.unwrap();
		assert_noop!(
			FeeRewardsVault::add_claimable_reward(
				DappAddress::get(),
				TokenAddress::get(),
				U256::max_value()
			),
			"Overflow adding a new claimable reward"
		);
	});
}

#[test]
fn sub_claimable_reward_should_work() {
	new_test_ext().execute_with(|| {
		let amount = U256::from(100);
		FeeRewardsVault::add_claimable_reward(DappAddress::get(), TokenAddress::get(), amount)
			.unwrap();
		FeeRewardsVault::sub_claimable_reward(
			DappAddress::get(),
			TokenAddress::get(),
			U256::from(50),
		)
		.unwrap();
		assert_eq!(
			FeeRewardsVault::get_claimable_reward(DappAddress::get(), TokenAddress::get()),
			U256::from(50)
		);
	});
}

#[test]
fn sub_claimable_reward_with_insufficient_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = U256::from(100);
		FeeRewardsVault::add_claimable_reward(DappAddress::get(), TokenAddress::get(), amount)
			.unwrap();
		assert_noop!(
			FeeRewardsVault::sub_claimable_reward(
				DappAddress::get(),
				TokenAddress::get(),
				U256::from(200)
			),
			"Insufficient balance"
		);
	});
}
