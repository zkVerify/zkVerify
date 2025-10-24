// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

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

#![allow(clippy::type_complexity)]

use crate::{
    currency::{Balance, VFY},
    types::{AccountId, Signature},
};
use alloc::{boxed::Box, format, vec::Vec};
use polkadot_primitives::{AssignmentId, AuthorityDiscoveryId, ValidatorId};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_staking::StakerStatus;

pub const ENDOWMENT: Balance = 1_000_000 * VFY;
pub const STASH_BOND: Balance = ENDOWMENT / 100;
pub const DEFAULT_ENDOWED_SEEDS: [&str; 6] = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

pub type AccountPublic = <Signature as Verify>::Signer;

pub type Ids = (
    AccountId,
    BabeId,
    GrandpaId,
    ValidatorId,
    AssignmentId,
    AuthorityDiscoveryId,
);

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn from_ss58check<T: sp_core::crypto::Ss58Codec>(
    key: &str,
) -> Result<T, sp_core::crypto::PublicError> {
    <T as sp_core::crypto::Ss58Codec>::from_ss58check(key)
}

// Generate authority IDs from SS58 addresses.
pub fn authority_ids_from_ss58(
    sr25519_account_key: &str,
    sr25519_session_key: &str,
    ed25519_session_key: &str,
) -> Result<Ids, sp_core::crypto::PublicError> {
    Ok((
        from_ss58check(sr25519_account_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(ed25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
        from_ss58check(sr25519_session_key)?,
    ))
}

pub trait StakerData {
    fn staker_data(
        &self,
    ) -> (
        AccountId,
        AccountId,
        Balance,
        sp_staking::StakerStatus<AccountId>,
    );
}

impl StakerData for Box<dyn StakerData> {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        self.as_ref().staker_data()
    }
}

impl StakerData for (AccountId, Balance) {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.0.clone(),
            self.0.clone(),
            self.1,
            StakerStatus::Validator,
        )
    }
}

#[derive(Clone)]
pub struct FundedAccount {
    /// The account-id
    pub account_id: AccountId,
    /// Initial balance
    pub balance: Balance,
}

impl FundedAccount {
    pub fn from_id(
        sr25519_key: &str,
        balance: Balance,
    ) -> Result<Self, sp_core::crypto::PublicError> {
        Ok(Self {
            account_id: from_ss58check(sr25519_key)?,
            balance,
        })
    }

    pub fn json_data(&self) -> (AccountId, Balance) {
        (self.account_id.clone(), self.balance)
    }
}

#[derive(Clone)]
pub struct ValidatorData {
    /// The account-id sr25519 public key
    pub account: FundedAccount,
    ///  Bonded data
    bonded: Balance,
    babe_id: BabeId,
    grandpa_id: GrandpaId,
    validator_id: ValidatorId,
    assignment_id: AssignmentId,
    authority_discovery_id: AuthorityDiscoveryId,
}

impl StakerData for ValidatorData {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.account.account_id.clone(),
            self.account.account_id.clone(),
            self.bonded,
            StakerStatus::Validator,
        )
    }
}

impl ValidatorData {
    pub fn from_ids(
        sr25519_account_key: &str,
        sr25519_common_key: &str,
        ed25519_key: &str,
        balance: Balance,
        bonded: Balance,
    ) -> Result<Self, sp_core::crypto::PublicError> {
        let (account_id, babe_id, grandpa_id, validator_id, assignment_id, authority_discovery_id) =
            authority_ids_from_ss58(sr25519_account_key, sr25519_common_key, ed25519_key)?;
        Ok(Self {
            account: FundedAccount {
                account_id,
                balance,
            },
            bonded,
            babe_id,
            grandpa_id,
            validator_id,
            assignment_id,
            authority_discovery_id,
        })
    }

    pub fn ids(&self) -> Ids {
        (
            self.account.account_id.clone(),
            self.babe_id.clone(),
            self.grandpa_id.clone(),
            self.validator_id.clone(),
            self.assignment_id.clone(),
            self.authority_discovery_id.clone(),
        )
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct NominatorData {
    /// The account-id sr25519 public key
    pub account: FundedAccount,
    bonded: Balance,
    voted: Vec<AccountId>,
}

impl StakerData for NominatorData {
    fn staker_data(&self) -> (AccountId, AccountId, Balance, StakerStatus<AccountId>) {
        (
            self.account.account_id.clone(),
            self.account.account_id.clone(),
            self.bonded,
            StakerStatus::Nominator(self.voted.clone()),
        )
    }
}
