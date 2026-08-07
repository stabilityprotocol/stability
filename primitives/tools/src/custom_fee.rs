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

use ethereum::TransactionV2;
use fp_ethereum::TransactionData;
use sp_core::U256;

pub struct CustomFeeInfo {
	pub actual_fee: U256, // The actual fee that will be charged to the user.
	pub max_priority_fee_per_gas: Option<U256>,
	pub user_conversion_rate_cap: (U256, U256),
}

impl CustomFeeInfo {
	pub fn new(base_fee: U256, transaction: &TransactionV2) -> Self {
		let data: TransactionData = transaction.into();
		compute_fee_details(
			base_fee,
			data.max_fee_per_gas.or(data.gas_price),
			data.max_priority_fee_per_gas,
		)
	}
}

// Fee calculation logic:
// 1. If max_fee_per_gas is provided and it is greater than base_fee, then the fee will be max_fee_per_gas.
// 2. If max_fee_per_gas is provided and it is less than base_fee, then the fee will be base_fee.
// 3. If max_fee_per_gas is not provided, then the fee will be base_fee.
// 4. If max_fee_per_gas is provided and it is zero, then the fee will be zero = ZERO GAS TRANSACTION
// 5. max_priority_fee_per_gas is ALWAYS IGNORED for the fee calculation.
pub fn compute_fee_details(
	base_fee: U256,
	max_fee_per_gas: Option<U256>,
	max_priority_fee_per_gas: Option<U256>,
) -> CustomFeeInfo {
	let reduced_fee = match (max_fee_per_gas, max_priority_fee_per_gas) {
		(Some(max_fee_per_gas), _) if max_fee_per_gas == U256::zero() => U256::zero(), // ZGT transaction
		(Some(max_fee_per_gas), _) if max_fee_per_gas > base_fee => max_fee_per_gas, // Use max fee if it's higher than base fee
		(Some(_), _) => base_fee, // Otherwise use base fee
		_ => base_fee,            // Default to base fee if no max fee provided
	};

	CustomFeeInfo {
		actual_fee: reduced_fee,
		user_conversion_rate_cap: (reduced_fee, 1_000_000_000.into()), // actual_fee / 1Gwei
		max_priority_fee_per_gas,
	}
}

impl CustomFeeInfo {
	// Check if the user's conversion rate is greater than the validator's conversion rate.
	// i.e: if the user's conversion rate is 1.5 and the validator's conversion rate is 1.0, the user's conversion rate is greater
	// and then it will be valid to proceed with the transaction.
	// if the user's conversion rate is 1.0 and the validator's conversion rate is 1.5, the user's conversion rate is less
	// and then it will be invalid to proceed with the transaction.
	pub fn match_validator_conversion_rate_limit(
		&self,
		validator_conversion_rate: (U256, U256),
	) -> bool {
		// Ensure no division by zero
		let user_denom = if self.user_conversion_rate_cap.1.is_zero() {
			U256::one()
		} else {
			self.user_conversion_rate_cap.1
		};

		let validator_denom = if validator_conversion_rate.1.is_zero() {
			U256::one()
		} else {
			validator_conversion_rate.1
		};

		// Cross-multiply to compare without division
		// user_rate >= validator_rate is equivalent to:
		// user_num * validator_denom >= validator_num * user_denom
		// full_mul widens to U512, so the comparison is exact and cannot overflow.
		self.user_conversion_rate_cap.0.full_mul(validator_denom)
			>= validator_conversion_rate.0.full_mul(user_denom)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sp_core::U256;

	#[test]
	fn compute_fee_details_for_trx_v1() {
		let info = compute_fee_details(
			U256::from(1_000_000_000),
			Some(U256::from(1_000_000_000)),
			None,
		);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.actual_fee, U256::from(1_000_000_000));
		assert_eq!(
			info.user_conversion_rate_cap,
			(U256::from(1_000_000_000), U256::from(1_000_000_000))
		);
	}

	#[test]
	fn compute_fee_details_for_trx_v2() {
		let base_fee = U256::from(1_000_000_000);
		let max_fee_x_gas = U256::from(1_500_000_000);
		let max_priority_fee_x_gas = U256::from(2_000_000_000);

		let info = compute_fee_details(base_fee, Some(max_fee_x_gas), Some(max_priority_fee_x_gas));

		assert_eq!(info.max_priority_fee_per_gas, Some(max_priority_fee_x_gas));
		assert_eq!(
			info.actual_fee,
			max_fee_x_gas // The fee is capped at max_fee_x_gas
		);
		assert_eq!(
			info.user_conversion_rate_cap,
			(info.actual_fee, U256::from(1_000_000_000))
		);
	}

	#[test]
	fn compute_fee_details_for_trx_v3() {
		let base_fee = U256::from(1_000_000_000);
		let max_fee_x_gas = U256::from(1_500_000_000);
		let max_priority_fee_x_gas = U256::from(500_000_000);

		let info = compute_fee_details(base_fee, Some(max_fee_x_gas), Some(max_priority_fee_x_gas));

		assert_eq!(info.max_priority_fee_per_gas, Some(max_priority_fee_x_gas));
		assert_eq!(
			info.actual_fee,
			base_fee.saturating_add(max_priority_fee_x_gas)
		);
		assert_eq!(
			info.user_conversion_rate_cap,
			(info.actual_fee, U256::from(1_000_000_000))
		);
	}

	#[test]
	fn compute_fee_details_for_read_trx() {
		let base_fee = U256::from(1_000_000_000);
		let info = compute_fee_details(base_fee, None, None);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.actual_fee, base_fee);
		assert_eq!(
			info.user_conversion_rate_cap,
			(base_fee, U256::from(1_000_000_000))
		);
	}

	#[test]
	fn compute_fee_details_for_zgt_trx_0() {
		let base_fee = U256::from(1_000_000_000);
		let info = compute_fee_details(base_fee, Some(U256::from(0)), None);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.actual_fee, U256::from(0));
		assert_eq!(
			info.user_conversion_rate_cap,
			(U256::from(0), U256::from(1_000_000_000))
		);
	}

	#[test]
	fn compute_fee_details_for_zgt_trx_1() {
		let base_fee = U256::from(1_000_000_000);
		let info = compute_fee_details(base_fee, Some(U256::from(0)), Some(U256::from(0)));

		assert_eq!(info.actual_fee, U256::from(0));
		assert_eq!(
			info.user_conversion_rate_cap,
			(U256::from(0), U256::from(1_000_000_000))
		);
	}

	fn info_with_cap(numerator: U256, denominator: U256) -> CustomFeeInfo {
		CustomFeeInfo {
			actual_fee: numerator,
			max_priority_fee_per_gas: None,
			user_conversion_rate_cap: (numerator, denominator),
		}
	}

	#[test]
	fn conversion_rate_limit_equal_rates_match() {
		let info = info_with_cap(U256::from(3), U256::from(2));
		assert!(info.match_validator_conversion_rate_limit((U256::from(3), U256::from(2))));
		assert!(info.match_validator_conversion_rate_limit((U256::from(6), U256::from(4))));
	}

	#[test]
	fn conversion_rate_limit_user_rate_higher_matches() {
		let info = info_with_cap(U256::from(3), U256::from(1));
		assert!(info.match_validator_conversion_rate_limit((U256::from(2), U256::from(1))));
	}

	#[test]
	fn conversion_rate_limit_user_rate_lower_rejects() {
		let info = info_with_cap(U256::from(1), U256::from(2));
		assert!(!info.match_validator_conversion_rate_limit((U256::from(2), U256::from(1))));
	}

	#[test]
	fn conversion_rate_limit_zero_denominators_do_not_panic() {
		// Zero denominators are normalized to 1 on both sides.
		let info = info_with_cap(U256::from(5), U256::zero());
		assert!(info.match_validator_conversion_rate_limit((U256::from(5), U256::zero())));
		assert!(info.match_validator_conversion_rate_limit((U256::from(4), U256::zero())));
		assert!(!info.match_validator_conversion_rate_limit((U256::from(6), U256::zero())));
	}

	#[test]
	fn conversion_rate_limit_user_side_overflow_is_exact() {
		// user = MAX/1, validator = MAX/1 with big denominators: cross products
		// exceed 256 bits but full_mul keeps the comparison exact.
		let info = info_with_cap(U256::max_value(), U256::from(2));
		assert!(info.match_validator_conversion_rate_limit((U256::max_value(), U256::from(3))));
		// user rate (MAX/3) < validator rate (MAX/2) must reject even though
		// both cross products overflow 256 bits.
		let info = info_with_cap(U256::max_value(), U256::from(3));
		assert!(!info.match_validator_conversion_rate_limit((U256::max_value(), U256::from(2))));
	}

	#[test]
	fn conversion_rate_limit_validator_side_overflow_is_exact() {
		// validator product overflows 256 bits, user product does not:
		// user rate is clearly lower, must reject.
		let info = info_with_cap(U256::from(1), U256::from(1));
		assert!(!info.match_validator_conversion_rate_limit((U256::max_value(), U256::from(2))));
		// and the symmetric case: huge user rate against tiny validator rate.
		let info = info_with_cap(U256::max_value(), U256::from(1));
		assert!(info.match_validator_conversion_rate_limit((U256::from(1), U256::from(2))));
	}
}
