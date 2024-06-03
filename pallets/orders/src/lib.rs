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

use frame_support::traits::Currency;
pub use pallet::*;
use pallet_broker::Timeslice;
use sp_runtime::{traits::BlockNumberProvider, SaturatedConversion};
use xcm_executor::traits::ConvertLocation;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use crate::types::*;

pub mod weights;
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Relay chain block number.
pub type RCBlockNumberOf<T> =
	<<T as crate::Config>::RCBlockNumberProvider as BlockNumberProvider>::BlockNumber;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::Saturating,
		traits::{fungible::Mutate, Get, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency used for purchasing coretime.
		type Currency: Mutate<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// How to get an `AccountId` value from a `Location`.
		type SovereignAccountOf: ConvertLocation<Self::AccountId>;

		/// Type responsible for dealing with order creation fees.
		type OrderCreationFeeHandler: FeeHandler<Self::AccountId, BalanceOf<Self>>;

		/// Type for getting the current relay chain block.
		///
		/// This is used for determining the current timeslice.
		type RCBlockNumberProvider: BlockNumberProvider;

		/// Number of Relay-chain blocks per timeslice.
		#[pallet::constant]
		type TimeslicePeriod: Get<RCBlockNumberOf<Self>>;

		/// The cost of order creation.
		///
		/// Used to prevent spamming.
		#[pallet::constant]
		type OrderCreationCost: Get<BalanceOf<Self>>;

		/// The minimum contribution to an order.
		#[pallet::constant]
		type MinimumContribution: Get<BalanceOf<Self>>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Created orders.
	#[pallet::storage]
	#[pallet::getter(fn orders)]
	pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, OrderId, Order<T::AccountId>>;

	/// Next order id.
	#[pallet::storage]
	#[pallet::getter(fn next_order_id)]
	pub type NextOrderId<T> = StorageValue<_, OrderId, ValueQuery>;

	/// Crowdfunding contributions.
	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	pub type Contributions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OrderId, //  order id.
		Blake2_128Concat,
		T::AccountId, // contributor
		BalanceOf<T>, // contributed amount
		ValueQuery,
	>;

	/// The total amount that was contributed to an order.
	///
	/// The sum of contributions for a specific order from the `Contributions` map should be equal
	/// to the total contribution stored here.
	#[pallet::storage]
	#[pallet::getter(fn total_contributions)]
	pub type TotalContributions<T: Config> =
		StorageMap<_, Blake2_128Concat, OrderId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An order was created.
		OrderCreated { order_id: OrderId, by: T::AccountId },
		/// An order was removed.
		OrderRemoved { order_id: OrderId, by: T::AccountId },
		/// A contribution was made to the order specificed by `order_id`.
		Contributed { order_id: OrderId, who: T::AccountId, amount: BalanceOf<T> },
		/// A contribution was removed from the cancelled order.
		ContributionRemoved { order_id: OrderId, who: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Invalid order id.
		InvalidOrderId,
		/// The caller is not the creator of the given order.
		NotAllowed,
		/// The contribution amount is too small.
		InvalidAmount,
		/// The order is expired.
		OrderExpired,
		/// The given order is not cancelled.
		OrderNotCancelled,
		/// The contributed amount equals to zero.
		NoContribution,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic for creating an order.
		///
		/// ## Arguments:
		/// - `para_id`: The para id to which Coretime will be allocated.
		/// - `requirements`: Region requirements of the order.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_order())]
		pub fn create_order(
			origin: OriginFor<T>,
			para_id: ParaId,
			requirements: Requirements,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::OrderCreationFeeHandler::handle(&who, T::OrderCreationCost::get())?;

			let order_id = NextOrderId::<T>::get();
			Orders::<T>::insert(order_id, Order { creator: who.clone(), para_id, requirements });
			NextOrderId::<T>::put(order_id.saturating_add(1));

			Self::deposit_event(Event::OrderCreated { order_id, by: who });
			Ok(())
		}

		/// Extrinsic for cancelling an order.
		///
		/// If the region requirements on which the order was based are for an expired region,
		/// anyone can cancel the order.
		///
		/// ## Arguments:
		/// - `order_id`: The order the caller wants to cancel.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cancel_order())]
		pub fn cancel_order(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(order_id).ok_or(Error::<T>::InvalidOrderId)?;
			ensure!(
				order.creator == who || order.requirements.end < Self::current_timeslice(),
				Error::<T>::NotAllowed
			);

			Orders::<T>::remove(order_id);

			Self::deposit_event(Event::OrderRemoved { order_id, by: who });
			Ok(())
		}

		/// Extrinsic for contributing to an order.
		///
		/// ## Arguments:
		/// - `order_id`: The order to which the caller wants to contribute.
		/// - `amount`: The amount of tokens the user wants to contribute.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::contribute())]
		pub fn contribute(
			origin: OriginFor<T>,
			order_id: OrderId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let order = Orders::<T>::get(order_id).ok_or(Error::<T>::InvalidOrderId)?;
			ensure!(Self::current_timeslice() < order.requirements.end, Error::<T>::OrderExpired);

			ensure!(amount >= T::MinimumContribution::get(), Error::<T>::InvalidAmount);
			T::Currency::reserve(&who, amount)?;

			let mut contribution: BalanceOf<T> = Contributions::<T>::get(order_id, who.clone());
			contribution = contribution.saturating_add(amount);
			Contributions::<T>::insert(order_id, who.clone(), contribution);

			let mut total_contributions = TotalContributions::<T>::get(order_id);
			total_contributions = total_contributions.saturating_add(amount);
			TotalContributions::<T>::insert(order_id, total_contributions);

			Self::deposit_event(Event::Contributed { order_id, who, amount });

			Ok(())
		}

		/// Extrinsic for removing contributions from a cancelled order.
		///
		/// ## Arguments:
		/// - `order_id`: The cancelled order from which the user wants to claim back their
		///   contribution.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::remove_contribution())]
		pub fn remove_contribution(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Orders::<T>::get(order_id).is_none(), Error::<T>::OrderNotCancelled);

			let amount: BalanceOf<T> = Contributions::<T>::get(order_id, who.clone());
			ensure!(amount != Default::default(), Error::<T>::NoContribution);

			T::Currency::unreserve(&who, amount);
			Contributions::<T>::remove(order_id, who.clone());

			let mut total_contributions = TotalContributions::<T>::get(order_id);
			total_contributions = total_contributions.saturating_sub(amount);
			TotalContributions::<T>::insert(order_id, total_contributions);

			Self::deposit_event(Event::ContributionRemoved { who, order_id, amount });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn current_timeslice() -> Timeslice {
			let latest_rc_block = T::RCBlockNumberProvider::current_block_number();
			let timeslice_period = T::TimeslicePeriod::get();
			(latest_rc_block / timeslice_period).saturated_into()
		}
	}
}
