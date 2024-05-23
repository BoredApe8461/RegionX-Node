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

use crate::{mock::*, *};
use frame_support::{
	assert_err, assert_ok,
	traits::{nonfungible::Mutate, Get},
};
use pallet_broker::{CoreMask, RegionRecord};

#[test]
fn calculate_region_price_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<u64, u64> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			80 // 8 * 10
		);

		// Remains same until a timeslice passes:
		RelayBlockNumber::set(79);
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<u64, u64> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			80 // 8 * 10
		);

		RelayBlockNumber::set(80);

		// Reduced by one after a timeslice elapses:
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<u64, u64> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			70 // 7 * 10
		);

		RelayBlockNumber::set(8 * 80);
		// Expired region has no value:
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<u64, u64> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			0
		);
	});
}

#[test]
fn list_region_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let signer = RuntimeOrigin::signed(seller);

		assert!(Regions::regions(&region_id).is_none());
		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecord<u64, u64> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;
		let recipient = 1;

		// Failure: Unknown region

		assert_err!(
			Market::list_region(signer.clone(), region_id, price, None),
			Error::<Test>::UnknownRegion
		);

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: Region expired
		RelayBlockNumber::set(10 * timeslice);

		assert_err!(
			Market::list_region(signer.clone(), region_id, price, None),
			Error::<Test>::RegionExpired
		);

		// Should be working
		RelayBlockNumber::set(1 * timeslice);
		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(recipient)));

		// Failure: Already listed
		assert_err!(
			Market::list_region(signer, region_id, price, None),
			Error::<Test>::AlreadyListed
		);

		// Check storage items
		assert_eq!(
			Market::listings(region_id),
			Some(Listing { seller, timeslice_price: price, sale_recipient: recipient })
		);

		// Check events
		System::assert_last_event(
			Event::Listed { region_id, timeslice_price: price, seller, sale_recipient: recipient }
				.into(),
		);
	});
}

#[test]
fn unlist_region_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let signer = RuntimeOrigin::signed(seller);

		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecord<u64, u64> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;
		let recipient = 1;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_err!(Market::unlist_region(signer.clone(), region_id), Error::<Test>::NotListed);

		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(recipient)));

		// Failure: NotAllowed
		RelayBlockNumber::set(1 * timeslice);
		assert_err!(
			Market::unlist_region(RuntimeOrigin::signed(3), region_id),
			Error::<Test>::NotAllowed
		);

		// Should be working now.
		assert_ok!(Market::unlist_region(signer, region_id));

		// Check storage items
		assert!(Market::listings(region_id).is_none());

		// Check events
		System::assert_last_event(Event::Unlisted { region_id }.into())
	});
}

#[test]
fn update_price_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let signer = RuntimeOrigin::signed(seller);

		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecord<u64, u64> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;
		let recipient = 1;
		let new_price = 2_000_000;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_err!(
			Market::update_region_price(signer.clone(), region_id, new_price),
			Error::<Test>::NotListed
		);

		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(recipient)));

		// Failure: NotAllowed - only the seller can update the price
		assert_err!(
			Market::update_region_price(RuntimeOrigin::signed(3), region_id, new_price),
			Error::<Test>::NotAllowed
		);

		// Failure: RegionExpired
		RelayBlockNumber::set(10 * timeslice);
		assert_err!(
			Market::update_region_price(signer.clone(), region_id, new_price),
			Error::<Test>::RegionExpired
		);

		// Should be working now
		RelayBlockNumber::set(2 * timeslice);
		assert_ok!(Market::update_region_price(signer, region_id, new_price));

		// Check storage
		assert_eq!(
			Market::listings(region_id),
			Some(Listing { seller, timeslice_price: new_price, sale_recipient: recipient })
		);
	});
}
