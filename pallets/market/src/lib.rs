// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::fungible::Inspect;
use frame_system::pallet_prelude::BlockNumberFor;
use nonfungible_primitives::LockableNonFungible;
pub use pallet::*;
use pallet_broker::{RegionId, Timeslice};
use sp_runtime::{traits::BlockNumberProvider, SaturatedConversion};

mod types;
use crate::types::*;

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect, Mutate},
			tokens::Balance,
		},
	};
	use frame_system::{pallet_prelude::*, WeightInfo};

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The balance type
		type Balance: Balance
			+ Into<<Self::Currency as Inspect<Self::AccountId>>::Balance>
			+ From<u32>;

		/// Currency used for purchasing coretime.
		type Currency: Mutate<Self::AccountId>;

		/// Type providing a way to lock coretime regions that are listed on sale.
		type Regions: LockableNonFungible<Self::AccountId, ItemId = RegionId>;

		/// A means of getting the current relay chain block.
		///
		/// This is used for determining the current timeslice.
		type RelayChainBlockNumber: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

		/// Number of Relay-chain blocks per timeslice.
		#[pallet::constant]
		type TimeslicePeriod: Get<BlockNumberFor<Self>>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Regions that got listed on sale.
	#[pallet::storage]
	#[pallet::getter(fn regions)]
	pub type Listings<T: Config> =
		StorageMap<_, Blake2_128Concat, RegionId, Listing<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A region got listed on sale.
		Listed {
			/// The region that got listed on sale.
			region_id: RegionId,
			/// The price per timeslice of the listed region.
			timeslice_price: BalanceOf<T>,
			/// The seller of the region.
			seller: T::AccountId,
			/// The sale revenue recipient.
			sale_recipient: T::AccountId,
		},
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Caller tried to list a region which is already listed.
		AlreadyListed,
		/// Caller tried to unlist a region which is not listed.
		NotListed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// A function for listing a region on sale.
		///
		/// ## Arguments:
		/// - `region_id`: The region that the caller intends to list for sale.
		/// - `timeslice_price`: The price per a single timeslice.
		/// - `sale_recipient`: The `AccountId` receiving the payment from the sale. If not
		///   specified this will be the caller.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)] // TODO
		pub fn list_region(
			origin: OriginFor<T>,
			region_id: RegionId,
			timeslice_price: BalanceOf<T>,
			sale_recipient: Option<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Listings::<T>::get(region_id).is_none(), Error::<T>::AlreadyListed);
			T::Regions::lock(&region_id, Some(who.clone()))?;

			let sale_recipient = sale_recipient.unwrap_or(who.clone());
			Listings::<T>::insert(
				region_id,
				Listing {
					seller: who.clone(),
					timeslice_price,
					sale_recipient: sale_recipient.clone(),
				},
			);

			Self::deposit_event(Event::Listed {
				region_id,
				timeslice_price,
				seller: who,
				sale_recipient,
			});

			Ok(())
		}

		/// A function for unlisting a region on sale.
		///
		/// ## Arguments:
		/// - `region_id`: The region that the caller intends to unlist from sale.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)] // TODO
		pub fn unlist_region(origin: OriginFor<T>, region_id: RegionId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Listings::<T>::get(region_id).is_some(), Error::<T>::NotListed);

			// If the region expired anyone can remove it from the market.
			let current_timeslice = Self::current_timeslice();

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn current_timeslice() -> Timeslice {
			let latest_rc_block = T::RelayChainBlockNumber::current_block_number();
			let timeslice_period = T::TimeslicePeriod::get();
			(latest_rc_block / timeslice_period).saturated_into()
		}
	}
}
