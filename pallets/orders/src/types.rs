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
use crate::BalanceOf;
use codec::{Decode, Encode, MaxEncodedLen};
pub use cumulus_primitives_core::ParaId;
use pallet_broker::{PartsOf57600, Timeslice};
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;

pub type RegionRecordOf<T> =
	pallet_broker::RegionRecord<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

/// Order identifier.
pub type OrderId = u32;

/// The region requirements of an order.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Requirements {
	/// The timeslice at which the Region begins.
	pub begin: Timeslice,
	/// The timeslice at which the Region ends.
	pub end: Timeslice,
	/// The minimum fraction of the core that the region should occupy.
	pub core_occupancy: PartsOf57600,
}

/// The information we store about a Coretime order.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Order<AccountId> {
	/// The `AccountId` that created the order.
	///
	/// In most cases this will probably be the sovereign account of the parachain.
	pub creator: AccountId,
	/// The para id to which Coretime will be allocated.
	pub para_id: ParaId,
	/// Region requirements of the order.
	pub requirements: Requirements,
}

pub trait FeeHandler<AccountId, Balance> {
	/// Function responsible for handling how we deal with fees.
	fn handle(who: &AccountId, fee: Balance) -> DispatchResult;
}
