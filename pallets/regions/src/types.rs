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
use pallet_broker::RegionRecord;
use scale_info::{prelude::format, TypeInfo};
use sp_core::H256;

pub type BalanceOf<T> =
	<<T as crate::Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type RegionRecordOf<T> = RegionRecord<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

/// The request status for getting the region record.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum Record<T: crate::Config> {
	/// An ISMP request was made to query the region record and we are now anticipating a response.
	///
	/// The hash represents the commitment of the ISMP get request.
	Pending(H256),
	/// An ISMP request was made, but we failed to get a response.
	Unavailable,
	/// Successfully retrieved the region record.
	Available(RegionRecordOf<T>),
}

impl<T: crate::Config> Record<T> {
	pub fn is_pending(&self) -> bool {
		matches!(self, Record::Pending(_))
	}

	pub fn is_unavailable(&self) -> bool {
		matches!(self, Record::Unavailable)
	}

	pub fn is_available(&self) -> bool {
		matches!(self, Record::Available(_))
	}

	pub fn get(&self) -> Option<RegionRecordOf<T>> {
		match self {
			Self::Available(r) => Some(r.clone()),
			_ => None,
		}
	}
}

/// Region that got cross-chain transferred from the Coretime chain.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Region<T: crate::Config> {
	/// Owner of the region.
	pub owner: T::AccountId,
	/// Indicates whether the region can be transferred or not.
	///
	/// Regions will be locked when listed on the market.
	pub locked: bool,
	/// The associated record of the region. If `None`, we still didn't receive a response
	/// for the ISMP GET request.
	///
	/// NOTE: The owner inside the record is the sovereign account of the parachain, so there
	/// isn't really a point to using it.
	pub record: Record<T>,
}

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
