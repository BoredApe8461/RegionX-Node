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

use crate::assigner::{AssignmentCallEncoder, RegionAssigner};
use frame_support::{
	traits::{nonfungible::Transfer, Currency, ExistenceRequirement},
	weights::WeightToFee,
};
use nonfungible_primitives::LockableNonFungible;
use order_primitives::{OrderFactory, OrderId, OrderInspect, ParaId, Requirements};
pub use pallet::*;
use pallet_broker::{RegionId, RegionRecord};
use region_primitives::{RegionFactory, RegionInspect};
use sp_runtime::traits::Convert;
use xcm::opaque::lts::MultiLocation;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod assigner;

mod weights;
pub use weights::WeightInfo;

const LOG_TARGET: &str = "runtime::order-creator";

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type RegionRecordOf<T> = RegionRecord<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect, Mutate},
			tokens::Balance,
			ReservableCurrency,
		},
	};
	use frame_system::pallet_prelude::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency used for purchasing coretime.
		type Currency: Mutate<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Relay chain balance type
		type Balance: Balance
			+ Into<<Self::Currency as Inspect<Self::AccountId>>::Balance>
			+ Into<u128>;

		/// Type over which we can access order data.
		type Orders: OrderInspect<Self::AccountId> + OrderFactory<Self::AccountId>;

		/// A way for getting the associated account of an order.
		type OrderToAccountId: Convert<OrderId, Self::AccountId>;

		/// Type providing a way of reading, transferring and locking regions.
		//
		// The item id is `u128` encoded RegionId.
		type Regions: Transfer<Self::AccountId, ItemId = u128>
			+ LockableNonFungible<Self::AccountId, ItemId = u128>
			+ RegionInspect<Self::AccountId, BalanceOf<Self>, ItemId = u128>
			+ RegionFactory<Self::AccountId, RegionRecordOf<Self>>;

		/// Type assigning the region to the specified task.
		type RegionAssigner: RegionAssigner;

		/// Type whcih encodes the region assignment call.
		type AssignmentCallEncoder: AssignmentCallEncoder;

		/// Type for weight to fee conversion on the ReigonX parachain.
		type WeightToFee: WeightToFee<Balance = Self::Balance>;

		/// The Coretime chain from which we read region state.
		type CoretimeChain: Get<MultiLocation>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Specifies the assignment for each region.
	#[pallet::storage]
	#[pallet::getter(fn listings)]
	pub type RegionAssignments<T: Config> =
		StorageMap<_, Blake2_128Concat, RegionId, ParaId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Order got fulfilled with a region which is matching the requirements.
		OrderProcessed { order_id: OrderId, region_id: RegionId, seller: T::AccountId },
		/// Region got successfully assigned to a parachain.
		RegionAssigned { region_id: RegionId, para_id: ParaId },
		/// Region assignment failed.
		AssignmentFailed(DispatchError),
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// Region not found.
		UnknownRegion,
		/// Order not found.
		UnknownOrder,
		/// The region doesn't start when it should based on the requirements.
		RegionStartsTooLate,
		/// The region doesn't end when it should based on the requirements.
		RegionEndsTooSoon,
		/// Regions core mask doesn't match the requirements.
		RegionCoreOccupancyInsufficient,
		/// The region record is not available.
		RecordUnavailable,
		/// Locked regions cannot be listed on sale.
		RegionLocked,
		/// The caller is not the owner of the region.
		NotOwner,
		/// We didn't find the task to which the region is supposed to be assigned.
		RegionAssignmentNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Extrinsic for fulfilling an order.
		///
		/// This extrinsic will also attempt to assign the region to the `para_id` specified by the
		/// order.
		///
		/// In case this fails, the region will be tranferred to the order creator and anyone can
		/// call the `assign` extrinsic to assign it to specific para. The region will be locked,
		/// and only assignment is allowed.
		///
		/// ## Arguments:
		/// - `origin`: Signed origin; the region owner.
		/// - `order_id`: The order which the caller intends to fulfill.
		/// - `region_id`: The region that the caller intends to sell to the coretime order. The
		///   region must match the order requierements otherwise the extrinsic will fail
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::fulfill_order())]
		pub fn fulfill_order(
			origin: OriginFor<T>,
			order_id: OrderId,
			region_id: RegionId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let region = T::Regions::region(&region_id.into()).ok_or(Error::<T>::UnknownRegion)?;
			ensure!(!region.locked, Error::<T>::RegionLocked);

			ensure!(region.owner == who, Error::<T>::NotOwner);

			let record = region.record.get().ok_or(Error::<T>::RecordUnavailable)?;
			let order = T::Orders::order(&order_id).ok_or(Error::<T>::UnknownOrder)?;

			Self::ensure_matching_requirements(region_id, record, order.requirements)?;

			// Transfer the region to the order creator
			//
			// We will try to assign the region to the task, however before we do that we will
			// transfer it to the order creator. This way in case the assignment fails the region
			// will still be owned by the creator.
			T::Regions::transfer(&region_id.into(), &order.creator)?;
			// Lock the region so the order creator cannot transfer it.
			T::Regions::lock(&region_id.into(), None)?;
			// Even though the region will be owned by the creator, anyone can assign it to the task
			// by calling the `assign` extrinsic.
			RegionAssignments::<T>::insert(region_id, order.para_id);

			let order_account = T::OrderToAccountId::convert(order_id);
			let amount = T::Currency::free_balance(&order_account);

			<<T as Config>::Currency as Currency<T::AccountId>>::transfer(
				&order_account,
				&who,
				amount,
				ExistenceRequirement::AllowDeath,
			)?;

			// remove the order
			T::Orders::remove_order(&order_id);

			Self::deposit_event(Event::OrderProcessed { order_id, region_id, seller: who });

			// NOTE: if an error occurs we don't return error, we instead return ok and emit
			// appropriate event so the transaction doesn't get reverted in case the assignment
			// fails.
			if let Err(err) = T::RegionAssigner::assign(region_id, order.para_id) {
				Self::deposit_event(Event::AssignmentFailed(err));
				return Ok(())
			}

			Self::deposit_event(Event::RegionAssigned { region_id, para_id: order.para_id });
			Ok(())
		}

		/// Assign a region to the specific `para_id`.
		///
		/// ## Arguments:
		/// - `origin`: Signed origin; can be anyone.
		/// - `region_id`: The region that the caller intends assign. Must be found in the
		///   `RegionAssignments` mapping.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::assign())]
		pub fn assign(origin: OriginFor<T>, region_id: RegionId) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let para_id = RegionAssignments::<T>::get(region_id)
				.ok_or(Error::<T>::RegionAssignmentNotFound)?;

			T::RegionAssigner::assign(region_id, para_id)?;

			Self::deposit_event(Event::RegionAssigned { region_id, para_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn ensure_matching_requirements(
			region_id: RegionId,
			record: RegionRecordOf<T>,
			requirements: Requirements,
		) -> DispatchResult {
			ensure!(region_id.begin <= requirements.begin, Error::<T>::RegionStartsTooLate);
			ensure!(record.end >= requirements.end, Error::<T>::RegionEndsTooSoon);

			let mask_as_nominator = region_id.mask.count_ones() * (57600 / 80);
			ensure!(
				mask_as_nominator >= requirements.core_occupancy.into(),
				Error::<T>::RegionCoreOccupancyInsufficient
			);

			Ok(())
		}
	}
}
