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

use crate::*;
use codec::Encode;
use frame_support::{
	ensure,
	pallet_prelude::DispatchResult,
	traits::nonfungible::{Inspect, Mutate, Transfer},
};
use nonfungible_primitives::LockableNonFungible;

impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
	type ItemId = u128;

	fn owner(item: &Self::ItemId) -> Option<T::AccountId> {
		Regions::<T>::get(RegionId::from(*item)).map(|r| r.owner)
	}

	fn attribute(item: &Self::ItemId, key: &[u8]) -> Option<Vec<u8>> {
		let id = RegionId::from(*item);
		let record = Regions::<T>::get(id)?.record.get()?;
		match key {
			b"begin" => Some(id.begin.encode()),
			b"end" => Some(record.end.encode()),
			b"length" => Some(record.end.saturating_sub(id.begin).encode()),
			b"core" => Some(id.core.encode()),
			b"part" => Some(id.mask.encode()),
			b"owner" => Some(record.owner.encode()),
			b"paid" => Some(record.paid.encode()),
			_ => None,
		}
	}
}

impl<T: Config> Transfer<T::AccountId> for Pallet<T> {
	fn transfer(item: &Self::ItemId, dest: &T::AccountId) -> DispatchResult {
		Self::do_transfer((*item).into(), None, dest.clone()).map_err(Into::into)
	}
}

impl<T: Config> Mutate<T::AccountId> for Pallet<T> {
	/// Minting is used for depositing a region from the holding registar.
	fn mint_into(item: &Self::ItemId, who: &T::AccountId) -> DispatchResult {
		let region_id: RegionId = (*item).into();

		// We don't want the region to get trapped, so we will handle the error.
		let record = match Self::do_request_region_record(region_id, who.clone()) {
			Ok(commitment) => Record::Pending(commitment),
			Err(_) => {
				log::error!(
					target: LOG_TARGET,
					"Failed to request region record for region_id: {:?}",
					region_id
				);
				Record::Unavailable
			},
		};

		// Even if requesting the region record fails we still want to mint it to the owner.
		//
		// We will just have the region record not set.
		Regions::<T>::insert(region_id, Region { owner: who.clone(), locked: false, record });

		Ok(())
	}

	/// Burning is used for withdrawing a region into the holding registrar.
	fn burn(item: &Self::ItemId, maybe_check_owner: Option<&T::AccountId>) -> DispatchResult {
		let region_id: RegionId = (*item).into();

		let region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;
		if let Some(owner) = maybe_check_owner {
			ensure!(owner.clone() == region.owner, Error::<T>::NotOwner);
		}

		Regions::<T>::remove(region_id);

		Ok(())
	}
}

impl<T: Config> LockableNonFungible<T::AccountId> for Pallet<T> {
	fn lock(item: &Self::ItemId, maybe_check_owner: Option<T::AccountId>) -> DispatchResult {
		let region_id: RegionId = (*item).into();
		let mut region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;

		if let Some(owner) = maybe_check_owner {
			ensure!(owner.clone() == region.owner, Error::<T>::NotOwner);
		}
		ensure!(!region.locked, Error::<T>::RegionLocked);

		region.locked = true;
		Regions::<T>::insert(region_id, region);

		Ok(())
	}

	fn unlock(item: &Self::ItemId, maybe_check_owner: Option<T::AccountId>) -> DispatchResult {
		let region_id: RegionId = (*item).into();
		let mut region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;

		if let Some(owner) = maybe_check_owner {
			ensure!(owner.clone() == region.owner, Error::<T>::NotOwner);
		}
		ensure!(region.locked, Error::<T>::RegionNotLocked);

		region.locked = false;
		Regions::<T>::insert(region_id, region);

		Ok(())
	}
}
