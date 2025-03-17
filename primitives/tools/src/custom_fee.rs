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

// Fee calculation logic (following EIP-1559):
// 1. For ZGT transactions (max_fee_per_gas = 0), fee is zero regardless of base_fee.
// 2. For EIP-1559 transactions with both max_fee_per_gas and max_priority_fee_per_gas:
//    - Calculate effective priority fee: min(max_priority_fee_per_gas, max_fee_per_gas - base_fee)
//    - Total fee = base_fee + effective_priority_fee (never exceeds max_fee_per_gas)
// 3. For legacy transactions with only max_fee_per_gas (gas_price):
//    - If max_fee_per_gas = 0, fee is zero (ZGT transaction)
//    - Otherwise, fee equals base_fee (no priority fee component)
// 4. For read-only transactions (no specified fees), fee equals base_fee.
pub fn compute_fee_details(
	base_fee: U256,
	max_fee_per_gas: Option<U256>,
	max_priority_fee_per_gas: Option<U256>,
) -> CustomFeeInfo {
	let reduced_fee = match (max_fee_per_gas, max_priority_fee_per_gas) {
		(Some(max_fee_per_gas), Some(max_priority_fee_per_gas)) => {
			if max_fee_per_gas == U256::zero() {
				max_fee_per_gas // ZGT transaction
			} else {
				// With tip, we include as much of the tip on top of base_fee that we can, never
				// exceeding max_fee_per_gas
				let actual_priority_fee_per_gas = max_fee_per_gas
					.saturating_sub(base_fee)
					.min(max_priority_fee_per_gas);

				base_fee.saturating_add(actual_priority_fee_per_gas)
			}
		}
		(Some(max_fee_per_gas), None) => {
			if max_fee_per_gas == U256::zero() {
				max_fee_per_gas // ZGT transaction
			} else {
				base_fee
			}
		}
		_ => base_fee,
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
		let adjusted_user_conversion_rate = self
			.user_conversion_rate_cap
			.0
			.div_mod(if self.user_conversion_rate_cap.1.is_zero() {
				U256::one()
			} else {
				self.user_conversion_rate_cap.1
			})
			.0; // only keep the integer part
		let adjusted_validator_conversion_rate = validator_conversion_rate
			.0
			.div_mod(if validator_conversion_rate.1.is_zero() {
				U256::one()
			} else {
				validator_conversion_rate.1
			})
			.0; // only keep the integer part

		adjusted_user_conversion_rate >= adjusted_validator_conversion_rate
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
}
