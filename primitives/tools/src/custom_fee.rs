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

// The fee is calculated as follows (EIP-1559):
// 1. If the transaction is a ZGT transaction (max_fee_per_gas is zero), the fee is zero.
// 2. If the transaction is a transaction with a max_fee_per_gas and max_priority_fee_per_gas:
//    - The effective priority fee is min(max_priority_fee_per_gas, max_fee_per_gas - base_fee)
//    - The total fee is min(max_fee_per_gas, base_fee + effective_priority_fee)
// 3. If the transaction is a transaction with only max_fee_per_gas:
//    - The fee is min(max_fee_per_gas, base_fee)
// 4. If nothing is specified, the fee is the base_fee.
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
				// Calculate effective priority fee (cannot exceed max_fee_per_gas - base_fee)
				let available_for_priority = if max_fee_per_gas > base_fee {
					max_fee_per_gas.saturating_sub(base_fee)
				} else {
					U256::zero()
				};

				let effective_priority_fee = max_priority_fee_per_gas.min(available_for_priority);

				// Total fee is base_fee + priority fee, capped at max_fee_per_gas
				base_fee
					.saturating_add(effective_priority_fee)
					.min(max_fee_per_gas)
			}
		}
		(Some(max_fee_per_gas), None) => {
			if max_fee_per_gas == U256::zero() {
				max_fee_per_gas // ZGT transaction
			} else {
				max_fee_per_gas.min(base_fee)
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
