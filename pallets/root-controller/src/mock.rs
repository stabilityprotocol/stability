#![cfg(test)]

use super::*;
use crate as root_controller;

use frame_support::ord_parameter_types;
use frame_support::traits::{ConstU32, ConstU64, Contains};
use sp_core::H256;
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

pub struct BlockEverything;
impl Contains<RuntimeCall> for BlockEverything {
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

impl frame_system::Config for Test {
	type BaseCallFilter = BlockEverything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type RuntimeTask = ();
	type Nonce = u64;
	type Block = Block;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

ord_parameter_types! {
	pub const AllowedAccountId: u64 = 1;
	pub const NotAllowedAccountId: u64 = 2;
}

impl Config for Test {
	type ControlOrigin = frame_system::EnsureSignedBy<AllowedAccountId, u64>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

// Logger module to track execution.
#[frame_support::pallet]
pub mod logger {

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn privileged_i32_log(origin: OriginFor<T>, i: i32) -> DispatchResultWithPostInfo {
			// Ensure that the `origin` is `Root`.
			ensure_root(origin)?;
			<I32Log<T>>::try_append(i).map_err(|_| "could not append")?;
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn force_fail_log(_: OriginFor<T>) -> DispatchResultWithPostInfo {
			Err(DispatchError::BadOrigin.into())
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn i32_log)]
	pub(super) type I32Log<T> = StorageValue<_, BoundedVec<i32, ConstU32<1_000>>, ValueQuery>;
}

impl logger::Config for Test {}

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		RootController: root_controller,
		Logger: logger,
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	t.into()
}

pub type LoggerCall = logger::Call<Test>;
