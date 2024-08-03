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

use crate::{
	AccountId, AssetId, AssetRegistry, Authorship, Balance, Balances, PalletCurrency, PotId,
	RegionXTreasuryAccount, Runtime, RuntimeCall, Tokens, Treasury,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{
	fungibles, tokens::ConversionToAssetBalance, Defensive, ExistenceRequirement, Imbalance,
	InstanceFilter, OnUnbalanced,
};
use order_primitives::ParaId;
use orml_asset_registry::DefaultAssetMetadata;
use orml_traits::{asset_registry::AssetProcessor, GetByKey};
use pallet_asset_tx_payment::HandleCredit;
use pallet_broker::{Finality, RegionId};
use pallet_processor::assigner::AssignmentCallEncoder as AssignmentCallEncoderT;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedDiv},
	ArithmeticError, DispatchError, DispatchResult, FixedPointNumber, FixedU128, RuntimeDebug,
	TokenError,
};

#[derive(
	Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
pub struct CustomAssetProcessor;

impl AssetProcessor<AssetId, DefaultAssetMetadata<Runtime>> for CustomAssetProcessor {
	fn pre_register(
		id: Option<AssetId>,
		metadata: DefaultAssetMetadata<Runtime>,
	) -> Result<(AssetId, DefaultAssetMetadata<Runtime>), DispatchError> {
		match id {
			Some(id) => Ok((id, metadata)),
			None => Err(DispatchError::Other("asset-registry: AssetId is required")),
		}
	}

	fn post_register(
		_id: AssetId,
		_metadata: DefaultAssetMetadata<Runtime>,
	) -> Result<(), DispatchError> {
		Ok(())
	}
}

/// A `HandleCredit` implementation that naively transfers the fees to the block author.
/// Will drop and burn the assets in case the transfer fails.
pub struct TokensToBlockAuthor;
impl HandleCredit<AccountId, Tokens> for TokensToBlockAuthor {
	fn handle_credit(credit: fungibles::Credit<AccountId, Tokens>) {
		use frame_support::traits::fungibles::Balanced;
		if let Some(author) = Authorship::author() {
			// In case of error: Will drop the result triggering the `OnDrop` of the imbalance.
			let _ = Tokens::resolve(&author, credit).defensive();
		}
	}
}

pub struct TokenToNativeConverter;
impl ConversionToAssetBalance<Balance, AssetId, Balance> for TokenToNativeConverter {
	type Error = DispatchError;

	fn to_asset_balance(balance: Balance, asset_id: AssetId) -> Result<Balance, Self::Error> {
		// NOTE: in the newer version of the asset-rate pallet the `ConversionToAssetBalance`
		// is implemented.
		//
		// However, that version is not matching with the rest of the versions we use, so we
		// will implement it manually for now.
		//
		// TODO: This should be updated once we start using the versions from `1.7.0` release.

		let rate = pallet_asset_rate::ConversionRateToNative::<Runtime>::get(asset_id)
			.ok_or(DispatchError::Token(TokenError::UnknownAsset))?;

		// We cannot use `saturating_div` here so we use `checked_div`.
		Ok(FixedU128::from_u32(1)
			.checked_div(&rate)
			.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?
			.saturating_mul_int(balance))
	}
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	Any,
	/// Can execute any call that does not transfer funds or assets.
	NonTransfer,
	/// Proxy with the ability to reject time-delay proxy announcements.
	CancelProxy,
	// TODO: add more proxies in future related to coretime trading.
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances { .. } |
					RuntimeCall::Tokens { .. } |
					RuntimeCall::Currencies { .. }
			),
			ProxyType::CancelProxy =>
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })),
		}
	}
}

pub struct ExistentialDeposits;
impl GetByKey<AssetId, Balance> for ExistentialDeposits {
	fn get(asset: &AssetId) -> Balance {
		if let Some(metadata) = AssetRegistry::metadata(asset) {
			metadata.existential_deposit
		} else {
			// As restrictive as we can be. The asset must have associated metadata.
			Balance::MAX
		}
	}
}

type NegativeImbalance = <Balances as PalletCurrency<AccountId>>::NegativeImbalance;

pub struct ToStakingPot;
impl OnUnbalanced<NegativeImbalance> for ToStakingPot {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		let staking_pot = PotId::get().into_account_truncating();
		Balances::resolve_creating(&staking_pot, amount);
	}
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// 60% of the fees go to the treasury, and the rest goes to the collators along with the
			// tips.
			let (treasury, mut collators) = fees.ration(60, 40);

			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut collators);
			}

			<ToStakingPot as OnUnbalanced<_>>::on_unbalanced(collators);
			Treasury::on_unbalanced(treasury);
		}
	}
}

pub struct OrderCreationFeeHandler;
impl pallet_orders::FeeHandler<AccountId, Balance> for OrderCreationFeeHandler {
	fn handle(who: &AccountId, fee: Balance) -> DispatchResult {
		// We send the order creation fee to the treasury:
		<Runtime as pallet_orders::Config>::Currency::transfer(
			who,
			&RegionXTreasuryAccount::get(),
			fee,
			ExistenceRequirement::KeepAlive,
		)?;
		Ok(())
	}
}

#[derive(Encode, Decode)]
enum CoretimeRuntimeCalls {
	#[codec(index = 50)]
	Broker(BrokerPalletCalls),
}

/// Broker pallet calls. We don't define all of them, only the ones we use.
#[derive(Encode, Decode)]
enum BrokerPalletCalls {
	#[codec(index = 10)]
	Assign(RegionId, ParaId, Finality),
}

pub struct AssignmentCallEncoder;
impl AssignmentCallEncoderT for AssignmentCallEncoder {
	fn encode_assignment_call(region_id: RegionId, para_id: ParaId) -> sp_std::vec::Vec<u8> {
		CoretimeRuntimeCalls::Broker(BrokerPalletCalls::Assign(region_id, para_id, Finality::Final))
			.encode()
	}
}
