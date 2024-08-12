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

use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::{assert_err, assert_ok, traits::nonfungible::Mutate};
use frame_system::RawOrigin;
use ismp::router::{Get as IsmpGet, GetResponse};
use pallet_broker::{CoreMask, RegionId, RegionRecord};
use sp_core::Get;

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

		assert_ok!(crate::Pallet::<T>::mint_into(&region_id.into(), &caller));

		#[extrinsic_call]
		_(RawOrigin::None, region_id);
		assert!(crate::Pallet::<T>::regions(&region_id).unwrap().record.is_pending());

		Ok(())
	}

	#[benchmark]
	fn drop_region() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let owner: T::AccountId = account("alice", 0, SEED);

		let region_id = RegionId { begin: 0, core: 72, mask: CoreMask::complete() };
		let record: RegionRecordOf<T> = RegionRecord { end: 0, owner, paid: None };

		assert_ok!(crate::Pallet::<T>::mint_into(&region_id.into(), &caller));
		assert_ok!(crate::Pallet::<T>::request_region_record(RawOrigin::None.into(), region_id));
		assert_ok!(crate::Pallet::<T>::set_record(region_id, record));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), region_id);

		assert_last_event::<T>(Event::RegionDropped { region_id, who: caller }.into());

		Ok(())
	}

	#[benchmark]
	fn on_accept() -> Result<(), BenchmarkError> {
		let module = IsmpModuleCallback::<T>::default();
		#[block]
		{
			// We don't support ISMP post.
			assert_err!(
				module.on_accept(Post {
					source: <T as crate::Config>::CoretimeChain::get(),
					dest: <T as crate::Config>::CoretimeChain::get(),
					nonce: Default::default(),
					from: Default::default(),
					to: Default::default(),
					timeout_timestamp: Default::default(),
					data: Default::default(),
				}),
				IsmpCustomError::NotSupported
			);
		}

		Ok(())
	}

	#[benchmark]
	fn on_response() -> Result<(), BenchmarkError> {
		let module = IsmpModuleCallback::<T>::default();

		let owner = whitelisted_caller();
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_ok!(crate::Pallet::<T>::mint_into(&region_id.into(), &owner));

		// We can use mock data for everything except the key since on_response is not reading
		// anything other than the key from the request.
		let key =
			crate::Pallet::<T>::region_storage_key(region_id).expect("Failed to get storage key");
		let get = IsmpGet {
			source: T::CoretimeChain::get(),
			dest: T::CoretimeChain::get(),
			nonce: 0,
			from: Default::default(),
			keys: vec![key.clone()],
			height: Default::default(),
			timeout_timestamp: 0,
		};

		let mock_record: RegionRecordOf<T> = RegionRecord { end: 113000, owner, paid: None };

		let mock_response = Response::Get(GetResponse {
			get: get.clone(),
			values: BTreeMap::from([(key, Some(mock_record.encode()))]),
		});

		#[block]
		{
			assert_ok!(module.on_response(mock_response));
		}

		assert!(crate::Pallet::<T>::regions(&region_id).unwrap().record.is_available());

		Ok(())
	}

	#[benchmark]
	fn on_timeout() -> Result<(), BenchmarkError> {
		let module = IsmpModuleCallback::<T>::default();

		let owner = whitelisted_caller();
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_ok!(crate::Pallet::<T>::mint_into(&region_id.into(), &owner));

		// We can use mock data for everything except the key since on_response is not reading
		// anything other than the key from the request.
		let key =
			crate::Pallet::<T>::region_storage_key(region_id).expect("Failed to get storage key");
		let get = IsmpGet {
			source: T::CoretimeChain::get(),
			dest: T::CoretimeChain::get(),
			nonce: 0,
			from: Default::default(),
			keys: vec![key.clone()],
			height: Default::default(),
			timeout_timestamp: 0,
		};
		let timeout = Timeout::Request(Request::Get(get.clone()));

		#[block]
		{
			assert_ok!(module.on_timeout(timeout));
		}

		assert!(crate::Pallet::<T>::regions(&region_id).unwrap().record.is_unavailable());

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
