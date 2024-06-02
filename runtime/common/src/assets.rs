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

use sp_core::ConstU32;

/// Asset identifier.
pub type AssetId = u32;

pub const REGX_ASSET_ID: AssetId = 0;
pub const RELAY_CHAIN_ASSET_ID: AssetId = 1;

pub type AssetsStringLimit = ConstU32<50>;
