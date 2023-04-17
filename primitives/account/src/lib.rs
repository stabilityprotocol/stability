// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! The Ethereum Signature implementation.
//!
//! It includes the Verify and IdentifyAccount traits for the AccountId20

#![cfg_attr(not(feature = "std"), no_std)]

use core::hash::Hash;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sha3::{Digest, Keccak256};
use sp_core::{ecdsa, H160};
use sp_runtime::traits::IdentifyAccount;
use sp_std::vec::Vec;

use sp_runtime::{app_crypto::app_crypto, CryptoTypeId, KeyTypeId, RuntimeAppPublic};

#[cfg(feature = "std")]
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "std")]
use sp_core::crypto::ByteArray;
#[cfg(feature = "std")]
use sp_core::crypto::{Derive, Pair as PairT};

//TODO Maybe this should be upstreamed into Frontier (And renamed accordingly) so that it can
// be used in palletEVM as well. It may also need more traits such as AsRef, AsMut, etc like
// AccountId32 has.

/// The account type to be used in Moonbeam. It is a wrapper for 20 fixed bytes. We prefer to use
/// a dedicated type to prevent using arbitrary 20 byte arrays were AccountIds are expected. With
/// the introduction of the `scale-info` crate this benefit extends even to non-Rust tools like
/// Polkadot JS.

#[derive(
	Eq,
	PartialEq,
	Copy,
	Clone,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
	Default,
	PartialOrd,
	Ord,
	Hash,
)]
pub struct AccountId20(pub [u8; 20]);

impl From<H160> for AccountId20 {
	fn from(h: H160) -> Self {
		Self(h.0)
	}
}

impl Into<H160> for AccountId20 {
	fn into(self) -> H160 {
		H160(self.0)
	}
}

impl AsMut<[u8]> for AccountId20 {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0
	}
}

#[cfg(feature = "std")]
impl ByteArray for AccountId20 {
	const LEN: usize = 20;

	fn from_slice(data: &[u8]) -> Result<Self, ()> {
		Self::try_from(data)
	}

	fn to_raw_vec(&self) -> Vec<u8> {
		self.as_slice().to_vec()
	}

	fn as_slice(&self) -> &[u8] {
		self.as_ref()
	}
}

#[cfg(feature = "std")]
impl Derive for AccountId20 {
	fn derive<Iter>(&self, _path: Iter) -> Option<Self> {
		None
	}
}

#[cfg(feature = "std")]
impl sp_core::crypto::Public for AccountId20 {
	fn to_public_crypto_pair(&self) -> sp_core::crypto::CryptoTypePublicPair {
		todo!()
	}
}

#[cfg(feature = "std")]
impl sp_core::crypto::CryptoType for AccountId20 {
	type Pair = EthereumSignaturePair;
}

#[cfg(feature = "std")]
impl_serde::impl_fixed_hash_serde!(AccountId20, 20);

#[cfg(feature = "std")]
impl std::fmt::Display for AccountId20 {
	//TODO This is a pretty quck-n-dirty implementation. Perhaps we should add
	// checksum casing here? I bet there is a crate for that.
	// Maybe this one https://github.com/miguelmota/rust-eth-checksum
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl IdentifyAccount for AccountId20 {
	type AccountId = AccountId20;
	fn into_account(self) -> Self::AccountId {
		self
	}
}

impl TryFrom<&[u8]> for AccountId20 {
	type Error = ();
	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		let mut inner = [0u8; 20];
		inner.copy_from_slice(bytes);
		Ok(Self(inner))
	}
}

impl core::fmt::Debug for AccountId20 {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:?}", H160(self.0))
	}
}

impl From<[u8; 20]> for AccountId20 {
	fn from(bytes: [u8; 20]) -> Self {
		Self(bytes)
	}
}

impl Into<[u8; 20]> for AccountId20 {
	fn into(self) -> [u8; 20] {
		self.0
	}
}

impl AsRef<[u8]> for AccountId20 {
	fn as_ref(&self) -> &[u8] {
		&self.0.as_slice()
	}
}

app_crypto!(ecdsa, KeyTypeId(*b"etha"));

#[cfg(feature = "std")]
impl RuntimeAppPublic for AccountId20 {
	/// An identifier for this application-specific key type.
	const ID: KeyTypeId = KeyTypeId(*b"etha");
	/// The identifier of the crypto type of this application-specific key type.
	const CRYPTO_ID: CryptoTypeId = CryptoTypeId(*b"etha");

	/// The signature that will be generated when signing with the corresponding private key.
	type Signature = EthereumSignature;

	/// Returns all public keys for this application in the keystore.
	fn all() -> sp_std::vec::Vec<Self> {
		// get access to the keystore and get all the public keys that matches the Keytype(*b"etha")

		Default::default()
	}

	/// Generate a public/private pair with an optional `seed` and store it in the keystore.
	///
	/// The `seed` needs to be valid utf8.
	///
	/// Returns the generated public key.
	fn generate_pair(seed: Option<Vec<u8>>) -> Self {
		let password = seed.map(|seed| {
			let seed_result_string = String::from_utf8(seed);

			seed_result_string.unwrap()
		});

		let (pair, _, _) = if let Some(pass) = password {
			EthereumSignaturePair::generate_with_phrase(Some(pass.as_str()))
		} else {
			EthereumSignaturePair::generate_with_phrase(None)
		};

		pair.public()
	}

	/// Sign the given message with the corresponding private key of this public key.
	///
	/// The private key will be requested from the keystore.
	///
	/// Returns the signature or `None` if the private key could not be found or some other error
	/// occurred.
	fn sign<M: AsRef<[u8]>>(&self, msg: &M) -> Option<Self::Signature> {
		None
	}

	/// Verify that the given signature matches the given message using this public key.
	fn verify<M: AsRef<[u8]>>(&self, msg: &M, signature: &Self::Signature) -> bool {
		false
	}

	/// Returns `Self` as raw vec.
	fn to_raw_vec(&self) -> Vec<u8> {
		Default::default()
	}
}

#[cfg(feature = "std")]
impl std::str::FromStr for AccountId20 {
	type Err = &'static str;
	fn from_str(input: &str) -> Result<Self, Self::Err> {
		H160::from_str(input)
			.map(Into::into)
			.map_err(|_| "invalid hex address.")
	}
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_core::RuntimeDebug, TypeInfo)]
pub struct EthereumSignature(ecdsa::Signature);

impl Hash for EthereumSignature {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		state.write(self.0 .0.as_slice());
		state.finish();
	}

	fn hash_slice<H: core::hash::Hasher>(data: &[Self], state: &mut H)
	where
		Self: Sized,
	{
		for piece in data {
			state.write(piece.0 .0.as_slice());
		}
		state.finish();
	}
}

impl From<ecdsa::Signature> for EthereumSignature {
	fn from(x: ecdsa::Signature) -> Self {
		EthereumSignature(x)
	}
}

impl sp_runtime::traits::Verify for EthereumSignature {
	type Signer = EthereumSigner;
	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, mut msg: L, signer: &AccountId20) -> bool {
		let mut m = [0u8; 32];
		m.copy_from_slice(Keccak256::digest(msg.get()).as_slice());
		match sp_io::crypto::secp256k1_ecdsa_recover(self.0.as_ref(), &m) {
			Ok(pubkey) => {
				AccountId20(H160::from_slice(&Keccak256::digest(&pubkey).as_slice()[12..32]).0)
					== *signer
			}
			Err(sp_io::EcdsaVerifyError::BadRS) => {
				log::error!(target: "evm", "Error recovering: Incorrect value of R or S");
				false
			}
			Err(sp_io::EcdsaVerifyError::BadV) => {
				log::error!(target: "evm", "Error recovering: Incorrect value of V");
				false
			}
			Err(sp_io::EcdsaVerifyError::BadSignature) => {
				log::error!(target: "evm", "Error recovering: Invalid signature");
				false
			}
		}
	}
}

/// Public key for an Ethereum / Moonbeam compatible account
#[derive(
	Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, sp_core::RuntimeDebug, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EthereumSigner([u8; 20]);

impl From<EthereumSigner> for AccountId20 {
	fn from(value: EthereumSigner) -> Self {
		Self(value.0)
	}
}

impl sp_runtime::traits::IdentifyAccount for EthereumSigner {
	type AccountId = AccountId20;
	fn into_account(self) -> AccountId20 {
		AccountId20(self.0)
	}
}

impl From<[u8; 20]> for EthereumSigner {
	fn from(x: [u8; 20]) -> Self {
		EthereumSigner(x)
	}
}

impl From<ecdsa::Public> for EthereumSigner {
	fn from(x: ecdsa::Public) -> Self {
		let decompressed = libsecp256k1::PublicKey::parse_slice(
			&x.0,
			Some(libsecp256k1::PublicKeyFormat::Compressed),
		)
		.expect("Wrong compressed public key provided")
		.serialize();
		let mut m = [0u8; 64];
		m.copy_from_slice(&decompressed[1..65]);
		let account = H160::from_slice(&Keccak256::digest(&m).as_slice()[12..32]);
		EthereumSigner(account.into())
	}
}

impl From<libsecp256k1::PublicKey> for EthereumSigner {
	fn from(x: libsecp256k1::PublicKey) -> Self {
		let mut m = [0u8; 64];
		m.copy_from_slice(&x.serialize()[1..65]);
		let account = H160::from_slice(&Keccak256::digest(&m).as_slice()[12..32]);
		EthereumSigner(account.into())
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for EthereumSigner {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "ethereum signature: {:?}", H160::from_slice(&self.0))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{ecdsa, Pair, H256};
	use sp_runtime::traits::IdentifyAccount;

	#[test]
	fn test_account_derivation_1() {
		// Test from https://asecuritysite.com/encryption/ethadd
		let secret_key =
			hex::decode("502f97299c472b88754accd412b7c9a6062ef3186fba0c0388365e1edec24875")
				.unwrap();
		let mut expected_hex_account = [0u8; 20];
		hex::decode_to_slice(
			"976f8456e4e2034179b284a23c0e0c8f6d3da50c",
			&mut expected_hex_account,
		)
		.expect("example data is 20 bytes of valid hex");

		let public_key = ecdsa::Pair::from_seed_slice(&secret_key).unwrap().public();
		let account: EthereumSigner = public_key.into();
		let expected_account = AccountId20::from(expected_hex_account);
		assert_eq!(account.into_account(), expected_account);
	}
	#[test]
	fn test_account_derivation_2() {
		// Test from https://asecuritysite.com/encryption/ethadd
		let secret_key =
			hex::decode("0f02ba4d7f83e59eaa32eae9c3c4d99b68ce76decade21cdab7ecce8f4aef81a")
				.unwrap();
		let mut expected_hex_account = [0u8; 20];
		hex::decode_to_slice(
			"420e9f260b40af7e49440cead3069f8e82a5230f",
			&mut expected_hex_account,
		)
		.expect("example data is 20 bytes of valid hex");

		let public_key = ecdsa::Pair::from_seed_slice(&secret_key).unwrap().public();
		let account: EthereumSigner = public_key.into();
		let expected_account = AccountId20::from(expected_hex_account);
		assert_eq!(account.into_account(), expected_account);
	}
	#[test]
	fn test_account_derivation_3() {
		let m = hex::decode("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
			.unwrap();
		let old = AccountId20(H160::from(H256::from_slice(Keccak256::digest(&m).as_slice())).0);
		let new = AccountId20(H160::from_slice(&Keccak256::digest(&m).as_slice()[12..32]).0);
		assert_eq!(new, old);
	}
}

#[cfg(feature = "std")]
#[derive(Clone)]
pub struct EthereumSignaturePair {
	public: AccountId20,
	ecdsa_pair: sp_core::ecdsa::Pair,
}

#[cfg(feature = "std")]
impl sp_core::crypto::CryptoType for EthereumSignaturePair {
	type Pair = EthereumSignaturePair;
}

#[cfg(feature = "std")]
impl AccountId20 {
	pub fn from_pubkey(public: sp_core::ecdsa::Public) -> Self {
		todo!("{:?}", public)
	}
}

#[cfg(feature = "std")]
impl PairT for EthereumSignaturePair {
	type Public = AccountId20;

	type Seed = <sp_core::ecdsa::Pair as PairT>::Seed;

	type Signature = <sp_core::ecdsa::Pair as PairT>::Signature;

	type DeriveError = <sp_core::ecdsa::Pair as PairT>::DeriveError;

	fn generate_with_phrase(password: Option<&str>) -> (Self, String, Self::Seed) {
		let (ecdsa_pair, phrase, seed) =
			<sp_core::ecdsa::Pair as PairT>::generate_with_phrase(password);

		(
			Self {
				public: AccountId20::from_pubkey(ecdsa_pair.public()),
				ecdsa_pair,
			},
			phrase,
			seed,
		)
	}

	fn from_phrase(
		phrase: &str,
		password: Option<&str>,
	) -> Result<(Self, Self::Seed), sp_core::crypto::SecretStringError> {
		let (ecdsa_pair, _) = <sp_core::ecdsa::Pair as PairT>::from_phrase(phrase, password)?;

		Ok((
			Self {
				public: AccountId20::from_pubkey(ecdsa_pair.clone().public()),
				ecdsa_pair: ecdsa_pair.clone(),
			},
			ecdsa_pair.seed(),
		))
	}

	fn derive<Iter: Iterator<Item = sp_core::crypto::DeriveJunction>>(
		&self,
		path: Iter,
		seed: Option<Self::Seed>,
	) -> Result<(Self, Option<Self::Seed>), Self::DeriveError> {
		let (pair, seed) = self.ecdsa_pair.derive(path, seed)?;

		Ok((
			Self {
				public: AccountId20::from_pubkey(pair.public()),
				ecdsa_pair: pair,
			},
			seed,
		))
	}

	fn from_seed(seed: &Self::Seed) -> Self {
		let ecdsa_pair = ecdsa::Pair::from_seed(seed);

		Self {
			public: AccountId20::from_pubkey(ecdsa_pair.public()),
			ecdsa_pair,
		}
	}

	fn from_seed_slice(seed: &[u8]) -> Result<Self, sp_core::crypto::SecretStringError> {
		let ecdsa_pair = ecdsa::Pair::from_seed_slice(seed)?;

		Ok(Self {
			public: AccountId20::from_pubkey(ecdsa_pair.public()),
			ecdsa_pair,
		})
	}

	fn sign(&self, message: &[u8]) -> Self::Signature {
		self.ecdsa_pair.sign(message)
	}

	fn verify<M: AsRef<[u8]>>(sig: &Self::Signature, message: M, address: &Self::Public) -> bool {
		sp_core::ecdsa::Signature::recover(sig, message)
			.map(|signer_pubkey| AccountId20::from_pubkey(signer_pubkey).eq(address))
			.unwrap_or(false)
	}

	fn verify_weak<P: AsRef<[u8]>, M: AsRef<[u8]>>(sig: &[u8], message: M, pubkey: P) -> bool {
		match ecdsa::Signature::from_slice(sig).and_then(|sig| sig.recover(message)) {
			Some(actual) => actual.as_ref() == pubkey.as_ref(),
			None => false,
		}
	}

	fn public(&self) -> Self::Public {
		self.public
	}

	fn to_raw_vec(&self) -> Vec<u8> {
		todo!()
	}
}
