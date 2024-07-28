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

//! Benchmarks for pallet-market

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::fungible::Mutate};
use frame_system::RawOrigin;
use pallet_broker::{CoreMask, RegionId, RegionRecord};
use sp_runtime::SaturatedConversion;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn fulfill_order() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let alice: T::AccountId = account("alice", 0, SEED);

		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 57600, // Full core.
		};

		<T as crate::Config>::Currency::make_free_balance_be(
			&alice.clone(),
			u64::MAX.saturated_into(),
		);
		assert_ok!(T::Orders::create_order(alice.clone(), 2000.into(), requirements.clone()));

		// Create a region which meets the requirements:
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 8, owner: caller.clone(), paid: None };
		T::Regions::create_region(region_id, record, caller.clone())?;

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0, region_id);

		assert_last_event::<T>(Event::RegionAssigned { region_id, para_id: 2000.into() }.into());

		Ok(())
	}

	#[benchmark]
	fn assign() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let para_id: ParaId = 2000.into();

		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		crate::RegionAssignments::<T>::insert(&region_id, para_id);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id);

		assert_last_event::<T>(Event::RegionAssigned { region_id, para_id }.into());

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(vec![]), crate::mock::Test);
}
