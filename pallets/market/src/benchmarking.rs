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
use frame_support::traits::fungible::Mutate;
use frame_system::RawOrigin;
use pallet_broker::{CoreMask, RegionId, RegionRecord};

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn list_region() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 8, owner: caller.clone(), paid: None };
		T::BenchmarkHelper::create_region(region_id, record, caller.clone())?;

		let timeslice_price: BalanceOf<T> = 1_000u32.into();
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id, timeslice_price, None);

		assert_last_event::<T>(
			Event::Listed {
				region_id,
				timeslice_price,
				seller: caller.clone(),
				sale_recipient: caller,
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn unlist_region() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 8, owner: caller.clone(), paid: None };
		T::BenchmarkHelper::create_region(region_id, record, caller.clone())?;

		let timeslice_price: BalanceOf<T> = 1_000u32.into();
		crate::Pallet::<T>::list_region(
			RawOrigin::Signed(caller.clone()).into(),
			region_id,
			timeslice_price,
			None,
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id);

		assert_last_event::<T>(Event::Unlisted { region_id }.into());

		Ok(())
	}

	#[benchmark]
	fn update_region_price() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 8, owner: caller.clone(), paid: None };
		T::BenchmarkHelper::create_region(region_id, record, caller.clone())?;

		crate::Pallet::<T>::list_region(
			RawOrigin::Signed(caller.clone()).into(),
			region_id,
			1_000u32.into(),
			None,
		)?;

		let new_timeslice_price = 2_000u32.into();
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id, new_timeslice_price);

		assert_last_event::<T>(Event::PriceUpdated { region_id, new_timeslice_price }.into());

		Ok(())
	}

	#[benchmark]
	fn purchase_region() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let alice: T::AccountId = account("alice", 0, SEED);

		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 8, owner: alice.clone(), paid: None };

		T::Currency::set_balance(&alice.clone(), u32::MAX.into());
		T::BenchmarkHelper::create_region(region_id, record, alice.clone())?;

		crate::Pallet::<T>::list_region(
			RawOrigin::Signed(alice).into(),
			region_id,
			1_000u32.into(),
			None,
		)?;

		T::Currency::set_balance(&caller.clone(), u32::MAX.into());
		let max_price = 8000u32.into();
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id, max_price);

		assert_last_event::<T>(
			Event::Purchased { region_id, total_price: max_price, buyer: caller }.into(),
		);

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
