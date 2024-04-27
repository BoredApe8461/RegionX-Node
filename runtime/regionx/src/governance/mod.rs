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

use crate::*;

mod tracks;
use tracks::*;

use polkadot_runtime_common::prod_or_fast;

parameter_types! {
	pub const VoteLockingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1);
}

impl pallet_conviction_voting::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = RelayChainCurrency;
	type VoteLockingPeriod = VoteLockingPeriod;
	type MaxVotes = ConstU32<512>;
	type MaxTurnout = frame_support::traits::tokens::currency::ActiveIssuanceOf<
		RelayChainCurrency,
		Self::AccountId,
	>;
	type Polls = Referenda;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const SubmissionDeposit: Balance = 5 * KSM;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

impl pallet_referenda::Config for Runtime {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = RelayChainCurrency;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin = EnsureRoot<AccountId>; // TODO: Or Technical fellowship
	type KillOrigin = EnsureRoot<AccountId>; // TODO: Or Technical fellowship
	type Slash = (); // TODO: treasury
	type Votes = pallet_conviction_voting::VotesOf<Runtime>;
	type Tally = pallet_conviction_voting::TallyOf<Runtime>;
	type SubmissionDeposit = SubmissionDeposit;
	type MaxQueued = ConstU32<100>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = TracksInfo;
	type Preimages = Preimage;
}
