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

use alloc::vec::Vec;
use frame_benchmarking::v2::*;
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::RawOrigin;
use hp_on_proof_verified::OnProofVerified;
use sp_core::Get;
use sp_runtime::traits::Bounded;

type BalanceOf<T> =
    <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

const SEED: u32 = 0;

pub mod utils {
    use super::*;
    use crate::data;
    use crate::data::{Delivery, DeliveryParams};
    use hp_dispatch::Destination;

    /// Return a allowlisted account with enough funds to do anything.
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
            data::ProofSecurityRules::OnlyAllowlisted, //Always the worst case
            None,
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
    use crate::data::{AggregateSecurityRules, ProofSecurityRules};
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
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            alloc::vec![caller.clone()],
        )
        .unwrap();
        insert_statements::<T>(caller.clone(), domain_id, Some(size - 1));

        #[block]
        {
            Pallet::<T>::on_proof_verified(
                Some(caller.clone()),
                Some(domain_id),
                Default::default(),
            );
        }

        // Sanity check: we put the aggregation in should be published
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.should_publish.len(), 1);
    }

    #[benchmark]
    fn aggregate(n: Linear<1, <T as Config>::AGGREGATION_SIZE>) {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), Some(n));
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            alloc::vec![caller.clone()],
        )
        .unwrap();
        fill_aggregation::<T>(caller.clone(), domain_id);

        #[extrinsic_call]
        aggregate(RawOrigin::Signed(caller), domain_id, 1);

        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.next.id, 2);
    }

    #[benchmark(extra)]
    fn aggregate_1() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        const N: u32 = 1;
        insert_domain::<T>(domain_id, caller.clone(), Some(N));
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            alloc::vec![caller.clone()],
        )
        .unwrap();
        fill_aggregation::<T>(caller.clone(), domain_id);
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            Pallet::<T>::do_aggregate(&aggregator, domain_id, 1).unwrap();
        }
        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.next.id, 2);
    }

    #[benchmark(extra)]
    fn aggregate_10() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        const N: u32 = 1;
        insert_domain::<T>(domain_id, caller.clone(), Some(N));
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            alloc::vec![caller.clone()],
        )
        .unwrap();
        for _ in 1..=10 {
            fill_aggregation::<T>(caller.clone(), domain_id);
        }
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            for id in 1..=10 {
                Pallet::<T>::do_aggregate(&aggregator, domain_id, id).unwrap();
            }
        }
        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();
        assert!(domain.next.statements.is_empty());
        assert_eq!(domain.next.id, 11);
    }

    #[benchmark(extra)]
    fn aggregate_100() {
        let caller: T::AccountId = funded_account::<T>();
        let domains_id: Vec<_> = (1..=10).collect();
        const N: u32 = 1;
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(N));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for _ in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
            }
        }
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            for domain_id in domains_id.iter().cloned() {
                for id in 1..=10 {
                    Pallet::<T>::do_aggregate(&aggregator, domain_id, id).unwrap();
                }
            }
        }
        // Sanity check: we consumed the aggregation
        for domain_id in domains_id {
            let domain = Domains::<T>::get(domain_id).unwrap();
            assert!(domain.next.statements.is_empty());
            assert_eq!(domain.next.id, 11);
        }
    }

    #[benchmark(extra)]
    fn aggregate_1000() {
        let caller: T::AccountId = funded_account::<T>();
        let domains_id: Vec<_> = (1..=100).collect();
        const N: u32 = 1;
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(N));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for _ in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
            }
        }
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            for domain_id in domains_id.iter().cloned() {
                for id in 1..=10 {
                    Pallet::<T>::do_aggregate(&aggregator, domain_id, id).unwrap();
                }
            }
        }
        // Sanity check: we consumed the aggregation
        for domain_id in domains_id {
            let domain = Domains::<T>::get(domain_id).unwrap();
            assert!(domain.next.statements.is_empty());
            assert_eq!(domain.next.id, 11);
        }
    }

    #[benchmark(extra)]
    fn aggregate_4800() {
        let caller: T::AccountId = funded_account::<T>();
        let domains_id: Vec<_> = (1..=480).collect();
        const N: u32 = 1;
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(N));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for _ in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
            }
        }
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            for domain_id in domains_id.iter().cloned() {
                for id in 1..=10 {
                    Pallet::<T>::do_aggregate(&aggregator, domain_id, id).unwrap();
                }
            }
        }
        // Sanity check: we consumed the aggregation
        for domain_id in domains_id {
            let domain = Domains::<T>::get(domain_id).unwrap();
            assert!(domain.next.statements.is_empty());
            assert_eq!(domain.next.id, 11);
        }
    }

    #[benchmark(extra)]
    fn aggregate_400_var(n: Linear<1, <T as Config>::AGGREGATION_SIZE>) {
        let caller: T::AccountId = funded_account::<T>();
        let domains_id: Vec<_> = (1..=40).collect();
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(n));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for _ in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
            }
        }
        let aggregator = crate::data::User::Account(caller);

        #[block]
        {
            for domain_id in domains_id.iter().cloned() {
                for id in 1..=10 {
                    Pallet::<T>::do_aggregate(&aggregator, domain_id, id).unwrap();
                }
            }
        }
        // Sanity check: we consumed the aggregation
        for domain_id in domains_id {
            let domain = Domains::<T>::get(domain_id).unwrap();
            assert!(domain.next.statements.is_empty());
            assert_eq!(domain.next.id, 11);
        }
    }

    #[benchmark(extra)]
    fn clean_submitted_storage_0() {
        #[block]
        {
            use frame_support::traits::OnInitialize;
            Pallet::<T>::on_initialize(0_u32.into());
        }
        assert!(Published::<T>::get().is_empty());
    }

    #[benchmark(extra)]
    fn clean_submitted_storage_1() {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(
            domain_id,
            caller.clone(),
            Some(<T as Config>::AGGREGATION_SIZE),
        );
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            alloc::vec![caller.clone()],
        )
        .unwrap();
        fill_aggregation::<T>(caller.clone(), domain_id);
        let aggregator = crate::data::User::Account(caller);
        Pallet::<T>::do_aggregate(&aggregator, domain_id, 1).unwrap();

        assert!(!Published::<T>::get().is_empty());
        #[block]
        {
            use frame_support::traits::OnInitialize;
            Pallet::<T>::on_initialize(0_u32.into());
        }
        assert!(Published::<T>::get().is_empty());
    }

    #[benchmark(extra)]
    fn clean_submitted_storage_4800_1() {
        let caller: T::AccountId = funded_account::<T>();
        let aggregator = crate::data::User::Account(caller.clone());
        const NEED_AGGREGATIONS: u32 = 4800;
        const AGGREGATION_SIZE: u32 = 1;
        let domains_id: Vec<_> = (1..=NEED_AGGREGATIONS / 10).collect();
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(AGGREGATION_SIZE));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for a_id in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
                Pallet::<T>::do_aggregate(&aggregator, domain_id, a_id).unwrap();
            }
        }

        assert_eq!(Published::<T>::get().len(), NEED_AGGREGATIONS as usize);
        #[block]
        {
            use frame_support::traits::OnInitialize;
            Pallet::<T>::on_initialize(0_u32.into());
        }
        assert!(Published::<T>::get().is_empty());
    }

    #[benchmark(extra)]
    fn clean_submitted_storage_1400_16() {
        let caller: T::AccountId = funded_account::<T>();
        let aggregator = crate::data::User::Account(caller.clone());
        const NEED_AGGREGATIONS: u32 = 1400;
        const AGGREGATION_SIZE: u32 = 16;
        let domains_id: Vec<_> = (1..=NEED_AGGREGATIONS / 10).collect();
        for domain_id in domains_id.iter().cloned() {
            insert_domain::<T>(domain_id, caller.clone(), Some(AGGREGATION_SIZE));
            Pallet::<T>::allowlist_proof_submitters(
                RawOrigin::Signed(caller.clone()).into(),
                domain_id,
                alloc::vec![caller.clone()],
            )
            .unwrap();
            for a_id in 1..=10 {
                fill_aggregation::<T>(caller.clone(), domain_id);
                Pallet::<T>::do_aggregate(&aggregator, domain_id, a_id).unwrap();
            }
        }

        assert_eq!(Published::<T>::get().len(), NEED_AGGREGATIONS as usize);
        #[block]
        {
            use frame_support::traits::OnInitialize;
            Pallet::<T>::on_initialize(0_u32.into());
        }
        assert!(Published::<T>::get().is_empty());
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
            ProofSecurityRules::OnlyAllowlisted,
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
        set_total_delivery_fee(
            RawOrigin::Signed(caller),
            domain_id,
            12345_u32.into(),
            123_u32.into(),
        );

        // Sanity check: we consumed the aggregation
        let domain = Domains::<T>::get(domain_id).unwrap();

        assert_eq!(domain.delivery.fee(), &12345_u32.into());
    }

    #[benchmark]
    fn allowlist_proof_submitters(n: Linear<0, <T as Config>::SUBMITTER_LIST_MAX_SIZE>) {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        let submitters = (0..n)
            .map(|i| account("submitter", i, SEED))
            .collect::<Vec<_>>();

        #[extrinsic_call]
        allowlist_proof_submitters(RawOrigin::Signed(caller), domain_id, submitters);

        assert_eq!(
            n,
            SubmittersAllowlist::<T>::iter_key_prefix(domain_id).count() as u32,
            "Not all submitter add"
        );
    }

    #[benchmark]
    fn remove_proof_submitters(n: Linear<0, <T as Config>::SUBMITTER_LIST_MAX_SIZE>) {
        let caller: T::AccountId = funded_account::<T>();
        let domain_id = 1;
        insert_domain::<T>(domain_id, caller.clone(), None);

        let submitters = (0..n)
            .map(|i| account("submitter", i, SEED))
            .collect::<Vec<_>>();
        Pallet::<T>::allowlist_proof_submitters(
            RawOrigin::Signed(caller.clone()).into(),
            domain_id,
            submitters.clone(),
        )
        .unwrap();
        assert_eq!(
            n,
            SubmittersAllowlist::<T>::iter_key_prefix(domain_id).count() as u32,
        );

        #[extrinsic_call]
        remove_proof_submitters(RawOrigin::Signed(caller), domain_id, submitters);

        assert_eq!(
            0,
            SubmittersAllowlist::<T>::iter_key_prefix(domain_id).count() as u32,
            "Not all submitter removed"
        );
    }

    #[cfg(test)]
    use crate::Pallet as Aggregate;
    impl_benchmark_test_suite!(Aggregate, crate::mock::test(), crate::mock::Test,);
}
