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

use frame_election_provider_support::private::sp_arithmetic;

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
    sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}
use super::origins;
use crate::{
    currency::{Balance, THOUSANDS},
    types::{BlockNumber, DAYS, HOURS, MINUTES},
    RuntimeOrigin,
};
use pallet_referenda::Curve;

const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_REFERENDUM_CANCELLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_CANCELLER: Curve =
    Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_MEDIUM_SPENDER: Curve = Curve::make_linear(23, 28, percent(50), percent(100));
const SUP_MEDIUM_SPENDER: Curve =
    Curve::make_reciprocal(16, 28, percent(1), percent(0), percent(50));

const TRACKS_DATA: &[(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>)] = &[
    #[cfg(any(feature = "runtime-benchmarks", test))]
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
        33,
        pallet_referenda::TrackInfo {
            name: "medium_spender",
            max_deciding: 50,
            decision_deposit: THOUSANDS,
            prepare_period: 4 * HOURS,
            decision_period: 28 * DAYS,
            confirm_period: 4 * DAYS,
            min_enactment_period: 24 * HOURS,
            min_approval: APP_MEDIUM_SPENDER,
            min_support: SUP_MEDIUM_SPENDER,
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
        if cfg!(any(feature = "runtime-benchmarks", test)) {
            match frame_system::RawOrigin::try_from(id.clone()) {
                Ok(frame_system::RawOrigin::Root) => return Ok(0),
                Ok(_) => return Err(()),
                _ => {}
            }
        }
        if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
            match custom_origin {
                origins::Origin::WishForChange => Ok(2),
                // Referendum admins
                origins::Origin::ReferendumCanceller => Ok(20),
                origins::Origin::MediumSpender => Ok(33),
            }
        } else {
            Err(())
        }
    }
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
