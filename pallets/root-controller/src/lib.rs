#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_std::prelude::*;

use frame_support::dispatch::GetDispatchInfo;
use frame_support::dispatch::Pays;
use frame_support::traits::EnsureOrigin;
use frame_support::traits::UnfilteredDispatchable;

#[frame_support::pallet]

pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type ControlOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type RuntimeCall: Parameter
            + UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn dispatch_as_root(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResultWithPostInfo {
            T::ControlOrigin::ensure_origin(origin)?;
            let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
            // Root user does not pay a fee.
            Ok(Pays::No.into())
        }
    }
}
