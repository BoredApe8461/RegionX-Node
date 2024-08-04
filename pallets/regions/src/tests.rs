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
	ismp_mock::requests, mock::*, pallet::Regions as RegionsStorage, types::RegionRecordOf, utils,
	Error, Event, IsmpCustomError, IsmpModuleCallback, Record, Region,
};
use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::*,
	traits::nonfungible::{Inspect, Mutate, Transfer as NonFungibleTransfer},
};
use ismp::{
	module::IsmpModule,
	router::{GetResponse, Post, PostResponse, Request, Response, Timeout},
};
use nonfungible_primitives::LockableNonFungible;
use pallet_broker::{CoreMask, RegionId, RegionRecord};
use region_primitives::RegionInspect;
use std::collections::BTreeMap;

// pallet hash + storage item hash
const REGION_PREFIX_KEY: &str = "4dcb50595177a3177648411a42aca0f53dc63b0b76ffd6f80704a090da6f8719";

#[test]
fn nonfungibles_implementation_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert!(Regions::regions(&region_id).is_none());
		assert_ok!(Regions::mint_into(&region_id.into(), &2));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 2, locked: false, record: Record::Unavailable }
		);

		// The user is not required to set the region record to withdraw the asset back to the
		// coretime chain.
		//
		// NOTE: Burning occurs when placing the region into the XCM holding registrar at the time
		// of reserve transfer.

		assert_noop!(Regions::burn(&region_id.into(), Some(&1)), Error::<Test>::NotOwner);

		assert_ok!(Regions::burn(&region_id.into(), Some(&2)));
		assert!(Regions::regions(&region_id).is_none());

		assert_noop!(Regions::burn(&region_id.into(), None), Error::<Test>::UnknownRegion);
	});
}

#[test]
fn set_record_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 81, mask: CoreMask::complete() };
		let record: RegionRecordOf<Test> = RegionRecord { end: 123600, owner: 1, paid: None };

		// The region with the given `region_id` does not exist.
		assert_noop!(Regions::set_record(region_id, record.clone()), Error::<Test>::UnknownRegion);

		// `set_record` succeeds:

		assert_ok!(Regions::mint_into(&region_id.into(), &2));
		assert_ok!(Regions::request_region_record(RuntimeOrigin::none(), region_id));

		assert!(Regions::regions(region_id).is_some());
		let region = Regions::regions(region_id).unwrap();
		assert!(region.record.is_pending());

		assert_ok!(Regions::set_record(region_id, record.clone()));
		System::assert_last_event(Event::RecordSet { region_id }.into());

		// check storage
		assert!(Regions::regions(region_id).is_some());
		let region = Regions::regions(region_id).unwrap();
		assert!(region.record.is_available());
		assert_eq!(region.owner, 2);
		assert_eq!(region.record, Record::Available(record.clone()));

		// call `set_record` again with the same record
		assert_noop!(Regions::set_record(region_id, record), Error::<Test>::RegionRecordAlreadySet);
	});
}

#[test]
fn request_region_record_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 81, mask: CoreMask::complete() };

		// fails to request unknown regions
		assert_noop!(
			Regions::request_region_record(RuntimeOrigin::none(), region_id),
			Error::<Test>::UnknownRegion
		);

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_ok!(Regions::request_region_record(RuntimeOrigin::none(), region_id));

		assert!(Regions::regions(region_id).is_some());
		let region = Regions::regions(region_id).unwrap();
		assert!(region.record.is_pending());
		// Cannot request if there is already a request pending.
		assert_noop!(
			Regions::request_region_record(RuntimeOrigin::none(), region_id),
			Error::<Test>::NotUnavailable
		);

		RegionsStorage::<Test>::mutate_exists(region_id, |val| {
			let mut v0 = val.clone().unwrap();
			v0.record = Record::Unavailable;
			*val = Some(v0);
		});

		assert_ok!(Regions::request_region_record(RuntimeOrigin::none(), region_id));
		assert!(region.record.is_pending());
		let request = &requests()[0];
		let Request::Get(get) = request.request.clone() else { panic!("Expected GET request") };

		// ensure correct storage key prefix:
		let hex_encoded = hex::encode(get.keys[0].clone());
		let key_length = hex_encoded.len();
		// `key_length - 64` because the key contains the hash and the encoded region id.
		// The hash is 16 bytes (32 nibbles), and the region id is also 16 bytes (32 nibbles).
		let key_prefix = &hex_encoded[..(key_length - 64)];
		assert_eq!(key_prefix, REGION_PREFIX_KEY);

		System::assert_last_event(
			Event::<Test>::RegionRecordRequested {
				region_id,
				request_commitment: Default::default(),
			}
			.into(),
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		// cannot transfer an unknown region
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };
		assert!(Regions::regions(region_id).is_none());

		assert_noop!(
			Regions::transfer(RuntimeOrigin::signed(1), region_id, 2),
			Error::<Test>::UnknownRegion
		);

		// only regions owned by the caller are transferable
		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_noop!(
			Regions::transfer(RuntimeOrigin::signed(3), region_id, 2),
			Error::<Test>::NotOwner
		);

		// transfer region success
		assert_ok!(Regions::transfer(RuntimeOrigin::signed(1), region_id, 2));

		System::assert_last_event(Event::Transferred { region_id, old_owner: 1, owner: 2 }.into());

		// check storage item
		assert!(Regions::regions(region_id).is_some());
		let region = Regions::regions(region_id).unwrap();

		assert_eq!(region.owner, 2);
	});
}

#[test]
fn on_response_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_ok!(Regions::mint_into(&region_id.into(), &2));
		assert_ok!(Regions::request_region_record(RuntimeOrigin::none(), region_id));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 2, locked: false, record: Record::Pending(Default::default()) }
		);

		let request = &requests()[0];
		let Request::Get(get) = request.request.clone() else { panic!("Expected GET request") };

		assert_eq!(request.who, 2);

		let mock_record: RegionRecordOf<Test> = RegionRecord { end: 113000, owner: 1, paid: None };

		let mock_response = Response::Get(GetResponse {
			get: get.clone(),
			values: BTreeMap::from([(get.keys[0].clone(), Some(mock_record.encode()))]),
		});

		let module: IsmpModuleCallback<Test> = IsmpModuleCallback::default();
		assert_ok!(module.on_response(mock_response));

		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 2, locked: false, record: Record::Available(mock_record.clone()) }
		);

		// Fails when invalid region id is passed as response:
		let mut invalid_get_req = get.clone();
		invalid_get_req.keys[0] = vec![0x23; 15];
		assert_noop!(
			module.on_response(Response::Get(GetResponse {
				get: invalid_get_req.clone(),
				values: BTreeMap::from([(
					invalid_get_req.keys[0].clone(),
					Some(mock_record.clone().encode())
				)]),
			})),
			IsmpCustomError::KeyDecodeFailed
		);

		// Fails when invalid region record is passed as response:
		assert_noop!(
			module.on_response(Response::Get(GetResponse {
				get: get.clone(),
				values: BTreeMap::from([(get.keys[0].clone(), Some(vec![0x42; 20]))]),
			})),
			IsmpCustomError::ResponseDecodeFailed
		);
	});
}

#[test]
fn on_response_only_handles_get() {
	new_test_ext().execute_with(|| {
		let module: IsmpModuleCallback<Test> = IsmpModuleCallback::default();

		let mock_response = Response::Post(PostResponse {
			post: Post {
				source: <Test as crate::Config>::CoretimeChain::get(),
				dest: <Test as crate::Config>::CoretimeChain::get(),
				nonce: Default::default(),
				from: Default::default(),
				to: Default::default(),
				timeout_timestamp: Default::default(),
				data: Default::default(),
			},
			response: Default::default(),
			timeout_timestamp: Default::default(),
		});

		assert_noop!(module.on_response(mock_response), IsmpCustomError::NotSupported);
	});
}

#[test]
fn on_timeout_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 72, mask: CoreMask::complete() };

		assert_ok!(Regions::mint_into(&region_id.into(), &2));
		assert_ok!(Regions::request_region_record(RuntimeOrigin::none(), region_id));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 2, locked: false, record: Record::Pending(Default::default()) }
		);

		let request = &requests()[0];

		let Request::Get(get) = request.request.clone() else { panic!("Expected GET request") };

		let module: IsmpModuleCallback<Test> = IsmpModuleCallback::default();
		let timeout = Timeout::Request(Request::Get(get.clone()));
		assert_ok!(module.on_timeout(timeout));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 2, locked: false, record: Record::Unavailable }
		);

		// failed to decode region_id
		let mut invalid_get_req = get.clone();
		invalid_get_req.keys.push(vec![0u8; 15]);
		assert_noop!(
			module.on_timeout(Timeout::Request(Request::Get(invalid_get_req.clone()))),
			IsmpCustomError::KeyDecodeFailed
		);

		// invalid id: region not found
		invalid_get_req.keys.pop();
		if let Some(key) = invalid_get_req.keys.get_mut(0) {
			for i in 0..key.len() {
				key[i] = key[i].reverse_bits();
			}
		}
		assert_noop!(
			module.on_timeout(Timeout::Request(Request::Get(invalid_get_req.clone()))),
			IsmpCustomError::RegionNotFound
		);

		let post = Post {
			source: <Test as crate::Config>::CoretimeChain::get(),
			dest: <Test as crate::Config>::CoretimeChain::get(),
			nonce: Default::default(),
			from: Default::default(),
			to: Default::default(),
			timeout_timestamp: Default::default(),
			data: Default::default(),
		};
		assert_ok!(module.on_timeout(Timeout::Request(Request::Post(post.clone()))));

		assert_ok!(module.on_timeout(Timeout::Response(PostResponse {
			post,
			response: Default::default(),
			timeout_timestamp: Default::default()
		})));
	});
}

#[test]
fn on_accept_works() {
	new_test_ext().execute_with(|| {
		let post = Post {
			source: <Test as crate::Config>::CoretimeChain::get(),
			dest: <Test as crate::Config>::CoretimeChain::get(),
			nonce: 0,
			from: Default::default(),
			to: Default::default(),
			timeout_timestamp: 0,
			data: Default::default(),
		};
		let module: IsmpModuleCallback<Test> = IsmpModuleCallback::default();
		assert_noop!(module.on_accept(post), IsmpCustomError::NotSupported);
	});
}

#[test]
fn nonfungible_owner_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 0, core: 72, mask: CoreMask::complete() };
		assert!(Regions::owner(&0).is_none());

		assert!(Regions::owner(&region_id.into()).is_none());
		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_eq!(Regions::owner(&region_id.into()), Some(1));
	});
}

#[test]
fn nonfungible_attribute_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };
		let record: RegionRecordOf<Test> = RegionRecord { end: 123600, owner: 1, paid: None };

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_ok!(Regions::set_record(region_id, record.clone()));

		assert!(Regions::attribute(&region_id.into(), "none".as_bytes().into()).is_none());
		assert_eq!(
			Regions::attribute(&region_id.into(), "begin".as_bytes()),
			Some(region_id.begin.encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "end".as_bytes()),
			Some(record.end.encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "length".as_bytes()),
			Some((record.end.saturating_sub(region_id.begin)).encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "core".as_bytes()),
			Some(region_id.core.encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "part".as_bytes()),
			Some(region_id.mask.encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "owner".as_bytes()),
			Some(record.owner.encode())
		);
		assert_eq!(
			Regions::attribute(&region_id.into(), "paid".as_bytes()),
			Some(record.paid.encode())
		);
	});
}

#[test]
fn nonfungible_transfer_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_eq!(Regions::owner(&region_id.into()), Some(1));

		assert_ok!(
			<Regions as NonFungibleTransfer::<<Test as frame_system::Config>::AccountId>>::transfer(
				&region_id.into(),
				&2
			)
		);
		assert_eq!(Regions::owner(&region_id.into()), Some(2));
	});
}

#[test]
fn region_locking_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_noop!(Regions::lock(&region_id.into(), Some(1)), Error::<Test>::UnknownRegion);

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_eq!(Regions::owner(&region_id.into()), Some(1));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 1, locked: false, record: Record::Unavailable }
		);

		// Must be the region owner:
		assert_noop!(Regions::lock(&region_id.into(), Some(2)), Error::<Test>::NotOwner);

		assert_ok!(Regions::lock(&region_id.into(), Some(1)));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 1, locked: true, record: Record::Unavailable }
		);

		assert_noop!(Regions::lock(&region_id.into(), Some(1)), Error::<Test>::RegionLocked);

		// Can't transfer locked region:
		assert_noop!(
			<Regions as NonFungibleTransfer::<<Test as frame_system::Config>::AccountId>>::transfer(
				&region_id.into(),
				&2
			),
			Error::<Test>::RegionLocked
		);
	});
}

#[test]
fn region_unlocking_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };

		assert_noop!(Regions::unlock(&region_id.into(), Some(1)), Error::<Test>::UnknownRegion);

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		assert_eq!(Regions::owner(&region_id.into()), Some(1));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 1, locked: false, record: Record::Unavailable }
		);
		assert_noop!(Regions::unlock(&region_id.into(), Some(1)), Error::<Test>::RegionNotLocked);

		assert_ok!(Regions::lock(&region_id.into(), Some(1)));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 1, locked: true, record: Record::Unavailable }
		);

		// Must be the region owner:
		assert_noop!(Regions::unlock(&region_id.into(), Some(2)), Error::<Test>::NotOwner);

		assert_ok!(Regions::unlock(&region_id.into(), Some(1)));
		assert_eq!(
			Regions::regions(&region_id).unwrap(),
			Region { owner: 1, locked: false, record: Record::Unavailable }
		);

		// The region can be transferred after unlocking.
		assert_ok!(
			<Regions as NonFungibleTransfer::<<Test as frame_system::Config>::AccountId>>::transfer(
				&region_id.into(),
				&2
			),
		);
		assert_eq!(Regions::owner(&region_id.into()), Some(2));
	});
}

#[test]
fn region_inspect_works() {
	new_test_ext().execute_with(|| {
		let region_id = RegionId { begin: 112830, core: 72, mask: CoreMask::complete() };
		assert!(Regions::record(&region_id.into()).is_none());

		assert_ok!(Regions::mint_into(&region_id.into(), &1));
		// the record is still not available so it will return `None`.
		assert!(Regions::record(&region_id.into()).is_none());

		let record: RegionRecordOf<Test> = RegionRecord { end: 123600, owner: 1, paid: None };
		assert_ok!(Regions::set_record(region_id, record.clone()));

		assert_eq!(Regions::record(&region_id.into()), Some(record));
	});
}

#[test]
fn utils_read_value_works() {
	new_test_ext().execute_with(|| {
		let mut values: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();
		values.insert("key1".as_bytes().to_vec(), Some("value1".as_bytes().to_vec()));
		values.insert("key2".as_bytes().to_vec(), None);

		assert_eq!(
			utils::read_value(&values, &"key1".as_bytes().to_vec()),
			Ok("value1".as_bytes().to_vec())
		);
		assert_eq!(
			utils::read_value(&values, &"key42".as_bytes().to_vec()),
			Err(IsmpCustomError::ValueNotFound.into())
		);
		assert_eq!(
			utils::read_value(&values, &"key2".as_bytes().to_vec()),
			Err(IsmpCustomError::EmptyValue.into())
		);
	});
}
