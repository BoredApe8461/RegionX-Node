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

use super::*;

mod origins;
pub use origins::{pallet_custom_origins, Spender, WhitelistedCaller};
mod tracks;
pub use tracks::*;

use polkadot_runtime_common::prod_or_fast;

pub type GeneralCouncilInstance = pallet_collective::Instance1;
pub type TechnicalCommitteeInstance = pallet_collective::Instance2;

pub type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
pub type TechnicalCommitteeMembershipInstance = pallet_membership::Instance2;

type EnsureTwoThirdGeneralCouncil =
	pallet_collective::EnsureProportionAtLeast<AccountId, GeneralCouncilInstance, 2, 3>;
pub type EnsureTwoThirdTechnicalCommittee =
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 2, 3>;

pub type DelegatedReferendaInstance = pallet_referenda::Instance1;
pub type NativeReferendaInstance = pallet_referenda::Instance2;

pub type DelegatedConvictionVotingInstance = pallet_conviction_voting::Instance1;
pub type NativeConvictionVotingInstance = pallet_conviction_voting::Instance2;

parameter_types! {
	pub const VoteLockingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1);
}

impl pallet_conviction_voting::Config<DelegatedConvictionVotingInstance> for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = RelayChainCurrency;
	type VoteLockingPeriod = VoteLockingPeriod;
	type MaxVotes = ConstU32<512>;
	type MaxTurnout = frame_support::traits::tokens::currency::ActiveIssuanceOf<
		RelayChainCurrency,
		Self::AccountId,
	>;
	type Polls = DelegatedReferenda;
}

impl pallet_conviction_voting::Config<NativeConvictionVotingInstance> for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type VoteLockingPeriod = VoteLockingPeriod;
	type MaxVotes = ConstU32<512>;
	type MaxTurnout =
		frame_support::traits::tokens::currency::ActiveIssuanceOf<Balances, Self::AccountId>;
	type Polls = NativeReferenda;
}

parameter_types! {
	pub const AlarmInterval: BlockNumber = 1;
	pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
	pub const DelegatedReferendaSubmissionDeposit: Balance = KSM;
	pub const NativeReferendaSubmissionDeposit: Balance = 50 * REGX;
}

impl pallet_referenda::Config<DelegatedReferendaInstance> for Runtime {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = RelayChainCurrency;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin =
		EitherOfDiverse<EnsureTwoThirdTechnicalCommittee, EnsureTwoThirdGeneralCouncil>;
	type KillOrigin =
		EitherOfDiverse<EnsureTwoThirdTechnicalCommittee, EnsureTwoThirdGeneralCouncil>;
	type Slash = (); // TODO: treasury
	type Votes = pallet_conviction_voting::VotesOf<Runtime, DelegatedConvictionVotingInstance>;
	type Tally = pallet_conviction_voting::TallyOf<Runtime, DelegatedConvictionVotingInstance>;
	type SubmissionDeposit = DelegatedReferendaSubmissionDeposit;
	type MaxQueued = ConstU32<50>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = DelegatedReferendaTrackInfo;
	type Preimages = Preimage;
}

impl pallet_referenda::Config<NativeReferendaInstance> for Runtime {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type Scheduler = Scheduler;
	type Currency = Balances;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	type CancelOrigin = EnsureTwoThirdGeneralCouncil;
	type KillOrigin = EnsureTwoThirdGeneralCouncil;
	type Slash = Treasury;
	type Votes = pallet_conviction_voting::VotesOf<Runtime, NativeConvictionVotingInstance>;
	type Tally = pallet_conviction_voting::TallyOf<Runtime, NativeConvictionVotingInstance>;
	type SubmissionDeposit = NativeReferendaSubmissionDeposit;
	type MaxQueued = ConstU32<50>;
	type UndecidingTimeout = UndecidingTimeout;
	type AlarmInterval = AlarmInterval;
	type Tracks = NativeReferendaTrackInfo;
	type Preimages = Preimage;
}

parameter_types! {
	pub const MotionDuration: BlockNumber = 3 * DAYS;
	pub const MaxMembers: u32 = 30;
	pub const MaxProposals: u32 = 10;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<GeneralCouncilInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = MotionDuration;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type SetMembersOrigin = EnsureTwoThirdGeneralCouncil;
	type WeightInfo = ();
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = MotionDuration;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type SetMembersOrigin = EnsureTwoThirdGeneralCouncil;
	type WeightInfo = ();
	type MaxProposalWeight = MaxProposalWeight;
}

impl pallet_membership::Config<GeneralCouncilMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureTwoThirdGeneralCouncil;
	type RemoveOrigin = EnsureTwoThirdGeneralCouncil;
	type SwapOrigin = EnsureTwoThirdGeneralCouncil;
	type ResetOrigin = EnsureTwoThirdGeneralCouncil;
	type PrimeOrigin = EnsureTwoThirdGeneralCouncil;
	type MembershipInitialized = GeneralCouncil;
	type MembershipChanged = GeneralCouncil;
	type MaxMembers = MaxMembers;
	type WeightInfo = ();
}

impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureTwoThirdGeneralCouncil;
	type RemoveOrigin = EnsureTwoThirdGeneralCouncil;
	type SwapOrigin = EnsureTwoThirdGeneralCouncil;
	type ResetOrigin = EnsureTwoThirdGeneralCouncil;
	type PrimeOrigin = EnsureTwoThirdGeneralCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = MaxMembers;
	type WeightInfo = ();
}

impl pallet_custom_origins::Config for Runtime {}

impl pallet_whitelist::Config for Runtime {
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WhitelistOrigin = EnsureTwoThirdTechnicalCommittee;
	type DispatchWhitelistedOrigin = WhitelistedCaller;
	type Preimages = Preimage;
}
