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
use region_primitives::RegionInspect;

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

		// Insert the region even though we only know the `RegionId`.
		//
		// A region in this state is not very useful since we don't know what the region record
		// is.
		//
		// The record is therefore set to `Unavailable`. The user can call the
		// `request_region_record` extrinsic at any time to fetch the record from the Coretime
		// chain.
		Regions::<T>::insert(
			region_id,
			Region { owner: who.clone(), locked: false, record: Record::Unavailable },
		);

		Pallet::<T>::deposit_event(Event::RegionMinted { region_id });

		log::info!(
			target: LOG_TARGET,
			"Minted region: {:?}",
			region_id
		);

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

		Pallet::<T>::deposit_event(Event::RegionBurnt { region_id });

		Ok(())
	}
}

impl<T: Config> RegionInspect<T::AccountId, BalanceOf<T>> for Pallet<T> {
	type ItemId = u128;
	fn record(item: &Self::ItemId) -> Option<RegionRecordOf<T>> {
		let region_id: RegionId = (*item).into();
		let region = Regions::<T>::get(region_id)?;
		region.record.get()
	}

	fn region(item: &Self::ItemId) -> Option<RegionOf<T>> {
		let region_id: RegionId = (*item).into();
		Regions::<T>::get(region_id)
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

		Pallet::<T>::deposit_event(Event::RegionLocked { region_id });
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

		Pallet::<T>::deposit_event(Event::RegionUnlocked { region_id });

		Ok(())
	}
}
