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
pub use cumulus_primitives_core::ParaId;
use sp_runtime::DispatchResult;

pub type RegionRecordOf<T> =
	pallet_broker::RegionRecord<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

pub trait FeeHandler<AccountId, Balance> {
	/// Function responsible for handling how we deal with fees.
	fn handle(who: &AccountId, fee: Balance) -> DispatchResult;
}
