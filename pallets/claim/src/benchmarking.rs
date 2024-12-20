// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::RawOrigin;

const MAX_BENEFICIARIES: u32 = 100;
const AMOUNT_OFFSET: u32 = 10;

fn get_beneficiaries_map<T: Config>(
    n: u32,
) -> (BTreeMap<T::AccountId, BalanceOf<T>>, BalanceOf<T>) {
    let base_amount = BalanceOf::<T>::from(100_000u32);
    let mut total_amount = BalanceOf::<T>::zero();
    let beneficiaries_map = (1..=n)
        .into_iter()
        .map(|i| {
            let amount = base_amount.saturating_add(i.into());
            total_amount = total_amount.saturating_add(amount);
            (account("", i, i), amount)
        })
        .collect::<BTreeMap<_, _>>();
    (beneficiaries_map, total_amount)
}

fn mutate_beneficiaries_map<T: Config>(beneficiaries: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
    beneficiaries
        .iter_mut()
        .enumerate()
        .for_each(|(i, (_, amount))| {
            // Modify claimable balances alternating giving more token and less tokens
            if i % 2 == 0 {
                *amount = amount.saturating_add((AMOUNT_OFFSET * i as u32).into());
            } else {
                *amount = amount.saturating_sub((AMOUNT_OFFSET * i as u32).into())
            }
        })
}

fn init_airdrop_state<T: Config>(
    n: u32,
    begin_airdrop: bool,
) -> BTreeMap<T::AccountId, BalanceOf<T>> {
    let (beneficiaries, total_amount) = get_beneficiaries_map::<T>(n);
    let _ = T::Currency::deposit_into_existing(
        &Pallet::<T>::account_id(),
        total_amount.saturating_mul(2u32.into()), // Just to be extra safe
    )
    .unwrap();

    if begin_airdrop {
        Pallet::<T>::begin_airdrop(RawOrigin::Root.into(), Some(beneficiaries.clone())).unwrap();
    }
    beneficiaries
}

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn begin_airdrop_empty_beneficiaries() {
        #[extrinsic_call]
        begin_airdrop(RawOrigin::Root, None);
    }

    #[benchmark]
    fn begin_airdrop_with_beneficiaries(n: Linear<1, MAX_BENEFICIARIES>) {
        let beneficiaries = init_airdrop_state::<T>(n, false);

        #[extrinsic_call]
        begin_airdrop(RawOrigin::Root, Some(beneficiaries));
    }

    #[benchmark]
    fn claim() {
        let _ = init_airdrop_state::<T>(100, true);

        let beneficiary: T::AccountId = account("", 10, 10);
        assert!(Beneficiaries::<T>::get(beneficiary.clone()).is_some());

        let dest = whitelisted_caller();

        #[extrinsic_call]
        claim(RawOrigin::Signed(beneficiary.clone()), Some(dest));

        // sanity check
        assert!(Beneficiaries::<T>::get(beneficiary).is_none());
    }

    #[benchmark]
    fn claim_for() {
        let _ = init_airdrop_state::<T>(100, true);

        let beneficiary: T::AccountId = account("", 10, 10);
        assert!(Beneficiaries::<T>::get(beneficiary.clone()).is_some());

        #[extrinsic_call]
        claim_for(RawOrigin::None, beneficiary.clone());

        // sanity check
        assert!(Beneficiaries::<T>::get(beneficiary).is_none());
    }

    #[benchmark]
    fn add_beneficiaries(n: Linear<1, MAX_BENEFICIARIES>) {
        let mut beneficiaries = init_airdrop_state::<T>(n, true);

        mutate_beneficiaries_map::<T>(&mut beneficiaries);

        // Worst case scenario: all the amounts of existing beneficiaries have been modified
        #[extrinsic_call]
        add_beneficiaries(RawOrigin::Root, beneficiaries);
    }

    #[benchmark]
    fn remove_beneficiaries(n: Linear<1, MAX_BENEFICIARIES>) {
        let beneficiaries = init_airdrop_state::<T>(n, true);

        #[extrinsic_call]
        remove_beneficiaries(RawOrigin::Root, beneficiaries.keys().cloned().collect());
    }

    #[benchmark]
    fn end_airdrop() {
        let _ = init_airdrop_state::<T>(100, true);

        #[extrinsic_call]
        end_airdrop(RawOrigin::Root);
    }

    #[cfg(test)]
    use crate::Pallet as Claim;
    impl_benchmark_test_suite!(Claim, crate::mock::test(), crate::mock::Test,);
}
