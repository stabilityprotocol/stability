#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A dispatch_as_root just took place. \[result\]
        DispatchAsRootOccurred { dispatch_result: DispatchResult },
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
            Self::deposit_event(Event::DispatchAsRootOccurred {
                dispatch_result: res.map(|_| ()).map_err(|e| e.error),
            });
            Ok(Pays::No.into())
        }
    }
}
