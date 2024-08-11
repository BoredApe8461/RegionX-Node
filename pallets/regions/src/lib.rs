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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{alloc::collections::BTreeMap, Decode};
use core::{cmp::max, marker::PhantomData};
use frame_support::{pallet_prelude::Weight, traits::nonfungible::Mutate as NftMutate, PalletId};
use ismp::{
	consensus::StateMachineId,
	dispatcher::{DispatchGet, DispatchRequest, FeeMetadata, IsmpDispatcher},
	error::Error as IsmpError,
	host::StateMachine,
	module::IsmpModule,
	router::{Post, Request, Response, Timeout},
};
use ismp_parachain::PARACHAIN_CONSENSUS_ID;
pub use pallet::*;
use pallet_broker::{RegionId, Timeslice};
use pallet_ismp::{weights::IsmpModuleWeight, ModuleId};
use primitives::StateMachineHeightProvider;
use region_primitives::{Record, Region, RegionFactory};
use scale_info::prelude::{format, vec, vec::Vec};
use sp_core::H256;
use sp_runtime::{
	traits::{BlockNumberProvider, Zero},
	SaturatedConversion,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod ismp_mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod nonfungible_impls;

mod types;
use types::*;

pub mod primitives;

pub mod weights;
pub use weights::WeightInfo;

const LOG_TARGET: &str = "runtime::regions";

/// Constant Pallet ID
pub const PALLET_ID: ModuleId = ModuleId::Pallet(PalletId(*b"regionsp"));

// Custom transaction error codes
const REGION_NOT_FOUND: u8 = 1;
const REGION_NOT_UNAVAILABLE: u8 = 2;

/// Relay chain block number.
pub type RCBlockNumberOf<T> =
	<<T as crate::Config>::RCBlockNumberProvider as BlockNumberProvider>::BlockNumber;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::fungible::Mutate};
	use frame_system::pallet_prelude::*;

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency implementation
		//
		// NOTE: Isn't used since we don't have a reward mechanism for ISMP relayers.
		type Currency: Mutate<Self::AccountId>;

		/// The Coretime chain from which we read region state.
		type CoretimeChain: Get<StateMachine>;

		/// Type for getting the current relay chain block.
		///
		/// This is used for determining the current timeslice.
		type RCBlockNumberProvider: BlockNumberProvider;

		/// Number of Relay-chain blocks per timeslice.
		#[pallet::constant]
		type TimeslicePeriod: Get<RCBlockNumberOf<Self>>;

		/// The ISMP dispatcher.
		type IsmpDispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = BalanceOf<Self>>
			+ Default;

		/// Used for getting the height of the Coretime chain.
		type StateMachineHeightProvider: StateMachineHeightProvider;

		/// Number of seconds before a GET request times out.
		type Timeout: Get<u64>;

		/// The priority of unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// Weight Info
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Regions that got cross-chain transferred to the RegionX parachain.
	#[pallet::storage]
	#[pallet::getter(fn regions)]
	pub type Regions<T> = StorageMap<_, Blake2_128Concat, RegionId, RegionOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Ownership of a Region has been transferred.
		Transferred {
			/// The Region which has been transferred.
			region_id: RegionId,
			/// The old owner of the Region.
			old_owner: T::AccountId,
			/// The new owner of the Region.
			owner: T::AccountId,
		},
		/// Region record of the given id has been set.
		RecordSet {
			/// The id of the region that the record has been set.
			region_id: RegionId,
		},
		/// An ISMP GET request was made to get the region record.
		RegionRecordRequested {
			/// The id of the region that the request was made for.
			region_id: RegionId,
			/// The ismp get request commitment.
			request_commitment: H256,
		},
		/// A region was minted via a cross chain transfer.
		RegionMinted {
			/// id of the minted region
			region_id: RegionId,
			/// address of the minter
			by: T::AccountId,
		},
		/// A region was burnt.
		RegionBurnt {
			/// id of the burnt region
			region_id: RegionId,
		},
		/// A region was locked.
		RegionLocked {
			/// id of the locked region
			region_id: RegionId,
		},
		/// A region was unlocked.
		RegionUnlocked {
			/// id of the unlocked region
			region_id: RegionId,
		},
		/// An expired region was dropped.
		RegionDropped {
			/// id of the dropped region
			region_id: RegionId,
			/// the account that dropped the region
			who: T::AccountId,
		},
		/// Request for a region record timed out.
		RequestTimedOut { region_id: RegionId },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T> {
		/// The given region identity is not known.
		UnknownRegion,
		/// The owner of the region is not the origin.
		NotOwner,
		/// The region record of the region is already set.
		RegionRecordAlreadySet,
		/// An error occured when attempting to dispatch an ISMP GET request.
		IsmpDispatchError,
		/// The record must be unavailable to be able to re-request it.
		NotUnavailable,
		/// The region record is not available.
		NotAvailable,
		/// The given region id is not valid.
		InvalidRegionId,
		/// Failed to get the latest height of the Coretime chain.
		LatestHeightInaccessible,
		/// Locked regions cannot be transferred.
		RegionLocked,
		/// Region isn't locked.
		RegionNotLocked,
		/// Region is not expired.
		RegionNotExpired,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			region_id: RegionId,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer(region_id, Some(who), new_owner)?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::request_region_record())]
		pub fn request_region_record(origin: OriginFor<T>, region_id: RegionId) -> DispatchResult {
			ensure_none(origin)?;

			let region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;
			ensure!(region.record.is_unavailable(), Error::<T>::NotUnavailable);

			// Even though we don't know if this was requested by the region owner, we can use it
			// since there is no fee charged.
			let commitment = Self::do_request_region_record(region_id, region.owner.clone())?;
			Regions::<T>::insert(
				region_id,
				Region { owner: region.owner, locked: false, record: Record::Pending(commitment) },
			);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::drop_region())]
		pub fn drop_region(origin: OriginFor<T>, region_id: RegionId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;

			ensure!(region.record.is_available(), Error::<T>::NotAvailable);

			if let Record::Available(record) = region.record {
				// Cannot drop a region that is not expired yet.

				// Allowing region removal 1 timeslice before it truly expires makes writing
				// benchmarks much easier. With this we can set the start and end to 0 and be able
				// to drop the region without having to modify the current timeslice.
				let current_timeslice = Self::current_timeslice();
				#[cfg(feature = "runtime-benchmarks")]
				ensure!(record.end <= current_timeslice, Error::<T>::RegionNotExpired);
				#[cfg(not(feature = "runtime-benchmarks"))]
				ensure!(record.end < current_timeslice, Error::<T>::RegionNotExpired);

				Regions::<T>::remove(region_id);

				Self::deposit_event(Event::RegionDropped { region_id, who });
				Ok(())
			} else {
				Err(Error::<T>::NotAvailable.into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn do_transfer(
			region_id: RegionId,
			maybe_check_owner: Option<T::AccountId>,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let mut region = Regions::<T>::get(region_id).ok_or(Error::<T>::UnknownRegion)?;

			ensure!(!region.locked, Error::<T>::RegionLocked);
			if let Some(check_owner) = maybe_check_owner {
				ensure!(check_owner == region.owner, Error::<T>::NotOwner);
			}

			let old_owner = region.owner;
			region.owner = new_owner;
			Regions::<T>::insert(region_id, &region);

			Self::deposit_event(Event::Transferred { region_id, old_owner, owner: region.owner });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn set_record(region_id: RegionId, record: RegionRecordOf<T>) -> DispatchResult {
			let Some(mut region) = Regions::<T>::get(region_id) else {
				return Err(Error::<T>::UnknownRegion.into());
			};
			ensure!(!region.record.is_available(), Error::<T>::RegionRecordAlreadySet);

			region.record = Record::Available(record);
			Regions::<T>::insert(region_id, region);

			Self::deposit_event(Event::RecordSet { region_id });

			Ok(())
		}

		pub(crate) fn do_request_region_record(
			region_id: RegionId,
			who: <T as frame_system::Config>::AccountId,
		) -> Result<H256, DispatchError> {
			let key = Self::region_storage_key(region_id)?;

			let coretime_chain_height =
				T::StateMachineHeightProvider::latest_state_machine_height(StateMachineId {
					state_id: T::CoretimeChain::get(),
					consensus_state_id: PARACHAIN_CONSENSUS_ID,
				})
				.ok_or(Error::<T>::LatestHeightInaccessible)?;

			// TODO: should requests be coupled in the future?
			let get = DispatchGet {
				dest: T::CoretimeChain::get(),
				from: PALLET_ID.to_bytes(),
				keys: vec![key],
				height: coretime_chain_height,
				timeout: T::Timeout::get(),
			};

			let dispatcher = T::IsmpDispatcher::default();

			let commitment = dispatcher
				.dispatch_request(
					DispatchRequest::Get(get),
					FeeMetadata { payer: who.clone(), fee: Zero::zero() },
				)
				.map_err(|_| Error::<T>::IsmpDispatchError)?;

			Self::deposit_event(Event::RegionRecordRequested {
				region_id,
				request_commitment: commitment,
			});

			Ok(commitment)
		}

		pub(crate) fn region_storage_key(region_id: RegionId) -> Result<Vec<u8>, DispatchError> {
			let pallet_hash = sp_io::hashing::twox_128("Broker".as_bytes());
			let storage_hash = sp_io::hashing::twox_128("Regions".as_bytes());
			let region_id_hash = sp_io::hashing::blake2_128(&region_id.encode());

			// We know a region id is 128 bits.
			let region_id_encoded: [u8; 16] =
				region_id.encode().try_into().map_err(|_| Error::<T>::InvalidRegionId)?;

			// pallet_hash + storage_hash + blake2_128(region_id) + scale encoded region_id
			let key = [pallet_hash, storage_hash, region_id_hash, region_id_encoded].concat();

			Ok(key)
		}

		pub(crate) fn current_timeslice() -> Timeslice {
			let latest_rc_block = T::RCBlockNumberProvider::current_block_number();
			let timeslice_period = T::TimeslicePeriod::get();
			(latest_rc_block / timeslice_period).saturated_into()
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let region_id = match call {
				Call::request_region_record { region_id } => region_id,
				_ => return InvalidTransaction::Call.into(),
			};

			let Some(region) = Regions::<T>::get(region_id) else {
				return InvalidTransaction::Custom(REGION_NOT_FOUND).into()
			};

			if !region.record.is_unavailable() {
				return InvalidTransaction::Custom(REGION_NOT_UNAVAILABLE).into()
			}

			ValidTransaction::with_tag_prefix("RecordRequest")
				.priority(T::UnsignedPriority::get())
				.and_provides(region_id)
				.propagate(true)
				.build()
		}

		fn pre_dispatch(_call: &Self::Call) -> Result<(), TransactionValidityError> {
			// Given that the `request_region_record` function contains checks there is no need to
			// call `validate_unsigned` again.
			Ok(())
		}
	}
}

/// Module callback for the pallet
pub struct IsmpModuleCallback<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> Default for IsmpModuleCallback<T> {
	fn default() -> Self {
		Self(core::marker::PhantomData)
	}
}

impl<T: Config> IsmpModule for IsmpModuleCallback<T> {
	fn on_accept(&self, _request: Post) -> Result<(), IsmpError> {
		Err(IsmpCustomError::NotSupported.into())
	}

	fn on_response(&self, response: Response) -> Result<(), IsmpError> {
		match response {
			Response::Post(_) => Err(IsmpCustomError::NotSupported)?,
			Response::Get(res) => {
				res.get.keys.iter().try_for_each(|key| -> Result<(), IsmpError> {
					let value = utils::read_value(&res.values, key)?;

					// The last 16 bytes represent the region id.
					let mut region_id_encoded = &key[max(0, key.len() as isize - 16) as usize..];

					let region_id = RegionId::decode(&mut region_id_encoded)
						.map_err(|_| IsmpCustomError::KeyDecodeFailed)?;

					let record = RegionRecordOf::<T>::decode(&mut value.as_slice())
						.map_err(|_| IsmpCustomError::ResponseDecodeFailed)?;

					crate::Pallet::<T>::set_record(region_id, record)
						.map_err(|e| IsmpError::Custom(format!("{:?}", e)))?;

					Ok(())
				})?;
			},
		}

		Ok(())
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), IsmpError> {
		match timeout {
			Timeout::Request(Request::Get(get)) => get.keys.iter().try_for_each(|key| {
				// The last 16 bytes represent the region id.
				let mut region_id_encoded = &key[max(0, key.len() as isize - 16) as usize..];

				let region_id = RegionId::decode(&mut region_id_encoded)
					.map_err(|_| IsmpCustomError::KeyDecodeFailed)?;

				let Some(mut region) = Regions::<T>::get(region_id) else {
					return Err(IsmpCustomError::RegionNotFound.into());
				};

				region.record = Record::Unavailable;
				Regions::<T>::insert(region_id, region);

				crate::Pallet::<T>::deposit_event(Event::RequestTimedOut { region_id });
				Ok(())
			}),
			Timeout::Request(Request::Post(_)) => Ok(()),
			Timeout::Response(_) => Ok(()),
		}
	}
}

pub struct IsmpRegionsModuleWeight<T: crate::Config> {
	marker: PhantomData<T>,
}

impl<T: crate::Config> IsmpModuleWeight for IsmpRegionsModuleWeight<T> {
	fn on_accept(&self, _request: &Post) -> Weight {
		T::WeightInfo::on_accept()
	}

	fn on_response(&self, _response: &Response) -> Weight {
		T::WeightInfo::on_response()
	}

	fn on_timeout(&self, _timeout: &Timeout) -> Weight {
		T::WeightInfo::on_timeout()
	}
}

impl<T: crate::Config> Default for IsmpRegionsModuleWeight<T> {
	fn default() -> Self {
		IsmpRegionsModuleWeight { marker: PhantomData }
	}
}

impl<T: crate::Config> RegionFactory<T::AccountId, RegionRecordOf<T>> for Pallet<T> {
	fn create_region(
		region_id: RegionId,
		record: RegionRecordOf<T>,
		owner: <T as frame_system::Config>::AccountId,
	) -> sp_runtime::DispatchResult {
		crate::Pallet::<T>::mint_into(&region_id.into(), &owner)?;
		crate::Pallet::<T>::set_record(region_id, record.clone())?;
		Ok(())
	}
}

mod utils {
	use super::{BTreeMap, IsmpCustomError, IsmpError};
	#[cfg(not(feature = "std"))]
	use scale_info::prelude::vec::Vec;

	pub fn read_value(
		values: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
		key: &Vec<u8>,
	) -> Result<Vec<u8>, IsmpError> {
		let result = values
			.get(key)
			.ok_or(IsmpCustomError::ValueNotFound)?
			.clone()
			.ok_or(IsmpCustomError::EmptyValue)?;

		Ok(result)
	}
}
