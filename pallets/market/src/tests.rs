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
	assert_noop, assert_ok,
	traits::{nonfungible::Mutate, Get},
};
use pallet_broker::{CoreMask, RegionRecord};
use sp_runtime::{DispatchError::Token, TokenError};

#[test]
fn calculate_region_price_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<Test> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			80 // 8 * 10
		);

		// Remains same until a timeslice passes:
		RelayBlockNumber::set(79);
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<Test> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			80 // 8 * 10
		);

		RelayBlockNumber::set(80);

		// Reduced by one after a timeslice elapses:
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<Test> { end: 8, owner: 1, paid: None },
				10 // timeslice price
			),
			70 // 7 * 10
		);

		RelayBlockNumber::set(8 * 80);
		// Expired region has no value:
		assert_eq!(
			Market::calculate_region_price(
				RegionId { begin: 0, core: 0, mask: CoreMask::complete() },
				RegionRecordOf::<Test> { end: 8, owner: 1, paid: None },
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

		let record: RegionRecordOf<Test> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;
		let recipient = 1;

		// Failure: Unknown region

		assert_noop!(
			Market::list_region(signer.clone(), region_id, price, None),
			Error::<Test>::RecordUnavailable
		);

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: Region expired
		RelayBlockNumber::set(10 * timeslice);

		assert_noop!(
			Market::list_region(signer.clone(), region_id, price, None),
			Error::<Test>::RegionExpired
		);

		// Should be working
		RelayBlockNumber::set(1 * timeslice);
		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(recipient)));

		// Failure: Already listed
		assert_noop!(
			Market::list_region(signer, region_id, price, None),
			Error::<Test>::AlreadyListed
		);

		// Check storage items
		assert_eq!(
			Market::listings(region_id),
			Some(Listing { seller, timeslice_price: price, sale_recipient: recipient })
		);

		assert!(Regions::regions(region_id).unwrap().locked);

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

		let record: RegionRecordOf<Test> = RegionRecord { end: 8, owner: seller, paid: None };
		let price = 1_000_000;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_noop!(Market::unlist_region(signer.clone(), region_id), Error::<Test>::NotListed);

		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(seller)));
		assert_eq!(
			Market::listings(region_id),
			Some(Listing { seller, timeslice_price: price, sale_recipient: seller })
		);

		// Failure: NotAllowed
		assert_noop!(
			Market::unlist_region(RuntimeOrigin::signed(3), region_id),
			Error::<Test>::NotAllowed
		);

		// Should be working now.
		assert_ok!(Market::unlist_region(signer, region_id));

		// Check storage items
		assert!(Market::listings(region_id).is_none());
		assert!(Regions::regions(region_id).unwrap().locked == false);

		// Check events
		System::assert_last_event(Event::Unlisted { region_id }.into())
	});
}

#[test]
fn unlist_expired_region_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let signer = RuntimeOrigin::signed(seller);

		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecordOf<Test> = RegionRecord { end: 8, owner: seller, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_noop!(Market::unlist_region(signer.clone(), region_id), Error::<Test>::NotListed);

		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(seller)));
		assert_eq!(
			Market::listings(region_id),
			Some(Listing { seller, timeslice_price: price, sale_recipient: seller })
		);

		RelayBlockNumber::set(9 * timeslice);

		// Anyone can unlist an expired region.
		assert_ok!(Market::unlist_region(RuntimeOrigin::signed(3), region_id));

		// Check events
		System::assert_last_event(Event::Unlisted { region_id }.into());

		// Check storage items
		assert!(Market::listings(region_id).is_none());
		assert!(Regions::regions(region_id).unwrap().locked == false);
	});
}

#[test]
fn update_region_price_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let signer = RuntimeOrigin::signed(seller);

		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecordOf<Test> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let price = 1_000_000;
		let recipient = 1;
		let new_timeslice_price = 2_000_000;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_noop!(
			Market::update_region_price(signer.clone(), region_id, new_timeslice_price),
			Error::<Test>::NotListed
		);

		assert_ok!(Market::list_region(signer.clone(), region_id, price, Some(recipient)));

		// Failure: NotAllowed - only the seller can update the price
		assert_noop!(
			Market::update_region_price(RuntimeOrigin::signed(3), region_id, new_timeslice_price),
			Error::<Test>::NotAllowed
		);

		// Failure: RegionExpired
		RelayBlockNumber::set(10 * timeslice);
		assert_noop!(
			Market::update_region_price(signer.clone(), region_id, new_timeslice_price),
			Error::<Test>::RegionExpired
		);

		// Should be working now
		RelayBlockNumber::set(2 * timeslice);
		assert_ok!(Market::update_region_price(signer, region_id, new_timeslice_price));

		// Check storage
		assert_eq!(
			Market::listings(region_id),
			Some(Listing {
				seller,
				timeslice_price: new_timeslice_price,
				sale_recipient: recipient
			})
		);

		// Check events
		System::assert_last_event(
			Event::<Test>::PriceUpdated { region_id, new_timeslice_price }.into(),
		);
	});
}

#[test]
fn purchase_region_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 0, mask: CoreMask::complete() };
		let seller = 2;
		let buyer = 3;

		assert_ok!(Regions::mint_into(&region_id.into(), &seller));

		let record: RegionRecordOf<Test> = RegionRecord { end: 8, owner: 1, paid: None };
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let timeslice_price = 1_000_000;
		let recipient = 1;

		assert_ok!(Regions::set_record(region_id, record.clone()));

		// Failure: NotListed
		assert_noop!(
			Market::purchase_region(RuntimeOrigin::signed(seller), region_id, 1 * timeslice_price),
			Error::<Test>::NotListed
		);

		assert_ok!(Market::list_region(
			RuntimeOrigin::signed(seller),
			region_id,
			timeslice_price,
			Some(recipient)
		));

		// Failure: NotAllowed
		assert_noop!(
			Market::purchase_region(RuntimeOrigin::signed(seller), region_id, timeslice_price),
			Error::<Test>::NotAllowed
		);
		assert_noop!(
			Market::purchase_region(RuntimeOrigin::signed(recipient), region_id, timeslice_price),
			Error::<Test>::NotAllowed
		);

		// Failure: PriceTooHigh
		RelayBlockNumber::set(timeslice);
		assert_noop!(
			Market::purchase_region(RuntimeOrigin::signed(buyer), region_id, timeslice_price),
			Error::<Test>::PriceTooHigh
		);

		// Failure: Insufficient Balance
		let balance_buyer_old = Balances::free_balance(buyer);
		assert_ok!(Balances::transfer_keep_alive(
			RuntimeOrigin::signed(buyer),
			seller,
			balance_buyer_old.saturating_sub(3 * timeslice_price),
		));
		assert_noop!(
			Market::purchase_region(RuntimeOrigin::signed(buyer), region_id, 8 * timeslice_price),
			Token(TokenError::FundsUnavailable)
		);
		assert_ok!(Balances::transfer_keep_alive(
			RuntimeOrigin::signed(seller),
			buyer,
			2 * timeslice_price
		));

		// Should be working
		let balance_recipient_old = Balances::free_balance(recipient);
		let balance_buyer_old = Balances::free_balance(buyer);

		RelayBlockNumber::set(4 * timeslice);
		let price = 4 * timeslice_price;
		assert_eq!(Market::calculate_region_price(region_id, record, timeslice_price), price);
		assert_ok!(Market::purchase_region(
			RuntimeOrigin::signed(buyer),
			region_id,
			5 * timeslice_price
		));

		// Check storage items
		assert!(Market::listings(region_id).is_none());
		assert!(Regions::regions(region_id).unwrap().locked == false);

		// Check events
		System::assert_last_event(Event::Purchased { region_id, buyer, total_price: price }.into());

		// Check account balances
		let balance_recipient = Balances::free_balance(recipient);
		assert_eq!(balance_recipient, balance_recipient_old + price);

		let balance_buyer = Balances::free_balance(buyer);
		assert_eq!(balance_buyer.saturating_add(price), balance_buyer_old);
	});
}
