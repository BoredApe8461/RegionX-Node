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

use crate::{
	mock::{
		assignments, new_test_ext, Balances, Orders, Processor, Regions, RuntimeOrigin, System,
		Test,
	},
	Error, Event,
};
use frame_support::{
	assert_noop, assert_ok,
	traits::{nonfungible::Mutate, Currency},
};
use nonfungible_primitives::LockableNonFungible;
use order_primitives::{Order, ParaId, Requirements};
use pallet_broker::{CoreMask, RegionId, RegionRecord};

#[test]
fn fulfill_order_works() {
	new_test_ext(vec![(2000, 1000), (10, 1000), (11, 1000), (12, 1000)]).execute_with(|| {
		let region_owner = 1;
		let order_creator = 2000;
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		// 1. create an order
		<Test as crate::Config>::Currency::make_free_balance_be(&order_creator, 1000u32.into());
		assert_ok!(Orders::create_order(
			RuntimeOrigin::signed(order_creator.clone()),
			2000.into(),
			requirements.clone()
		));
		assert_eq!(
			Orders::orders(0),
			Some(Order { para_id: 2000.into(), creator: order_creator, requirements })
		);

		// 2. make contributions to an order
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(10), 0, 500));
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(11), 0, 800));
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(12), 0, 200));

		// Fulfill order fails with a region that doesn't meet the requirements:
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::from_chunk(0, 10) };
		assert_ok!(Regions::mint_into(&region_id.into(), &region_owner));
		assert_ok!(Regions::set_record(region_id, RegionRecord { end: 8, owner: 1, paid: None }));
		assert_noop!(
			Processor::fulfill_order(RuntimeOrigin::signed(region_owner), 0, region_id),
			Error::<Test>::RegionCoreOccupancyInsufficient
		);

		// Create a region which meets the requirements:
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		assert_ok!(Regions::mint_into(&region_id.into(), &region_owner));
		assert_ok!(Regions::set_record(region_id, RegionRecord { end: 8, owner: 1, paid: None }));

		// Fails if the region is locked:
		Regions::lock(&region_id.into(), None).unwrap();
		assert_noop!(
			Processor::fulfill_order(RuntimeOrigin::signed(region_owner), 0, region_id),
			Error::<Test>::RegionLocked
		);

		// Works with a region that meets the requirements and is unlocked:

		Regions::unlock(&region_id.into(), None).unwrap();
		assert_ok!(Processor::fulfill_order(RuntimeOrigin::signed(region_owner), 0, region_id));
		// Check events
		System::assert_has_event(Event::RegionAssigned { region_id, para_id: 2000.into() }.into());
		System::assert_has_event(
			Event::OrderProcessed { order_id: 0, region_id, seller: region_owner }.into(),
		);

		// Ensure order is removed:
		assert!(Orders::orders(0).is_none());

		// Region owner receives as the contributions for fulfilling the order:
		assert_eq!(Balances::free_balance(region_owner), 1500);
		// The region is removed since the assignment was successful:
		assert!(Regions::regions(region_id).is_none());

		// Assignment request is emmited:
		assert_eq!(assignments(), vec![(region_id, 2000.into())]);
	});
}

#[test]
fn assign_works() {
	new_test_ext(vec![]).execute_with(|| {
		let para_id: ParaId = 2000.into();
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };

		// Fails if the record cannot be found in `RegionAssignments`
		assert_noop!(
			Processor::assign(RuntimeOrigin::signed(1), region_id),
			Error::<Test>::RegionAssignmentNotFound
		);

		Regions::mint_into(&region_id.into(), &1).unwrap();
		crate::RegionAssignments::<Test>::insert(&region_id, para_id);

		assert_ok!(Processor::assign(RuntimeOrigin::signed(1), region_id));
		System::assert_last_event(Event::RegionAssigned { region_id, para_id: 2000.into() }.into());
	});
}

#[test]
fn ensure_matching_requirements_works() {
	new_test_ext(vec![]).execute_with(|| {
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		// Region starts too late:
		assert_noop!(
			Processor::ensure_matching_requirements(
				RegionId { begin: 2, core: 0, mask: CoreMask::complete() },
				RegionRecord { end: 10, owner: 1, paid: None },
				requirements.clone()
			),
			Error::<Test>::RegionStartsTooLate
		);

		// Region ends too soon:
		assert_noop!(
			Processor::ensure_matching_requirements(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecord { end: 4, owner: 1, paid: None },
				requirements.clone()
			),
			Error::<Test>::RegionEndsTooSoon
		);

		// Region core occupancy insufficient:
		assert_noop!(
			Processor::ensure_matching_requirements(
				RegionId { begin: 0, core: 0, mask: CoreMask::from_chunk(0, 39) },
				RegionRecord { end: 8, owner: 1, paid: None },
				requirements.clone()
			),
			Error::<Test>::RegionCoreOccupancyInsufficient
		);

		// Works when all requirements are met:
		assert_ok!(Processor::ensure_matching_requirements(
			RegionId { begin: 0, core: 0, mask: CoreMask::from_chunk(0, 40) },
			RegionRecord { end: 8, owner: 1, paid: None },
			requirements.clone()
		),);
	})
}
