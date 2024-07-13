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

use crate::{mock::*, Error, Event, Order, ParaId, Requirements};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, Get},
};
use sp_runtime::{traits::Convert, ArithmeticError, DispatchError, TokenError};

#[test]
fn create_order_works() {
	new_test_ext(vec![(BOB, 1000), (CHARLIE, 1000)]).execute_with(|| {
		let creator = ALICE;
		let para_id: ParaId = 2000.into();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		// Creating an order requires the caller to pay the creation fee.
		// The call fails with insufficient balance:
		assert_noop!(
			Orders::create_order(
				RuntimeOrigin::signed(creator.clone()),
				para_id,
				requirements.clone()
			),
			DispatchError::Token(TokenError::FundsUnavailable)
		);

		<Test as crate::Config>::Currency::make_free_balance_be(&creator, 1000u32.into());

		assert_ok!(Orders::create_order(
			RuntimeOrigin::signed(creator.clone()),
			para_id,
			requirements.clone()
		));

		// Check storage items
		assert_eq!(Orders::next_order_id(), 1);
		assert_eq!(Orders::orders(0), Some(Order { para_id, creator: ALICE, requirements }));
		assert!(Orders::orders(1).is_none());

		// Balance should be reduced due to fee payment:
		assert_eq!(Balances::free_balance(creator.clone()), 900);
		// Fee goes to the 'treasury':
		assert_eq!(Balances::free_balance(TREASURY), 100);

		// Check events
		System::assert_last_event(Event::OrderCreated { order_id: 0, by: creator }.into());
	});
}

#[test]
fn anyone_can_cancel_expired_order() {
	new_test_ext(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(|| {
		let creator = ALICE;
		let para_id: ParaId = 2000.into();
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		let requirements = Requirements {
			begin: 0,
			end: 8,
			core_occupancy: 28800, // Half of a core.
		};

		// Unknown order id
		assert_noop!(
			Orders::cancel_order(RuntimeOrigin::signed(creator.clone()), 0),
			Error::<Test>::InvalidOrderId
		);

		// Create an order:
		assert_ok!(Orders::create_order(
			RuntimeOrigin::signed(creator.clone()),
			para_id,
			requirements.clone()
		));

		// Cannot cancel a non-expired order:
		assert_noop!(
			Orders::cancel_order(RuntimeOrigin::signed(ALICE), 0),
			Error::<Test>::NotAllowed
		);

		// Anyone can cancel expired order:
		RelayBlockNumber::set(9 * timeslice);
		assert_ok!(Orders::cancel_order(RuntimeOrigin::signed(BOB), 0));

		// Check storage items
		assert!(Orders::orders(0).is_none());

		// Check events
		System::assert_last_event(Event::OrderRemoved { order_id: 0, by: BOB }.into());
	});
}

#[test]
fn contribute_works() {
	new_test_ext(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(|| {
		// Create an order
		assert_ok!(Orders::create_order(
			RuntimeOrigin::signed(ALICE),
			2000.into(),
			Requirements { begin: 0, end: 8, core_occupancy: 28800 }
		));

		// Invalid order id
		assert_noop!(
			Orders::contribute(RuntimeOrigin::signed(ALICE), 2, 1_000),
			Error::<Test>::InvalidOrderId
		);

		// Contribution amount is too small
		assert_noop!(
			Orders::contribute(RuntimeOrigin::signed(CHARLIE), 0, 0),
			Error::<Test>::InvalidAmount,
		);

		// Insufficient balance:
		assert_noop!(
			Orders::contribute(RuntimeOrigin::signed(CHARLIE), 0, 100_000),
			ArithmeticError::Underflow
		);

		assert_eq!(Orders::contributions(0, CHARLIE), 0);

		// Should work fine
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(CHARLIE), 0, 500));
		System::assert_last_event(
			Event::Contributed { order_id: 0, who: CHARLIE, amount: 500 }.into(),
		);

		assert_ok!(Orders::contribute(RuntimeOrigin::signed(BOB), 0, 100));
		System::assert_last_event(Event::Contributed { order_id: 0, who: BOB, amount: 100 }.into());
		// Check storage items
		assert_eq!(Orders::contributions(0, CHARLIE), 500);
		assert_eq!(Orders::contributions(0, BOB), 100);

		assert_eq!(Balances::free_balance(CHARLIE), 500);
		assert_eq!(Balances::free_balance(BOB), 900);
		let order_account = OrderToAccountId::convert(0);
		assert_eq!(Balances::free_balance(order_account), 600);

		// Additional contributions work:
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(CHARLIE), 0, 300));
		assert_eq!(Orders::contributions(0, CHARLIE), 800);
		assert_eq!(Orders::contributions(0, BOB), 100);

		// Cannot contribute to an expired order
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		RelayBlockNumber::set(timeslice * 9);

		assert_noop!(
			Orders::contribute(RuntimeOrigin::signed(ALICE), 0, 100),
			Error::<Test>::OrderExpired
		);

		assert_eq!(Orders::contributions(0, CHARLIE), 800);
		assert_eq!(Orders::contributions(0, BOB), 100);
	});
}

#[test]
fn remove_contribution_works() {
	new_test_ext(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(|| {
		// Create an order
		assert_ok!(Orders::create_order(
			RuntimeOrigin::signed(ALICE),
			2000.into(),
			Requirements { begin: 0, end: 8, core_occupancy: 28800 }
		));

		// Order is not cancelled
		assert_noop!(
			Orders::remove_contribution(RuntimeOrigin::signed(CHARLIE), 0),
			Error::<Test>::OrderNotCancelled
		);

		// Contribute to the order
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(CHARLIE), 0, 500));
		assert_ok!(Orders::contribute(RuntimeOrigin::signed(BOB), 0, 200));

		assert_eq!(Balances::free_balance(CHARLIE), 500);
		let order_account = OrderToAccountId::convert(0);
		assert_eq!(Balances::free_balance(order_account.clone()), 700);

		// Cancel the expired order:
		let timeslice: u64 = <Test as crate::Config>::TimeslicePeriod::get();
		RelayBlockNumber::set(9 * timeslice);
		assert_ok!(Orders::cancel_order(RuntimeOrigin::signed(ALICE), 0));

		// Should work fine
		assert_ok!(Orders::remove_contribution(RuntimeOrigin::signed(CHARLIE), 0));

		// Check storage items
		assert_eq!(Balances::free_balance(CHARLIE), 1000);
		assert_eq!(Balances::free_balance(order_account), 200);

		// Check the events
		System::assert_last_event(
			Event::ContributionRemoved { order_id: 0, who: CHARLIE, amount: 500 }.into(),
		);

		assert_noop!(
			Orders::remove_contribution(RuntimeOrigin::signed(CHARLIE), 0),
			Error::<Test>::NoContribution
		);
	});
}
