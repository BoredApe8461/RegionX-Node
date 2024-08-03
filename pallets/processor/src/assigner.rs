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
use sp_runtime::{traits::Get, DispatchError, DispatchResult, SaturatedConversion, Saturating};
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertLocation;

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
pub struct XcmRegionAssigner<
	T: crate::Config,
	SovereignAccountOf: ConvertLocation<T::AccountId>,
	OwnParaId: Get<u32>,
	FeeBuffer: Get<<T as crate::Config>::Balance>,
>(PhantomData<(T, SovereignAccountOf, OwnParaId, FeeBuffer)>);

impl<
		T: crate::Config + pallet_xcm::Config,
		SovereignAccountOf: ConvertLocation<T::AccountId>,
		OwnParaId: Get<u32>,
		FeeBuffer: Get<<T as crate::Config>::Balance>,
	> RegionAssigner for XcmRegionAssigner<T, SovereignAccountOf, OwnParaId, FeeBuffer>
where
	T::AccountId: Into<[u8; 32]>,
{
	fn assign(region_id: RegionId, para_id: ParaId) -> DispatchResult {
		let assignment_call = T::AssignmentCallEncoder::encode_assignment_call(region_id, para_id);

		// NOTE: the weight is runtime dependant, however we are rounding up a lot so it should
		// always be sufficient.
		//
		// After some testing, the conclusion is that the following weight limit is sufficient:
		let call_weight = Weight::from_parts(500_000_000, 10_000);
		let fee = T::WeightToFee::weight_to_fee(&call_weight)
			.saturating_add(FeeBuffer::get().saturated_into());

		let _sovereign_account = SovereignAccountOf::convert_location(&MultiLocation::new(
			1,
			X1(Parachain(OwnParaId::get())),
		))
		.ok_or(DispatchError::Other("Couldn't get the sovereign account"))?;

		let message = Xcm(vec![
			WithdrawAsset(
				MultiAsset { id: Concrete(MultiLocation::parent()), fun: Fungible(fee.into()) }
					.into(),
			),
			BuyExecution {
				fees: MultiAsset {
					id: Concrete(MultiLocation::parent()),
					fun: Fungible(fee.into()),
				},
				weight_limit: Unlimited,
			},
			Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: call_weight,
				call: assignment_call.into(),
			},
			RefundSurplus,
			/* TODO: https://github.com/RegionX-Labs/RegionX-Node/issues/220
			DepositAsset {
				assets: All.into(),
				beneficiary: AccountId32 { id: sovereign_account.into(), network: None }.into(),
			},
			*/
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
