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
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use hp_on_proof_verified::OnProofVerified;
use sp_core::Get;
use sp_runtime::traits::Bounded;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

fn funded_account<T: Config>() -> T::AccountId {
    let caller: T::AccountId = whitelisted_caller();
    T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value() / 2u32.into());
    caller
}

fn insert_domain<T: Config>(
    domain_id: u32,
    account: AccountOf<T>,
    size: Option<u32>,
) -> AggregationSize {
    let aggregation_size = size
        .unwrap_or_else(|| <T as Config>::AggregationSize::get() as u32)
        .try_into()
        .unwrap();
    let domain = Domain::<T>::create(
        domain_id,
        account.into(),
        1,
        aggregation_size,
        <T as Config>::MaxPendingPublishQueueSize::get(),
        None,
    );
    Domains::<T>::insert(domain_id, domain);
    aggregation_size
}

fn fill_aggregation<T: Config>(caller: AccountOf<T>, domain_id: u32) {
    let domain = Domains::<T>::get(domain_id).unwrap();

    for _ in 0..domain.max_aggregation_size {
        Pallet::<T>::on_proof_verified(Some(caller.clone()), Some(domain_id), Default::default());
    }
}

#[benchmarks]
mod benchmarks {
    use __private::traits::UnfilteredDispatchable;
    use codec::{Decode, Encode};

    use super::*;

    #[benchmark]
    fn aggregate(n: Linear<1, <T as Config>::AGGREGATION_SIZE>) {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), Some(n));
        fill_aggregation::<T>(caller.clone(), domain_id);

        #[extrinsic_call]
        aggregate(RawOrigin::Signed(caller), domain_id, 1);

        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.next.id, 2);
    }

    #[benchmark]
    fn aggregate_on_invalid_domain() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;

        let call = Call::<T>::aggregate { domain_id, id: 1 };
        let benchmarked_call_encoded = Encode::encode(&call);
        #[block]
        {
            let call_decoded = <Call<T> as Decode>::decode(&mut &benchmarked_call_encoded[..])
                .expect("call is encoded above, encoding must be correct");
            let origin = RawOrigin::Signed(caller).into();

            let _ =
                <Call<T> as UnfilteredDispatchable>::dispatch_bypass_filter(call_decoded, origin);
        }

        // Sanity check: domain doesn't exist
        assert!(Domains::<T>::get(domain_id).is_none());
    }

    #[benchmark]
    fn aggregate_on_invalid_id() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        let call = Call::<T>::aggregate { domain_id, id: 1 };
        let benchmarked_call_encoded = Encode::encode(&call);
        #[block]
        {
            let call_decoded = <Call<T> as Decode>::decode(&mut &benchmarked_call_encoded[..])
                .expect("call is encoded above, encoding must be correct");
            let origin = RawOrigin::Signed(caller).into();

            let _ =
                <Call<T> as UnfilteredDispatchable>::dispatch_bypass_filter(call_decoded, origin);
        }
    }

    #[benchmark]
    fn register_domain() {
        let caller: T::AccountId = funded_account::<T>();

        #[extrinsic_call]
        register_domain(
            RawOrigin::Signed(caller),
            <T as Config>::AggregationSize::get(),
            Some(<T as Config>::MaxPendingPublishQueueSize::get()),
        );
    }

    #[benchmark]
    fn unregister_domain() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        for _ in 0..T::MaxPendingPublishQueueSize::get() {
            fill_aggregation::<T>(caller.clone(), domain_id);
        }

        #[extrinsic_call]
        unregister_domain(RawOrigin::Signed(caller), domain_id);

        // Sanity check: we consumed the aggregation
        assert!(Domains::<T>::get(domain_id).is_none());
    }

    #[cfg(test)]
    use crate::Pallet as Poe;
    impl_benchmark_test_suite!(Poe, crate::mock::test(), crate::mock::Test,);
}
