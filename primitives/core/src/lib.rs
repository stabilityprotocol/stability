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

#![cfg_attr(not(feature = "std"), no_std)]

use account::EthereumSignature;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic;
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
};

pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = EthereumSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// AssetId type
pub type AssetId = u128;
/// Balance of an account.
pub type Balance = u128;
/// An index to a block.
pub type BlockNumber = u32;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Index of a transaction in the chain.
pub type Index = u32;
/// The address format for describing accounts.
pub type Address = AccountId;
/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;
/// Digest item type.
pub type DigestItem = generic::DigestItem;

pub mod aura {
	use sp_application_crypto::{app_crypto, ecdsa, KeyTypeId};

	app_crypto!(ecdsa, KeyTypeId(*b"aura"));
}
