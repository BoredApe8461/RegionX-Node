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
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::Parameter;
use pallet_broker::RegionRecord;
use scale_info::TypeInfo;
use sp_core::H256;

pub type RegionRecordOf<AccountId, Balance> = RegionRecord<AccountId, Balance>;

/// The request status for getting the region record.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Record<AccountId: Clone, Balance: Clone> {
	/// An ISMP request was made to query the region record and we are now anticipating a response.
	///
	/// The hash represents the commitment of the ISMP get request.
	Pending(H256),
	/// An ISMP request was made, but we failed to get a response.
	Unavailable,
	/// Successfully retrieved the region record.
	Available(RegionRecordOf<AccountId, Balance>),
}

impl<AccountId: Clone, Balance: Clone> Record<AccountId, Balance> {
	pub fn is_pending(&self) -> bool {
		matches!(self, Record::Pending(_))
	}

	pub fn is_unavailable(&self) -> bool {
		matches!(self, Record::Unavailable)
	}

	pub fn is_available(&self) -> bool {
		matches!(self, Record::Available(_))
	}

	pub fn get(&self) -> Option<RegionRecordOf<AccountId, Balance>> {
		match self {
			Self::Available(r) => Some(r.clone()),
			_ => None,
		}
	}
}

/// Region that got cross-chain transferred from the Coretime chain.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Region<AccountId: Clone, Balance: Clone> {
	/// Owner of the region.
	pub owner: AccountId,
	/// Indicates whether the region can be transferred or not.
	///
	/// Regions will be locked when listed on the market.
	pub locked: bool,
	/// The associated record of the region. If `None`, we still didn't receive a response
	/// for the ISMP GET request.
	///
	/// NOTE: The owner inside the record is the sovereign account of the parachain, so there
	/// isn't really a point to using it.
	pub record: Record<AccountId, Balance>,
}

pub trait RegionInspect<AccountId: Clone, Balance: Clone> {
	/// Type for identifying the region.
	type ItemId: Parameter;

	/// Get the associated record of a region.
	///
	/// If `None` the region record was not found.
	fn record(region_id: &Self::ItemId) -> Option<RegionRecordOf<AccountId, Balance>>;
}
