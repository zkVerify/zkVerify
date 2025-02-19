// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Track configurations for governance.

use super::*;
use frame_election_provider_support::private::sp_arithmetic;

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
    sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;
const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_STAKING_ADMIN: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_STAKING_ADMIN: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_TREASURER: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_TREASURER: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_REFERENDUM_CANCELLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_CANCELLER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_REFERENDUM_KILLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_KILLER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
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

const TRACKS_DATA: &[(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>)] = &[
    #[cfg(feature = "runtime-benchmarks")]
    (
        0,
        pallet_referenda::TrackInfo {
            name: "root",
            max_deciding: 1,
            decision_deposit: 100 * THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 24 * HOURS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_ROOT,
            min_support: SUP_ROOT,
        },
    ),
    (
        2,
        pallet_referenda::TrackInfo {
            name: "wish_for_change",
            max_deciding: 10,
            decision_deposit: 20 * THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 24 * HOURS,
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_ROOT,
            min_support: SUP_ROOT,
        },
    ),
    (
        10,
        pallet_referenda::TrackInfo {
            name: "staking_admin",
            max_deciding: 10,
            decision_deposit: 5 * THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 3 * HOURS,
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_STAKING_ADMIN,
            min_support: SUP_STAKING_ADMIN,
        },
    ),
    (
        11,
        pallet_referenda::TrackInfo {
            name: "treasurer",
            max_deciding: 10,
            decision_deposit: THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 7 * DAYS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_TREASURER,
            min_support: SUP_TREASURER,
        },
    ),
    (
        20,
        pallet_referenda::TrackInfo {
            name: "referendum_canceller",
            max_deciding: 1_000,
            decision_deposit: 10 * THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 7 * DAYS,
            confirm_period: 3 * HOURS,
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_REFERENDUM_CANCELLER,
            min_support: SUP_REFERENDUM_CANCELLER,
        },
    ),
    (
        21,
        pallet_referenda::TrackInfo {
            name: "referendum_killer",
            max_deciding: 1_000,
            decision_deposit: 50 * THOUSANDS,
            prepare_period: 2 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 3 * HOURS,
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_REFERENDUM_KILLER,
            min_support: SUP_REFERENDUM_KILLER,
        },
    ),
    (
        30,
        pallet_referenda::TrackInfo {
            name: "small_tipper",
            max_deciding: 200,
            decision_deposit: ACME,
            prepare_period: MINUTES,
            decision_period: 7 * DAYS,
            confirm_period: 10 * MINUTES,
            min_enactment_period: MINUTES,
            min_approval: APP_SMALL_TIPPER,
            min_support: SUP_SMALL_TIPPER,
        },
    ),
    (
        31,
        pallet_referenda::TrackInfo {
            name: "big_tipper",
            max_deciding: 100,
            decision_deposit: 10 * ACME,
            prepare_period: 10 * MINUTES,
            decision_period: 7 * DAYS,
            confirm_period: HOURS,
            min_enactment_period: 10 * MINUTES,
            min_approval: APP_BIG_TIPPER,
            min_support: SUP_BIG_TIPPER,
        },
    ),
    (
        32,
        pallet_referenda::TrackInfo {
            name: "small_spender",
            max_deciding: 50,
            decision_deposit: 100 * ACME,
            prepare_period: 4 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 2 * DAYS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_SMALL_SPENDER,
            min_support: SUP_SMALL_SPENDER,
        },
    ),
    (
        33,
        pallet_referenda::TrackInfo {
            name: "medium_spender",
            max_deciding: 50,
            decision_deposit: 200 * ACME,
            prepare_period: 4 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 4 * DAYS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_MEDIUM_SPENDER,
            min_support: SUP_MEDIUM_SPENDER,
        },
    ),
    (
        34,
        pallet_referenda::TrackInfo {
            name: "big_spender",
            max_deciding: 50,
            decision_deposit: 400 * ACME,
            prepare_period: 4 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 7 * DAYS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_BIG_SPENDER,
            min_support: SUP_BIG_SPENDER,
        },
    ),
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
    type Id = u16;
    type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
    fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
        TRACKS_DATA
    }
    fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
        let system_origin = frame_system::RawOrigin::try_from(id.clone());
        if cfg!(feature = "runtime-benchmarks") && system_origin.is_ok() {
            match system_origin.unwrap() {
                frame_system::RawOrigin::Root => Ok(0),
                _ => Err(()),
            }
        } else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
            match custom_origin {
                origins::Origin::WishForChange => Ok(2),
                // General admin
                origins::Origin::StakingAdmin => Ok(10),
                origins::Origin::Treasurer => Ok(11),
                // Referendum admins
                origins::Origin::ReferendumCanceller => Ok(20),
                origins::Origin::ReferendumKiller => Ok(21),
                // Limited treasury spenders
                origins::Origin::SmallTipper => Ok(30),
                origins::Origin::BigTipper => Ok(31),
                origins::Origin::SmallSpender => Ok(32),
                origins::Origin::MediumSpender => Ok(33),
                origins::Origin::BigSpender => Ok(34),
            }
        } else {
            Err(())
        }
    }
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);