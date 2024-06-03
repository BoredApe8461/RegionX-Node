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

//! Benchmarks for pallet-orders

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_order() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let para_id: ParaId = 2000.into();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		<T as crate::Config>::Currency::make_free_balance_be(
			&caller.clone(),
			<T as crate::Config>::OrderCreationCost::get() * 2u32.into(),
		);
		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), para_id, requirements);

		assert_last_event::<T>(Event::OrderCreated { order_id: 0, by: caller }.into());

		Ok(())
	}

	#[benchmark]
	fn cancel_order() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let para_id: ParaId = 2000.into();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		<T as crate::Config>::Currency::make_free_balance_be(
			&caller.clone(),
			<T as crate::Config>::OrderCreationCost::get() * 2u32.into(),
		);
		crate::Pallet::<T>::create_order(
			RawOrigin::Signed(caller.clone()).into(),
			para_id,
			requirements,
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), 0);

		assert_last_event::<T>(Event::OrderRemoved { order_id: 0, by: caller }.into());

		Ok(())
	}

	#[benchmark]
	fn contribute() -> Result<(), BenchmarkError> {
		let creator: T::AccountId = whitelisted_caller();

		let para_id: ParaId = 2000.into();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		<T as crate::Config>::Currency::make_free_balance_be(
			&creator.clone(),
			(<T as crate::Config>::OrderCreationCost::get() +
				<T as crate::Config>::MinimumContribution::get()) *
				2u32.into(),
		);
		crate::Pallet::<T>::create_order(
			RawOrigin::Signed(creator.clone()).into(),
			para_id,
			requirements,
		)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(creator.clone()), 0, <T as crate::Config>::MinimumContribution::get());

		assert_last_event::<T>(
			Event::Contributed {
				order_id: 0,
				who: creator,
				amount: <T as crate::Config>::MinimumContribution::get(),
			}
			.into(),
		);

		Ok(())
	}

	#[benchmark]
	fn remove_contribution() -> Result<(), BenchmarkError> {
		let creator: T::AccountId = whitelisted_caller();

		let para_id: ParaId = 2000.into();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		<T as crate::Config>::Currency::make_free_balance_be(
			&creator.clone(),
			(<T as crate::Config>::OrderCreationCost::get() +
				<T as crate::Config>::MinimumContribution::get()) *
				2u32.into(),
		);
		crate::Pallet::<T>::create_order(
			RawOrigin::Signed(creator.clone()).into(),
			para_id,
			requirements,
		)?;
		crate::Pallet::<T>::contribute(
			RawOrigin::Signed(creator.clone()).into(),
			0,
			<T as crate::Config>::MinimumContribution::get(),
		)?;
		crate::Pallet::<T>::cancel_order(RawOrigin::Signed(creator.clone()).into(), 0)?;

		#[extrinsic_call]
		_(RawOrigin::Signed(creator.clone()), 0);

		assert_last_event::<T>(
			Event::ContributionRemoved {
				order_id: 0,
				who: creator,
				amount: <T as crate::Config>::MinimumContribution::get(),
			}
			.into(),
		);

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(vec![]), crate::mock::Test);
}
