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

use core::cell::RefCell;
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	traits::{fungible::Mutate, tokens::Preservation, Everything},
	weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
use ismp::{
	consensus::StateMachineId,
	dispatcher::{DispatchRequest, FeeMetadata, IsmpDispatcher},
	error::Error,
	host::StateMachine,
	router::PostResponse,
};
use ismp_testsuite::mocks::Host;
use order_primitives::{OrderId, ParaId};
use pallet_broker::RegionId;
use pallet_orders::FeeHandler;
use pallet_regions::primitives::StateMachineHeightProvider;
use smallvec::smallvec;
use sp_core::{ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, BlockNumberProvider, Convert, IdentityLookup},
	BuildStorage, DispatchResult, Perbill,
};
use std::sync::Arc;
use xcm::latest::prelude::*;

type AccountId = u64;
type Block = frame_system::mocking::MockBlock<Test>;

pub const TREASURY: AccountId = 42;

pub const MILLIUNIT: u64 = 1_000_000_000;
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = u64;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLIUNIT / 10;
		let q = 100 * u64::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		Orders: pallet_orders::{Pallet, Call, Storage, Event<T>},
		Regions: pallet_regions::{Pallet, Call, Storage, Event<T>},
		Processor: crate::{Pallet, Call, Storage, Event<T>},
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
	type AccountId = AccountId;
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
	pub const CoretimeChainStateMachine: StateMachine = StateMachine::Kusama(1005); // coretime-kusama
	pub const RegionsUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

impl pallet_regions::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CoretimeChain = CoretimeChainStateMachine;
	type IsmpDispatcher = MockDispatcher<Self>;
	type StateMachineHeightProvider = MockStateMachineHeightProvider;
	type Timeout = ConstU64<1000>;
	type UnsignedPriority = RegionsUnsignedPriority;
	type WeightInfo = ();
}

pub struct OrderCreationFeeHandler;
impl FeeHandler<AccountId, u64> for OrderCreationFeeHandler {
	fn handle(who: &AccountId, fee: u64) -> DispatchResult {
		<Test as pallet_orders::Config>::Currency::transfer(
			who,
			&TREASURY,
			fee,
			Preservation::Preserve,
		)?;
		Ok(())
	}
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

pub struct OrderToAccountId;
impl Convert<OrderId, AccountId> for OrderToAccountId {
	fn convert(order: OrderId) -> AccountId {
		1000u64 + order as u64
	}
}

impl pallet_orders::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type OrderCreationCost = ConstU64<100>;
	type MinimumContribution = ConstU64<50>;
	type RCBlockNumberProvider = RelayBlockNumberProvider;
	type OrderToAccountId = OrderToAccountId;
	type TimeslicePeriod = ConstU64<80>;
	type OrderCreationFeeHandler = OrderCreationFeeHandler;
	type WeightInfo = ();
}

parameter_types! {
	// The location of the Coretime parachain.
	pub const CoretimeChain: MultiLocation = MultiLocation { parents: 1, interior: X1(Parachain(1005)) };
}

#[derive(Encode, Decode)]
enum CoretimeRuntimeCalls {
	#[codec(index = 50)]
	Broker(BrokerPalletCalls),
}

/// Broker pallet calls. We don't define all of them, only the ones we use.
#[derive(Encode, Decode)]
enum BrokerPalletCalls {
	#[codec(index = 10)]
	Assign(RegionId, ParaId),
}

pub struct AssignmentCallEncoder;
impl crate::AssignmentCallEncoder for AssignmentCallEncoder {
	fn encode_assignment_call(region_id: RegionId, para_id: ParaId) -> Vec<u8> {
		CoretimeRuntimeCalls::Broker(BrokerPalletCalls::Assign(region_id, para_id)).encode()
	}
}

thread_local! {
	pub static ASSIGNMENTS: RefCell<Vec<(RegionId, ParaId)>> = Default::default();
}

pub fn assignments() -> Vec<(RegionId, ParaId)> {
	ASSIGNMENTS.with(|assignments| assignments.borrow().clone())
}

pub struct DummyRegionAssigner;
impl crate::RegionAssigner for DummyRegionAssigner {
	fn assign(region_id: RegionId, para_id: ParaId) -> DispatchResult {
		ASSIGNMENTS.with(|assignments| {
			let mut assignments = assignments.borrow_mut();
			assignments.push((region_id, para_id));
		});
		Ok(())
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Balance = u64;
	type Orders = Orders;
	type OrderToAccountId = OrderToAccountId;
	type Regions = Regions;
	type AssignmentCallEncoder = AssignmentCallEncoder;
	type RegionAssigner = DummyRegionAssigner;
	type CoretimeChain = CoretimeChain;
	type WeightToFee = WeightToFee;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(endowed_accounts: Vec<(u64, u64)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: endowed_accounts }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
