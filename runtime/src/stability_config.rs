use frame_support::{
	dispatch::DispatchClass,
	weights::{
		constants::{ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_MILLIS},
		Weight,
	},
};
use sp_runtime::Perbill;

use crate::WEIGHT_PER_GAS;

// Block time
pub const MILLISECS_PER_BLOCK: u64 = 2000;

/// How much of time of block time is consumed (at most) in computing normal extrinsics
const COMPUTATION_BLOCK_TIME_RATIO: (u64, u64) = (2, 3); // 2 third parts of the block time

// how much weight for normal extrinsics could be processed in a block
pub const MAXIMUM_NORMAL_BLOCK_WEIGHT: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_MILLIS)
	.mul(MILLISECS_PER_BLOCK)
	.mul(COMPUTATION_BLOCK_TIME_RATIO.0)
	.div(COMPUTATION_BLOCK_TIME_RATIO.1) // 1_333_333_333_333
	.set_proof_size(u64::MAX);

pub const OPERATION_RESERVE_FACTOR: Perbill = Perbill::from_percent(20);

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

pub const TARGET_BLOCK_GAS_LIMIT: u64 = 50_000_000u64;

// Since BlockWeights::builder is not a const function we have to embed into a function
// It uses TARGET_BLOCK_GAS_LIMIT to set the block_weights limitations
// It checks using MAXIMUM_NORMAL_BLOCK_WEIGHT that the target is
// achieveable.
pub fn build_block_weights() -> frame_system::limits::BlockWeights {
	let normal_max_extrinsic =
		Weight::from_ref_time(TARGET_BLOCK_GAS_LIMIT * WEIGHT_PER_GAS).set_proof_size(u64::MAX);
	let normal_max_weight = normal_max_extrinsic
		.add(2 * ExtrinsicBaseWeight::get().ref_time())
		.mul(10)
		.div(9)
		.set_proof_size(u64::MAX);

	let weights = frame_system::limits::BlockWeights::builder()
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_extrinsic = Some(normal_max_extrinsic);
			weights.max_total = Some(normal_max_weight);
		})
		.for_class(DispatchClass::Operational, |weights| {
			let reserved = OPERATION_RESERVE_FACTOR * normal_max_extrinsic;
			weights.max_total = Some(normal_max_weight + reserved);
			weights.reserved = Some(reserved);
			weights.max_extrinsic = weights
				.max_total
				.map(|total| total - total.div(10) - ExtrinsicBaseWeight::get())
				.into();
		})
		.build()
		.expect("Sensible defaults are tested to be valid; qed");

	assert!(
		weights.max_block.ref_time() <= MAXIMUM_NORMAL_BLOCK_WEIGHT.ref_time(),
		"max_block weight is not computable under the given circustances"
	);

	weights
}
