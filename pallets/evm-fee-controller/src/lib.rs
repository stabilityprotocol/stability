#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::traits::FindAuthor;
	use pallet_evm::{EvmConfig, OnChargeEVMTransaction, Runner};
	use sp_core::{Get, H160, H256, U256};
	use sp_std::vec::Vec;
	use stbl_primitives::{eth, map_err};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		InternalError,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type FeeTokenSelectorAddress: Get<H160>;
	}

	impl<R: pallet_evm::Config, T: Config> OnChargeEVMTransaction<R> for Pallet<T> {
		type LiquidityInfo = U256;

		fn withdraw_fee(
			payer: &H160,
			fee: U256,
		) -> Result<Self::LiquidityInfo, pallet_evm::Error<R>> {
			let token = map_err!(
				Self::user_fee_token(payer),
				pallet_evm::Error::WithdrawFailed
			);

			let author = map_err!(Self::find_block_author(), pallet_evm::Error::WithdrawFailed);

			let conversion_rate = map_err!(
				Self::get_unit_conversion_rate(author, token),
				pallet_evm::Error::WithdrawFailed
			);

			frame_support::log::info!(
				"fees to be withdrawn are: {:?}",
				fee.saturating_mul(conversion_rate.0)
					.div_mod(conversion_rate.1)
					.0
			);

			// ERC20Manager::forceTransferFrom(token, payer, author, fee);

			Ok(U256::default())
		}

		fn correct_and_deposit_fee(
			who: &H160,
			corrected_fee: U256,
			base_fee: U256,
			already_withdrawn: Self::LiquidityInfo,
		) -> Self::LiquidityInfo {
			U256::default()
		}

		fn pay_priority_fee(liquidity: Self::LiquidityInfo) {}
	}

	impl<T: Config> Pallet<T> {
		fn user_fee_token(address: &H160) -> Result<H160, Error<T>> {
			let args: Vec<H256> = sp_std::vec![Into::<H256>::into(*address)];

			let calldata: Vec<u8> = eth::generate_calldata("getFeeToken(address)", &args);

			let result = T::Runner::call(
				H160::from_low_u64_be(0),
				T::FeeTokenSelectorAddress::get(),
				calldata,
				U256::from(0),
				u64::MAX,
				None,
				None,
				Some(U256::from(0)),
				Default::default(),
				false,
				false,
				&EvmConfig::istanbul(),
			);

			let output_bytes = map_err!(result, Error::<T>::InternalError).value;

			Ok(H160::from_slice(output_bytes.as_slice()))
		}

		fn find_block_author() -> Result<H160, ()> {
			let digest = <frame_system::Pallet<T>>::digest();

			match T::FindAuthor::find_author(digest.logs.iter().filter_map(|d| d.as_pre_runtime()))
			{
				None => Err(()),
				Some(author) => Ok(author),
			}
		}

		fn get_unit_conversion_rate(validator: H160, token: H160) -> Result<(U256, U256), ()> {
			let args: Vec<H256> =
				sp_std::vec![Into::<H256>::into(validator), Into::<H256>::into(token)];

			let calldata: Vec<u8> =
				eth::generate_calldata("safeTokenConversionRate(address,address)", &args);

			let output_bytes = map_err!(
				T::Runner::call(
					H160::from_low_u64_be(0),
					T::FeeTokenSelectorAddress::get(),
					calldata,
					U256::from(0),
					u64::MAX,
					None,
					None,
					Some(U256::from(0)),
					Default::default(),
					false,
					false,
					&EvmConfig::istanbul(),
				),
				()
			)
			.value;

			let splitted_output = &mut sp_std::vec![];
			eth::read_bytes32_from_output_into(
				output_bytes.clone().as_slice(),
				2,
				splitted_output,
			)?;

			let numerator = U256::from(splitted_output[0]);
			let denominator = U256::from(splitted_output[1]);

			Ok((numerator, denominator))
		}
	}
}
