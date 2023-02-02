use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_MILLIS, Weight};
use sp_runtime::Perbill;

// Block time
pub const MILLISECS_PER_BLOCK: u64 = 2000;

// Block weight limitation
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// How much of time of block time is consumed (at most) in computing transactions
const COMPUTATION_BLOCK_TIME_RATIO: (u64, u64) = (2, 3); // 2 third parts of the block time

// how much weight is processed in a block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_MILLIS)
    .mul(MILLISECS_PER_BLOCK)
    .mul(COMPUTATION_BLOCK_TIME_RATIO.0)
    .div(COMPUTATION_BLOCK_TIME_RATIO.1) // 1_333_333_333_333
    .set_proof_size(u64::MAX);
// `.set_proof_size`, since migration to WeightV2, we have set the proof size weight for the maximum block.
// https://github.com/paritytech/substrate/pull/12277
// https://substrate.stackexchange.com/questions/5557/construct-runtime-integrity-test-failing

pub const MAXIMUM_BLOCK_LENGTH: u32 = u32::MAX;

// Council
pub const COUNCIL_MOTION_MINUTES_DURATION: u32 = 10;
pub const COUNCIL_MAX_PROPOSALS: u32 = 100;
pub const COUNCIL_MAX_MEMBERS: u32 = 100;

pub const DEFAULT_OWNER: &str = "0xa58482131a8d67725e996af72D91A849AcC0F4A1";

/// Minimum deposit of an account to exist.
/// Since this minimum deposit would be reduced
/// from the actual account balance we set it to zero
pub const EXISTENTIAL_DEPOSIT: u128 = 0u128;
