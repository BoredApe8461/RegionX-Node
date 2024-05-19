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

use frame_support::{pallet_prelude::DispatchResult, traits::nonfungible::Inspect};

/// Nonfungible implementation which can be locked.
pub trait LockableNonFungible<AccountId>: Inspect<AccountId> {
	/// Lock an item. This will restrict transfers.
	fn lock(item: &Self::ItemId) -> DispatchResult;

	/// Unlock an item.
	///
	/// Should fail if the item wasn't previously locked. This will make the item transferable
	/// again.
	fn unlock(item: &Self::ItemId) -> DispatchResult;
}
