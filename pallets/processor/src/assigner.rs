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

use crate::LOG_TARGET;
use core::marker::PhantomData;
use frame_support::weights::WeightToFee;
use order_primitives::ParaId;
use pallet_broker::RegionId;
#[cfg(not(feature = "std"))]
use scale_info::prelude::{vec, vec::Vec};
use sp_runtime::{traits::Get, DispatchResult};
use xcm::latest::prelude::*;

/// Type which encodes the region assignment call.
pub trait AssignmentCallEncoder {
	fn encode_assignment_call(region_id: RegionId, para_id: ParaId) -> Vec<u8>;
}

/// Type assigning the region to the specified task.
pub trait RegionAssigner {
	// Assigns the region to the specified task.
	fn assign(region_id: RegionId, para_id: ParaId) -> DispatchResult;
}

/// A type that implements the RegionAssigner trait and assigns a region to a task by sending the
/// appropriate XCM message to the Coretime chain.
pub struct XcmRegionAssigner<T: crate::Config>(PhantomData<T>);
impl<T: crate::Config + pallet_xcm::Config> RegionAssigner for XcmRegionAssigner<T> {
	fn assign(region_id: RegionId, para_id: ParaId) -> DispatchResult {
		let assignment_call = T::AssignmentCallEncoder::encode_assignment_call(region_id, para_id);

		// NOTE: the weight is runtime dependant, however we are rounding up a lot so it should
		// always be sufficient.
		//
		// `ref_time` = `31_500`, we will round up to: `100_000_000`.
		// `proof_size` = `4700`, we will round up to: `10_000`.
		let call_weight = Weight::from_parts(100_000_000, 10_000);
		let fee = T::WeightToFee::weight_to_fee(&call_weight);

		let message = Xcm(vec![
			Instruction::WithdrawAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(fee.into()) }
					.into(),
			),
			Instruction::BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(fee.into()),
				},
				weight_limit: Unlimited,
			},
			Instruction::Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: call_weight,
				call: assignment_call.into(),
			},
		]);

		match pallet_xcm::Pallet::<T>::send_xcm(
			Here,
			<T as crate::Config>::CoretimeChain::get(),
			message,
		) {
			Ok(_) => log::info!(
				target: LOG_TARGET,
				"Region assignment sent successfully"
			),
			Err(e) => log::error!(
				target: LOG_TARGET,
				"Failed to send region assignment: {:?}",
				e
			),
		}

		Ok(())
	}
}
