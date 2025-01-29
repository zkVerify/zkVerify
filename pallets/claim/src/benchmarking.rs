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
use frame_system::RawOrigin;
use sp_runtime::Saturating;

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

fn init_airdrop_state<T: Config>(
    n: u32,
    begin_airdrop: bool,
) -> BTreeMap<T::AccountId, BalanceOf<T>> {
    let (beneficiaries, total_amount) = get_beneficiaries_map::<T>(n);
    let _ = T::Currency::mint_into(
        &Pallet::<T>::account_id(),
        total_amount.saturating_mul(2u32.into()), // Just to be extra safe
    )
    .unwrap();

    if begin_airdrop {
        Pallet::<T>::begin_airdrop(RawOrigin::Root.into(), beneficiaries.clone()).unwrap();
    }
    beneficiaries
}

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn begin_airdrop(n: Linear<0, <T as Config>::MAX_BENEFICIARIES>) {
        let beneficiaries = init_airdrop_state::<T>(n, false);

        #[extrinsic_call]
        begin_airdrop(RawOrigin::Root, beneficiaries);
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
    fn add_beneficiaries(n: Linear<1, <T as Config>::MAX_BENEFICIARIES>) {
        // Init airdrop
        Pallet::<T>::begin_airdrop(RawOrigin::Root.into(), BTreeMap::new()).unwrap();

        // Prepare beneficiaries and sufficient amount
        let (beneficiaries, total_amount) = get_beneficiaries_map::<T>(n);
        let _ = T::Currency::mint_into(
            &Pallet::<T>::account_id(),
            total_amount.saturating_mul(2u32.into()), // Just to be extra safe
        )
        .unwrap();

        #[extrinsic_call]
        add_beneficiaries(RawOrigin::Root, beneficiaries);
    }

    #[benchmark]
    fn end_airdrop(n: Linear<1, <T as Config>::MAX_BENEFICIARIES>) {
        let _ = init_airdrop_state::<T>(n, true);

        #[extrinsic_call]
        end_airdrop(RawOrigin::Root);
    }

    #[cfg(test)]
    use crate::Pallet as Claim;
    impl_benchmark_test_suite!(Claim, crate::mock::test(), crate::mock::Test,);
}
