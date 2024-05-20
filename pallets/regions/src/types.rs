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

use crate::IsmpError;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::fungible::Inspect;
use scale_info::{prelude::format, TypeInfo};

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type RegionRecordOf<T> =
	region_primitives::RegionRecordOf<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

/// ISMP errors specific to the RegionX project.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum IsmpCustomError {
	/// Operation not supported.
	NotSupported,
	/// Failed to decode ismp request key.
	KeyDecodeFailed,
	/// Failed to decode ismp response.
	ResponseDecodeFailed,
	/// Couldn't find the region with the associated `RegionId`
	RegionNotFound,
	/// Couldn't find the corresponding value of a key in the ISMP result.
	ValueNotFound,
	/// Found the corresponding value, but it is `None`.
	EmptyValue,
}

impl core::fmt::Display for IsmpCustomError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::NotSupported => write!(f, "NotSupported"),
			Self::KeyDecodeFailed => write!(f, "KeyDecodeFailed"),
			Self::ResponseDecodeFailed => write!(f, "ResponseDecodeFailed"),
			Self::RegionNotFound => write!(f, "RegionNotFound"),
			Self::ValueNotFound => write!(f, "ValueNotFound"),
			Self::EmptyValue => write!(f, "EmptyValue"),
		}
	}
}

impl From<IsmpCustomError> for IsmpError {
	fn from(error: IsmpCustomError) -> Self {
		IsmpError::Custom(format!("{}", error))
	}
}
