#![cfg(test)]

use super::*;
use crate as root_controller;

use frame_support::ord_parameter_types;
use frame_support::traits::{ConstU32, ConstU64, Contains};
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type Block = frame_system::mocking::MockBlock<Test>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

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
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
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
}

ord_parameter_types! {
    pub const AllowedAccountId: u64 = 1;
    pub const NotAllowedAccountId: u64 = 2;
}

impl Config for Test {
    type ControlOrigin = frame_system::EnsureSignedBy<AllowedAccountId, u64>;
    type RuntimeCall = RuntimeCall;
}

// Logger module to track execution.
#[frame_support::pallet]
pub mod logger {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(*weight)]
        pub fn privileged_i32_log(
            origin: OriginFor<T>,
            i: i32,
            weight: Weight,
        ) -> DispatchResultWithPostInfo {
            // Ensure that the `origin` is `Root`.
            ensure_root(origin)?;
            <I32Log<T>>::try_append(i).map_err(|_| "could not append")?;
            Self::deposit_event(Event::AppendI32 { value: i, weight });
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(*weight)]
        pub fn non_privileged_log(
            origin: OriginFor<T>,
            i: i32,
            weight: Weight,
        ) -> DispatchResultWithPostInfo {
            // Ensure that the `origin` is some signed account.
            let sender = ensure_signed(origin)?;
            <I32Log<T>>::try_append(i).map_err(|_| "could not append")?;
            <AccountLog<T>>::try_append(sender.clone()).map_err(|_| "could not append")?;
            Self::deposit_event(Event::AppendI32AndAccount {
                sender,
                value: i,
                weight,
            });
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AppendI32 {
            value: i32,
            weight: Weight,
        },
        AppendI32AndAccount {
            sender: T::AccountId,
            value: i32,
            weight: Weight,
        },
    }

    #[pallet::storage]
    #[pallet::getter(fn account_log)]
    pub(super) type AccountLog<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<1_000>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn i32_log)]
    pub(super) type I32Log<T> = StorageValue<_, BoundedVec<i32, ConstU32<1_000>>, ValueQuery>;
}

impl logger::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}

frame_support::construct_runtime!(
    pub enum Test
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic, {
            System: frame_system,
            RootController: root_controller,
            Logger: logger,
        }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    t.into()
}

pub type LoggerCall = logger::Call<Test>;
