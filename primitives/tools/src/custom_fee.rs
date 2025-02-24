use ethereum::TransactionV2;
use fp_ethereum::TransactionData;
use sp_core::U256;

pub struct CustomFeeInfo {
	pub max_conversion_rate: Option<(U256, U256)>,
	pub max_fee_per_gas: U256,
	pub max_priority_fee_per_gas: Option<U256>,
}

impl CustomFeeInfo {
	pub fn new(base_fee: U256, transaction: &TransactionV2) -> Self {
		let data: TransactionData = transaction.into();
		custom_info_from_fee_params(
			base_fee,
			data.max_fee_per_gas.or(data.gas_price),
			data.max_priority_fee_per_gas,
		)
	}
}

pub fn custom_info_from_fee_params(
	base_fee: U256,
	max_fee_per_gas: Option<U256>,
	max_priority_fee_per_gas: Option<U256>,
) -> CustomFeeInfo {
	// Calculate the reduced max_fee_per_gas
	let reduced_max_fee_per_gas = match (max_fee_per_gas, max_priority_fee_per_gas) {
		// With tip, we include as much of the tip on top of base_fee that we can, never
		// exceeding max_fee_per_gas
		(Some(max_fee_per_gas), Some(max_priority_fee_per_gas)) => {
			if max_fee_per_gas == U256::zero() {
				max_fee_per_gas // It's a ZGT transaction
			} else {
				let actual_priority_fee_per_gas = max_fee_per_gas
					.saturating_sub(base_fee)
					.min(max_priority_fee_per_gas);
				base_fee.saturating_add(actual_priority_fee_per_gas)
			}
		}
		// Without tip, we include as much of the base_fee as we can, never exceeding
		(Some(max_fee_per_gas), None) => {
			if max_fee_per_gas == U256::zero() {
				max_fee_per_gas // It's a ZGT transaction
			} else if max_fee_per_gas < base_fee {
				base_fee
			} else {
				max_fee_per_gas
			}
		}
		// If there is no max_fee_per_gas, we just use the base_fee added to the max_priority_fee_per_gas
		(None, Some(max_priority_fee_per_gas)) => max_priority_fee_per_gas.saturating_add(base_fee),
		// If there is no max_fee_per_gas and no max_priority_fee_per_gas, we just use the base_fee
		_ => base_fee,
	};

	CustomFeeInfo {
		max_conversion_rate: max_priority_fee_per_gas
			.map(|_| (reduced_max_fee_per_gas, 1_000_000_000.into())),
		max_fee_per_gas: reduced_max_fee_per_gas,
		max_priority_fee_per_gas,
	}
}

impl CustomFeeInfo {
	pub fn match_validator_conversion_rate_limit(
		&self,
		validator_conversion_rate: (U256, U256),
	) -> bool {
		if let Some(max_conversion_rate) = self.max_conversion_rate {
			max_conversion_rate.0 * validator_conversion_rate.1
				>= max_conversion_rate.1 * validator_conversion_rate.0
		} else {
			true
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use sp_core::U256;

	#[test]
	fn custom_info_from_fee_params_for_trx_v1() {
		let info = custom_info_from_fee_params(
			U256::from(1_000_000_000),
			Some(U256::from(1_000_000_000)),
			None,
		);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.max_fee_per_gas, U256::from(1_000_000_000));
		assert_eq!(info.max_conversion_rate, None);
	}

	#[test]
	fn custom_info_from_fee_params_for_trx_v2() {
		let base_fee = U256::from(1_000_000_000);
		let max_fee_x_gas = U256::from(2_000_000_000);
		let max_priority_fee_x_gas = U256::from(500_000_000);

		let info = custom_info_from_fee_params(
			base_fee,
			Some(max_fee_x_gas),
			Some(max_priority_fee_x_gas),
		);

		assert_eq!(info.max_priority_fee_per_gas, Some(max_priority_fee_x_gas));
		assert_eq!(
			info.max_fee_per_gas,
			base_fee.saturating_add(max_priority_fee_x_gas)
		);
		assert_eq!(
			info.max_conversion_rate,
			Some((
				max_fee_x_gas
					.saturating_sub(base_fee)
					.saturating_add(max_priority_fee_x_gas),
				U256::from(1_000_000_000)
			))
		);
	}

	#[test]
	fn custom_info_from_fee_params_for_read_trx() {
		let base_fee = U256::from(1_000_000_000);
		let info = custom_info_from_fee_params(base_fee, None, None);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.max_fee_per_gas, base_fee);
		assert_eq!(info.max_conversion_rate, None);
	}
}
