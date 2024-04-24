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

//! Benchmarking setup for pallet-regions

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::nonfungible::Mutate};
use frame_system::RawOrigin;
use pallet_broker::{CoreMask, RegionId};

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn transfer() -> Result<(), BenchmarkError> {
		let caller = whitelisted_caller();
		let new_owner: T::AccountId = account("alice", 0, SEED);
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		// We first mint a region.
		assert_ok!(crate::Pallet::<T>::mint_into(&region_id.into(), &caller));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id, new_owner.clone());

		assert_last_event::<T>(
			Event::Transferred { region_id, old_owner: caller, owner: new_owner }.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn request_region_record() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		// Create a region with an unavailable record, allowing us to re-request the record.
		crate::Regions::<T>::insert(
			region_id,
			Region { owner: caller.clone(), record: Record::<T>::Unavailable },
		);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id);

		assert_last_event::<T>(Event::RegionRecordRequested { region_id, account: caller }.into());

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
