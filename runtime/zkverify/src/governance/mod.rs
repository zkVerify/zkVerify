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

//! New governance configurations for the zkVerify runtime.

use crate::{
    currency::{Balance, VFY},
    governance::origins::{ReferendumCanceller, Spender},
    prod_or_fast,
    types::{AccountId, BlockNumber, DAYS},
    weights, Balances, Preimage, Referenda, Runtime, RuntimeCall, RuntimeEvent, Scheduler, System,
    Treasury,
};
use frame_support::{parameter_types, traits::EitherOf};
use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use sp_core::ConstU32;

mod origins;
mod tracks;

pub use origins::pallet_custom_origins;
use tracks::TracksInfo;

parameter_types! {
    pub const VoteLockingPeriod: BlockNumber = prod_or_fast!(7 * DAYS, 1);
}

impl pallet_conviction_voting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_conviction_voting::ZKVWeight<Runtime>;
    type Currency = Balances;
    type Polls = Referenda;
    type MaxTurnout =
        frame_support::traits::tokens::currency::ActiveIssuanceOf<Balances, Self::AccountId>;
    type MaxVotes = ConstU32<512>;
    type VoteLockingPeriod = VoteLockingPeriod;
    type BlockNumberProvider = System;
    type VotingHooks = ();
}

parameter_types! {
    pub const AlarmInterval: BlockNumber = 1;
    pub const SubmissionDeposit: Balance = VFY;
    pub const UndecidingTimeout: BlockNumber = 14 * DAYS;
}

parameter_types! {
    pub const MaxBalance: Balance = Balance::MAX;
}
pub type TreasurySpender = EitherOf<EnsureRootWithSuccess<AccountId, MaxBalance>, Spender>;

impl pallet_custom_origins::Config for Runtime {}

impl pallet_referenda::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = weights::pallet_referenda::ZKVWeight<Runtime>;
    type Scheduler = Scheduler;
    type Currency = Balances;
    type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
    type CancelOrigin = EitherOf<EnsureRoot<AccountId>, ReferendumCanceller>;
    type KillOrigin = EnsureRoot<AccountId>;
    type Slash = Treasury;
    type Votes = pallet_conviction_voting::VotesOf<Runtime>;
    type Tally = pallet_conviction_voting::TallyOf<Runtime>;
    type SubmissionDeposit = SubmissionDeposit;
    type MaxQueued = ConstU32<100>;
    type UndecidingTimeout = UndecidingTimeout;
    type AlarmInterval = AlarmInterval;
    type Tracks = TracksInfo;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}
