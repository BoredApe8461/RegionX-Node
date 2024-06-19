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

use super::*;
use pallet_referenda::Curve;

const fn percent(x: i32) -> sp_runtime::FixedI64 {
	sp_runtime::FixedI64::from_rational(x as u128, 100)
}

// Same curves as on Polkadot:

const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(16, 28 * 24, percent(96), percent(50), percent(100));
const SUP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(1, 28, percent(20), percent(5), percent(50));
const APP_TREASURER: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_TREASURER: Curve = Curve::make_linear(28, 28, percent(0), percent(50));

const APP_SMALL_TIPPER: Curve = Curve::make_linear(10, 28, percent(50), percent(100));
const SUP_SMALL_TIPPER: Curve = Curve::make_reciprocal(1, 28, percent(4), percent(0), percent(50));
const APP_BIG_TIPPER: Curve = Curve::make_linear(10, 28, percent(50), percent(100));
const SUP_BIG_TIPPER: Curve = Curve::make_reciprocal(8, 28, percent(1), percent(0), percent(50));
const APP_SMALL_SPENDER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_SMALL_SPENDER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_MEDIUM_SPENDER: Curve = Curve::make_linear(23, 28, percent(50), percent(100));
const SUP_MEDIUM_SPENDER: Curve =
	Curve::make_reciprocal(16, 28, percent(1), percent(0), percent(50));
const APP_BIG_SPENDER: Curve = Curve::make_linear(28, 28, percent(50), percent(100));
const SUP_BIG_SPENDER: Curve = Curve::make_reciprocal(20, 28, percent(1), percent(0), percent(50));

const DELEGATED_REFERENDA_TRACKS: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 2] = [
	(
		0,
		pallet_referenda::TrackInfo {
			name: "root",
			max_deciding: 1,
			decision_deposit: 200 * ROC,
			prepare_period: 2 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 24 * HOURS,
			min_enactment_period: 24 * HOURS,
			min_approval: APP_ROOT,
			min_support: SUP_ROOT,
		},
	),
	(
		1,
		pallet_referenda::TrackInfo {
			name: "whitelisted_caller",
			max_deciding: 50,
			decision_deposit: 50 * ROC,
			prepare_period: 30 * MINUTES,
			decision_period: 14 * DAYS,
			confirm_period: 30 * MINUTES,
			min_enactment_period: 30 * MINUTES,
			min_approval: APP_WHITELISTED_CALLER,
			min_support: SUP_WHITELISTED_CALLER,
		},
	),
];

const NATIVE_REFERENDA_TRACKS: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 6] = [
	(
		0,
		pallet_referenda::TrackInfo {
			name: "treasurer",
			max_deciding: 5,
			decision_deposit: 20_000 * COCOS,
			prepare_period: 12 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 48 * HOURS,
			min_enactment_period: 24 * HOURS,
			min_approval: APP_TREASURER,
			min_support: SUP_TREASURER,
		},
	),
	(
		1,
		pallet_referenda::TrackInfo {
			name: "small_tipper",
			max_deciding: 50,
			decision_deposit: 100 * COCOS,
			prepare_period: 30 * MINUTES,
			decision_period: 7 * DAYS,
			confirm_period: 4 * HOURS,
			min_enactment_period: 30 * MINUTES,
			min_approval: APP_SMALL_TIPPER,
			min_support: SUP_SMALL_TIPPER,
		},
	),
	(
		2,
		pallet_referenda::TrackInfo {
			name: "big_tipper",
			max_deciding: 20,
			decision_deposit: 250 * COCOS,
			prepare_period: 2 * HOURS,
			decision_period: 10 * DAYS,
			confirm_period: 8 * HOURS,
			min_enactment_period: 4 * HOURS,
			min_approval: APP_BIG_TIPPER,
			min_support: SUP_BIG_TIPPER,
		},
	),
	(
		3,
		pallet_referenda::TrackInfo {
			name: "small_spender",
			max_deciding: 10,
			decision_deposit: 1000 * COCOS,
			prepare_period: 4 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 12 * HOURS,
			min_enactment_period: 24 * HOURS,
			min_approval: APP_SMALL_SPENDER,
			min_support: SUP_SMALL_SPENDER,
		},
	),
	(
		4,
		pallet_referenda::TrackInfo {
			name: "medium_spender",
			max_deciding: 5,
			decision_deposit: 2000 * COCOS,
			prepare_period: 8 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 24 * HOURS,
			min_enactment_period: 24 * HOURS,
			min_approval: APP_MEDIUM_SPENDER,
			min_support: SUP_MEDIUM_SPENDER,
		},
	),
	(
		5,
		pallet_referenda::TrackInfo {
			name: "big_spender",
			max_deciding: 5,
			decision_deposit: 5000 * COCOS,
			prepare_period: 12 * HOURS,
			decision_period: 14 * DAYS,
			confirm_period: 48 * HOURS,
			min_enactment_period: 24 * HOURS,
			min_approval: APP_BIG_SPENDER,
			min_support: SUP_BIG_SPENDER,
		},
	),
];

pub struct DelegatedReferendaTrackInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for DelegatedReferendaTrackInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&DELEGATED_REFERENDA_TRACKS[..]
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::WhitelistedCaller => Ok(1),
				_ => Err(()),
			}
		} else {
			// Treasury is only controllable with native tokens.
			Err(())
		}
	}
}
pallet_referenda::impl_tracksinfo_get!(DelegatedReferendaTrackInfo, Balance, BlockNumber);

pub struct NativeReferendaTrackInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for NativeReferendaTrackInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&NATIVE_REFERENDA_TRACKS[..]
	}
	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(_system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			// Root is the only available origin for relay chain asset holders.
			Err(())
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::Treasurer => Ok(0),
				origins::Origin::SmallTipper => Ok(1),
				origins::Origin::BigTipper => Ok(2),
				origins::Origin::SmallSpender => Ok(3),
				origins::Origin::MediumSpender => Ok(4),
				origins::Origin::BigSpender => Ok(5),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
pallet_referenda::impl_tracksinfo_get!(NativeReferendaTrackInfo, Balance, BlockNumber);
