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
use hp_on_proof_verified::OnProofVerified;
use sp_core::Get;
use sp_runtime::traits::Bounded;

type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub mod utils {
    use super::*;
    use crate::data::{Delivery, DeliveryParams};
    use hp_dispatch::Destination;

    /// Return a whitelisted account with enough funds to do anything.
    pub fn funded_account<T: Config>() -> T::AccountId {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, BalanceOf::<T>::max_value() / 2u32.into());
        caller
    }

    pub(crate) fn delivery<T: Config>(destination: Destination) -> Delivery<crate::BalanceOf<T>> {
        Delivery::new(destination, 1_000_000_000_u32.into(), 1_000_000_u32.into())
    }

    /// Insert a domain into the system.
    pub fn insert_domain<T: Config>(
        domain_id: u32,
        account: AccountOf<T>,
        size: Option<u32>,
    ) -> AggregationSize {
        let aggregation_size = size
            .unwrap_or_else(|| <T as Config>::AggregationSize::get() as u32)
            .try_into()
            .unwrap();

        let delivery = DeliveryParams::new(account.clone(), delivery::<T>(Destination::None));

        let domain = Domain::<T>::try_create(
            domain_id,
            account.into(),
            1,
            aggregation_size,
            <T as Config>::MaxPendingPublishQueueSize::get(),
            data::AggregateSecurityRules::Untrusted,
            None,
            delivery,
        )
        .unwrap();
        Domains::<T>::insert(domain_id, domain);
        aggregation_size
    }
}

fn insert_statements<T: Config>(caller: AccountOf<T>, domain_id: u32, elements: Option<u32>) {
    let domain = Domains::<T>::get(domain_id).unwrap();
    let elements = elements.unwrap_or_else(|| domain.max_aggregation_size);

    for _ in 0..elements {
        Pallet::<T>::on_proof_verified(Some(caller.clone()), Some(domain_id), Default::default());
    }
}

fn fill_aggregation<T: Config>(caller: AccountOf<T>, domain_id: u32) {
    insert_statements::<T>(caller, domain_id, None);
}

#[benchmarks]
mod benchmarks {
    use super::{utils::*, *};
    use __private::traits::UnfilteredDispatchable;
    use codec::{Decode, Encode};
    use data::DomainState;
    use hp_dispatch::Destination;

    #[benchmark]
    fn on_proof_verified() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        let size = 16;
        insert_domain::<T>(domain_id, caller.clone(), Some(size));
        insert_statements::<T>(caller.clone(), domain_id, Some(size - 1));

        #[block]
        {
            Pallet::<T>::on_proof_verified(
                Some(caller.clone()),
                Some(domain_id),
                Default::default(),
            );
        }

        // Sanity check: we putted the aggregation in should be published
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.should_publish.len(), 1);
    }

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

        let call = Call::<T>::aggregate {
            domain_id,
            aggregation_id: 1,
        };
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

        let call = Call::<T>::aggregate {
            domain_id,
            aggregation_id: 1,
        };
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

        let delivery = delivery::<T>(Destination::None);

        #[extrinsic_call]
        register_domain(
            RawOrigin::Signed(caller),
            <T as Config>::AggregationSize::get(),
            Some(<T as Config>::MaxPendingPublishQueueSize::get()),
            AggregateSecurityRules::Untrusted,
            delivery,
            Some(caller.clone()),
        );
    }

    #[benchmark]
    fn hold_domain() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        #[extrinsic_call]
        hold_domain(RawOrigin::Signed(caller), domain_id);

        // Sanity check: we consumed the aggregation
        assert_eq!(
            Domains::<T>::get(domain_id).map(|d| d.state),
            Some(DomainState::Removable)
        );
    }

    #[benchmark]
    fn unregister_domain() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        Domains::<T>::try_mutate(domain_id, |domain| {
            if let Some(d) = domain.as_mut() {
                d.state = DomainState::Removable;
            }
            Ok::<(), ()>(())
        })
        .unwrap();

        for _ in 0..T::MaxPendingPublishQueueSize::get() {
            fill_aggregation::<T>(caller.clone(), domain_id);
        }

        #[extrinsic_call]
        unregister_domain(RawOrigin::Signed(caller), domain_id);

        // Sanity check: we consumed the aggregation
        assert!(Domains::<T>::get(domain_id).is_none());
    }

    #[benchmark]
    fn set_total_delivery_fee() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        #[extrinsic_call]
        set_total_delivery_fee(RawOrigin::Signed(caller), domain_id, 12345_u32.into(), 123_u32.into());

        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();

        assert_eq!(domain.delivery.fee(), &12345_u32.into());
    }

    use crate::data::AggregateSecurityRules;
    #[cfg(test)]
    use crate::Pallet as Aggregate;
    impl_benchmark_test_suite!(Aggregate, crate::mock::test(), crate::mock::Test,);
}
