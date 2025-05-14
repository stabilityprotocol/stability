// Copyright © 2022 STABILITY SOLUTIONS, INC. (“STABILITY”)
// This file is part of the Stability Global Trust Network client
// software and accompanying documentation (the “Software”).

// You can download and use the Software for free under the terms of
// the Stability Open License Agreement as published by Stability on
// Github at https://github.com/stabilityprotocol/stability/LICENSE.

// THE SOFTWARE IS PROVIDED “AS IS” WITHOUT WARRANTY OF ANY KIND.
// STABILITY EXPRESSLY DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED,
// INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND
// NON-INFRINGEMENT. IN NO EVENT SHALL OWNER BE LIABLE FOR ANY
// INDIRECT, INCIDENTAL, SPECIAL OR CONSEQUENTIAL DAMAGES ARISING
// OUT OF USE OF THE SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGES.

// Please see the Stability Open License Agreement for more
// information.

use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_MILLIS, Weight};
use sp_runtime::{Perbill, Permill};

// Block time
pub const MILLISECS_PER_BLOCK: u64 = 2000;

pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// How much of time of block time is consumed (at most) in computing normal extrinsics
const COMPUTATION_BLOCK_TIME_RATIO: (u64, u64) = (2, 3); // 2 third parts of the block time

const COMPUTATION_POWER_MULTIPLIER: u64 = 6; // 6 times more computation power than normal

// how much weight for normal extrinsics could be processed in a block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_MILLIS
		* MILLISECS_PER_BLOCK
		* COMPUTATION_POWER_MULTIPLIER
		* COMPUTATION_BLOCK_TIME_RATIO.0
		/ COMPUTATION_BLOCK_TIME_RATIO.1,
	u64::MAX,
);

// `.set_proof_size`, since migration to WeightV2, we have set the proof size weight for the maximum block.
// https://github.com/paritytech/substrate/pull/12277
// https://substrate.stackexchange.com/questions/5557/construct-runtime-integrity-test-failing

pub const MAXIMUM_BLOCK_LENGTH: u32 = u32::MAX;

// Council
pub const COUNCIL_MOTION_MINUTES_DURATION: u32 = 10;
pub const COUNCIL_MAX_PROPOSALS: u32 = 100;
pub const COUNCIL_MAX_MEMBERS: u32 = 100;

pub const DEFAULT_OWNER: &str = "0xaf537bd156c7E548D0BF2CD43168dABF7aF2feb5";

pub const DEFAULT_FEE_TOKEN: &str = "0x261FB2d971eFBBFd027A9C9Cebb8548Cf7d0d2d5";

/// Minimum deposit of an account to exist.
/// Since this minimum deposit would be reduced
/// from the actual account balance we set it to zero
pub const EXISTENTIAL_DEPOSIT: u128 = 0u128;

// Session
pub const SESSION_MINUTES_DURATION: u32 = 2;

// VALIDATOR SET

pub const VALIDATOR_SET_MIN_VALIDATORS: u32 = 1;

// Gas Base Fee
pub const GAS_BASE_FEE: u128 = 1_000_000_000;
pub const DEFAULT_ELASTICITY: Permill = Permill::from_parts(0);
