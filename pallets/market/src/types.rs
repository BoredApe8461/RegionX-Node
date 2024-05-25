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
use crate::{BalanceOf, RegionId};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;

pub type RegionRecordOf<T> =
	pallet_broker::RegionRecord<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

/// The information we store about a region that got listed on sale.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Listing<AccountId, Balance> {
	/// The `AccountId` selling the region.
	pub seller: AccountId,
	/// The price per a single timeslice.
	pub timeslice_price: Balance,
	/// The `AccountId` receiving the payment from the sale.
	///
	/// This will usually be the seller account.
	pub sale_recipient: AccountId,
}

/// Trait for creating regions. Used for benchmarking.
pub trait RegionFactory<T: crate::Config> {
	fn create_region(
		region_id: RegionId,
		record: RegionRecordOf<T>,
		owner: T::AccountId,
	) -> DispatchResult;
}
