use fp_ethereum::{Transaction, TransactionData};
use sp_core::U256;

pub struct CustomFeeInfo {
	pub max_conversion_rate: Option<(U256, U256)>,
	pub max_fee_per_gas: U256,
	pub max_priority_fee_per_gas: Option<U256>,
}

impl CustomFeeInfo {
	pub fn new(base_fee: U256, transaction: &Transaction) -> Self {
		let data: TransactionData = TransactionData::from(transaction);
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
	CustomFeeInfo {
		max_conversion_rate: max_priority_fee_per_gas
			.map(|_| (max_fee_per_gas.unwrap(), 1_000_000_000.into())),
		max_fee_per_gas: max_fee_per_gas
			.or(max_priority_fee_per_gas.map(|e| e.saturating_add(base_fee)))
			.unwrap_or_default(),
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
		let info = custom_info_from_fee_params(
			U256::from(1_000_000_000),
			Some(U256::from(2_000_000_000)),
			Some(U256::from(500_000_000)),
		);

		assert_eq!(info.max_priority_fee_per_gas, Some(U256::from(500_000_000)));
		assert_eq!(info.max_fee_per_gas, U256::from(2_000_000_000));
		assert_eq!(
			info.max_conversion_rate,
			Some((U256::from(2_000_000_000), U256::from(1_000_000_000)))
		);
	}

	#[test]
	fn custom_info_from_fee_params_for_read_trx() {
		let info = custom_info_from_fee_params(U256::from(1_000_000_000), None, None);

		assert_eq!(info.max_priority_fee_per_gas, None);
		assert_eq!(info.max_fee_per_gas, U256::from(0));
		assert_eq!(info.max_conversion_rate, None);
	}
}
