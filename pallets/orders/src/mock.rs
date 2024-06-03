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

use crate::{FeeHandler, ParaId};
use frame_support::{
	pallet_prelude::*,
	parameter_types,
	traits::{fungible::Mutate, tokens::Preservation, Everything},
};
use sp_core::{ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	AccountId32, BuildStorage,
};
use xcm::opaque::lts::NetworkId;

use xcm_builder::{
	AccountId32Aliases, ChildParachainConvertsVia, DescribeAllTerminal, HashedDescription,
};

type AccountId = AccountId32;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: AccountId = AccountId::new([0u8; 32]);
pub const BOB: AccountId = AccountId::new([1u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([2u8; 32]);
pub const TREASURY: AccountId = AccountId::new([3u8; 32]);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		Orders: crate::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const AnyNetwork: Option<NetworkId> = None;
}

pub type SovereignAccountOf = (
	ChildParachainConvertsVia<ParaId, AccountId>,
	AccountId32Aliases<AnyNetwork, AccountId>,
	HashedDescription<AccountId, DescribeAllTerminal>,
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

pub struct OrderCreationFeeHandler;
impl FeeHandler<AccountId, u64> for OrderCreationFeeHandler {
	fn handle(who: &AccountId, fee: u64) -> DispatchResult {
		<Test as crate::Config>::Currency::transfer(who, &TREASURY, fee, Preservation::Preserve)?;
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

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SovereignAccountOf = SovereignAccountOf;
	type OrderCreationCost = ConstU64<100>;
	type MinimumContribution = ConstU64<50>;
	type RCBlockNumberProvider = RelayBlockNumberProvider;
	type TimeslicePeriod = ConstU64<80>;
	type OrderCreationFeeHandler = OrderCreationFeeHandler;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(endowed_accounts: Vec<(AccountId32, u64)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: endowed_accounts }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
