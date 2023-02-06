// Copyright 2023 Stability Solutions.
// This file is part of Stability.

// Stability is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Stability is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stability.  If not, see <http://www.gnu.org/licenses/>.

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Currency;
use crate::stability_config::FEES_POT;
use crate::AccountId;
use crate::Balances;
use core::str::FromStr;
use frame_support::traits::OnUnbalanced;
use pallet_evm::{AddressMapping, HashedAddressMapping};
use sp_core::H160;
use sp_runtime::traits::BlakeTwo256;

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		let pot_account: AccountId =
			<HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId>>::into_account_id(
				H160::from_str(FEES_POT).expect("invalid address"),
			);

		if let Some(fees) = fees_then_tips.next() {
			Balances::resolve_creating(&pot_account, fees);
			if let Some(tips) = fees_then_tips.next() {
				Balances::resolve_creating(&pot_account, tips);
			}
		}
	}
}
