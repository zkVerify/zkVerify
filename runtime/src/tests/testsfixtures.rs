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

use sp_core::{Pair, Public};
use sp_std::sync::LazyLock;

use crate::{currency, Balance, EXISTENTIAL_DEPOSIT};

use crate::*;
// Existential deposit used in pallet_balances
pub const EXISTENTIAL_DEPOSIT_REMAINDER: Balance = 1;
pub const NUM_TEST_ACCOUNTS: u32 = 4;
pub const STASH_DEPOSIT: Balance = 1 * currency::ACME; // MUST not be smaller than EXISTENTIAL_DEPOSIT
pub const NUM_VALIDATORS: u32 = 2;

pub fn get_from_seed<TPublic: Public>(seed: u8) -> TPublic::Pair {
    TPublic::Pair::from_string(&format!("//test_seed{}", seed), None)
        .expect("static values are valid; qed")
}

/// The BABE epoch configuration at genesis.
const TEST_BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: crate::PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

/// Function used for creating the environment for the test.
/// It must return a sp_io::TestExternalities, and the actual test will execute this one before running.
pub fn test() -> sp_io::TestExternalities {
    // This builds the initial genesis storage for this test
    let mut t = frame_system::GenesisConfig::<super::Runtime>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<super::Runtime> {
        balances: SAMPLE_USERS
            .iter()
            .cloned()
            .map(|user| (user.raw_account.into(), user.starting_balance))
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_babe::GenesisConfig::<super::Runtime> {
        authorities: vec![],
        epoch_config: TEST_BABE_GENESIS_EPOCH_CONFIG,
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Add authorities
    pallet_session::GenesisConfig::<super::Runtime> {
        keys: SAMPLE_USERS
            .iter()
            .cloned()
            .map(|user| {
                (
                    user.raw_account.into(),
                    user.raw_account.into(),
                    SessionKeys {
                        babe: get_from_seed::<BabeId>(user.session_key_seed).public(),
                        grandpa: get_from_seed::<GrandpaId>(user.session_key_seed).public(),
                        #[cfg(not(feature = "relay"))]
                        im_online: get_from_seed::<ImOnlineId>(user.session_key_seed).public(),
                        #[cfg(feature = "relay")]
                        para_validator: get_from_seed::<ValidatorId>(user.session_key_seed)
                            .public(),
                        #[cfg(feature = "relay")]
                        para_assignment: get_from_seed::<polkadot_primitives::AssignmentId>(
                            user.session_key_seed,
                        )
                        .public(),
                        #[cfg(feature = "relay")]
                        authority_discovery: get_from_seed::<
                            polkadot_primitives::AuthorityDiscoveryId,
                        >(user.session_key_seed)
                        .public(),
                    },
                )
            })
            .take(NUM_VALIDATORS as usize)
            .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_staking::GenesisConfig::<super::Runtime> {
        stakers: SAMPLE_USERS
            .iter()
            .cloned()
            .map(|user| {
                (
                    user.raw_account.into(),
                    user.raw_account.into(),
                    STASH_DEPOSIT,
                    sp_staking::StakerStatus::Validator::<AccountId>,
                )
            })
            .take(NUM_VALIDATORS as usize)
            .collect(),
        minimum_validator_count: NUM_VALIDATORS,
        validator_count: NUM_VALIDATORS,
        canceled_payout: 0,
        force_era: pallet_staking::Forcing::ForceNone,
        invulnerables: [].to_vec(),
        max_nominator_count: None,
        max_validator_count: None,
        min_nominator_bond: 1,
        min_validator_bond: STASH_DEPOSIT,
        slash_reward_fraction: Perbill::zero(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_sudo::GenesisConfig::<super::Runtime> {
        key: Some(SAMPLE_USERS[0].raw_account.into()),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_claim::GenesisConfig::<super::Runtime> {
        beneficiaries: SAMPLE_USERS
            .iter()
            .cloned()
            .map(|user| (user.raw_account.into(), user.starting_balance))
            .collect(),
        genesis_balance: TOTAL_BALANCE.clone(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(t);

    ext.execute_with(|| System::set_block_number(1));

    // Return the test externalities
    ext
}

#[derive(Clone)]
pub struct SampleAccount {
    pub raw_account: [u8; 32],
    pub starting_balance: Balance,
    pub session_key_seed: u8,
}

// Build a vector containing a few sample user accounts, along with their starting balances
pub static SAMPLE_USERS: [SampleAccount; NUM_TEST_ACCOUNTS as usize] = [
    SampleAccount {
        raw_account: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ],
        starting_balance: 1000001 * currency::ACME,
        session_key_seed: 1,
    },
    SampleAccount {
        raw_account: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ],
        starting_balance: 12345432 * currency::ACME,
        session_key_seed: 2,
    },
    SampleAccount {
        raw_account: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 3,
        ],
        starting_balance: 9955223 * currency::ACME,
        session_key_seed: 3,
    },
    SampleAccount {
        raw_account: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 4,
        ],
        starting_balance: EXISTENTIAL_DEPOSIT,
        session_key_seed: 4,
    },
];

pub static TOTAL_BALANCE: LazyLock<Balance> = LazyLock::new(|| {
    SAMPLE_USERS
        .iter()
        .map(|account| account.starting_balance)
        .sum()
});
