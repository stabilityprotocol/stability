#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use frame_support::weights::Weight;
use sp_core::{Get, U256};

pub struct FixedBaseFee<G: Get<U256>, W: Get<Weight>>(PhantomData<(G, W)>);

impl<G: Get<U256>, W: Get<Weight>> fp_evm::FeeCalculator for FixedBaseFee<G, W> {
	fn min_gas_price() -> (U256, Weight) {
		(G::get(), W::get())
	}
}
