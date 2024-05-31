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

use crate::{RegionId, RegionRecordOf};
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	traits::{nonfungible::Mutate, Everything},
};
use ismp::{
	consensus::StateMachineId,
	dispatcher::{DispatchRequest, FeeMetadata, IsmpDispatcher},
	error::Error,
	host::StateMachine,
	router::PostResponse,
};
use ismp_testsuite::mocks::Host;
use pallet_regions::primitives::StateMachineHeightProvider;
use sp_core::{ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	BuildStorage, DispatchResult,
};
use std::sync::Arc;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		Regions: pallet_regions::{Pallet, Call, Storage, Event<T>},
		Market: crate::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeTask = RuntimeTask;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxHolds = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

pub struct MockStateMachineHeightProvider;
impl StateMachineHeightProvider for MockStateMachineHeightProvider {
	fn latest_state_machine_height(_id: StateMachineId) -> Option<u64> {
		Some(0)
	}
}

pub struct MockDispatcher<T: crate::Config>(pub Arc<Host>, PhantomData<T>);

impl<T: crate::Config> Default for MockDispatcher<T> {
	fn default() -> Self {
		MockDispatcher(Default::default(), PhantomData::<T>::default())
	}
}
impl<T: crate::Config> IsmpDispatcher for MockDispatcher<T> {
	type Account = u64;
	type Balance = u64;

	fn dispatch_request(
		&self,
		_request: DispatchRequest,
		_fee: FeeMetadata<Self::Account, Self::Balance>,
	) -> Result<H256, Error> {
		Ok(Default::default())
	}

	fn dispatch_response(
		&self,
		_response: PostResponse,
		_fee: FeeMetadata<Self::Account, Self::Balance>,
	) -> Result<H256, Error> {
		Ok(Default::default())
	}
}

parameter_types! {
	pub const CoretimeChain: StateMachine = StateMachine::Kusama(1005); // coretime-kusama
}

impl pallet_regions::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CoretimeChain = CoretimeChain;
	type IsmpDispatcher = MockDispatcher<Self>;
	type StateMachineHeightProvider = MockStateMachineHeightProvider;
	type Timeout = ConstU64<1000>;
	type WeightInfo = ();
}

parameter_types! {
	pub static RelayBlockNumber: u64 = 0;
}

pub struct RelayBlockNumberProvider;
impl BlockNumberProvider for RelayBlockNumberProvider {
	type BlockNumber = u64;
	fn current_block_number() -> Self::BlockNumber {
		RelayBlockNumber::get()
	}
}

pub struct RegionFactory;
impl crate::RegionFactory<Test> for RegionFactory {
	fn create_region(
		region_id: RegionId,
		record: RegionRecordOf<Test>,
		owner: <Test as frame_system::Config>::AccountId,
	) -> DispatchResult {
		Regions::mint_into(&region_id.into(), &owner)?;
		Regions::set_record(region_id, record.clone())?;
		Ok(())
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Regions = Regions;
	type RCBlockNumberProvider = RelayBlockNumberProvider;
	type TimeslicePeriod = ConstU64<80>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = RegionFactory;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10_000_000), (2, 10_000_000), (3, 10_000_000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
