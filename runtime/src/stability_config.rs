use core::str::FromStr;

use frame_support::weights::{constants::WEIGHT_PER_MILLIS, Weight};
use sp_core::{parameter_types, H160};
use sp_runtime::Perbill;

// Block time
pub const MILLISECS_PER_BLOCK: u64 = 2000;

// Block weight limitation
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// How much of time of block time is consumed (at most) in computing transactions
const COMPUTATION_BLOCK_TIME_RATIO: (u64, u64) = (2, 3); // 2 third parts of the block time

// how much weight is processed in a block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_MILLIS
    .mul(MILLISECS_PER_BLOCK)
    .mul(COMPUTATION_BLOCK_TIME_RATIO.0)
    .div(COMPUTATION_BLOCK_TIME_RATIO.1); // 1_333_333_333_333

pub const MAXIMUM_BLOCK_LENGTH: u32 = u32::MAX;

parameter_types! {
    pub DefaultOwner : H160 = H160::from_str("0x0e92e75Dcdc783c86749D4cd9F19e1D634C3a7dd").expect("invalid address");
}
