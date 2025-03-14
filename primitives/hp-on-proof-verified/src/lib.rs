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

#![no_std]

use sp_core::H256;
use sp_weights::Weight;

/// Trait used by proof verifier pallets to signal that a successful proof verification happened.
/// This must be implemented by proof storage pallets (e.g. pallet-aggregate) to subscribe to proof verification events.
pub trait OnProofVerified<A> {
    fn on_proof_verified(account: Option<A>, domain_id: Option<u32>, pubs_hash: H256);
    fn weight(domain_id: &Option<u32>) -> Weight;
}

#[impl_trait_for_tuples::impl_for_tuples(4)]
impl<A: Clone> OnProofVerified<A> for OnProofVerifiedTuple {
    fn on_proof_verified(account: Option<A>, domain_id: Option<u32>, pubs_hash: H256) {
        for_tuples!( #( OnProofVerifiedTuple::on_proof_verified(account.clone(), domain_id, pubs_hash); )* )
    }

    fn weight(domain_id: &Option<u32>) -> Weight {
        [for_tuples!( #( OnProofVerifiedTuple::weight(domain_id) ),* )]
            .into_iter()
            .fold(Weight::default(), |acc, w| acc.compose(w))
    }
}
pub trait Compose {
    fn compose(self, other: Self) -> Self;
}

impl Compose for Weight {
    fn compose(self, other: Self) -> Self {
        // The result time is the sum of the time.
        // The proof size is the max of the proof size of all storages. The proof
        // size is the length of the patricia tree path used to identify the state of
        // the storage. I observed the computed weight that the proof size was always
        // equal to the max of the all components, for instance when I computed the weight
        // for aggregate pallet I saw that the proof size of `on_aggregate` call was equal
        // of all `submit_proof` exstrinsic of any verifier pallet.
        // Finally, the code in
        // https://docs.rs/crate/frame-benchmarking-cli/latest/source/src/pallet/writer.rs#320
        // clearly show that it compute the max of every component and every slope: that is
        // the value used in the template to set the proof size.
        Weight::from_parts(
            self.ref_time().saturating_add(other.ref_time()),
            self.proof_size().max(other.proof_size()),
        )
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    extern crate std;
    use std::{collections::HashMap, thread_local};

    use super::*;

    type CallParameters = (Option<u64>, Option<u32>, H256);

    struct Mock<const ID: u64>;

    impl<const ID: u64> Mock<ID> {
        thread_local! {
            pub static CALLED : RefCell<HashMap<u64, CallParameters                >> = RefCell::new(HashMap::new());
        }

        pub fn called() -> (Option<u64>, Option<u32>, H256) {
            Self::CALLED.with(|c| c.borrow_mut().remove(&ID)).unwrap()
        }
    }

    impl<const ID: u64> OnProofVerified<u64> for Mock<ID> {
        fn on_proof_verified(account: Option<u64>, domain_id: Option<u32>, pubs_hash: H256) {
            Mock::<ID>::CALLED.with(|c| c.borrow_mut().insert(ID, (account, domain_id, pubs_hash)));
        }

        fn weight(domain_id: &Option<u32>) -> Weight {
            domain_id
                .map(|id| {
                    Weight::from_parts(
                        ID + id as u64 * 1_000_000,
                        ID + 1000 + id as u64 * 1_000_000,
                    )
                })
                .unwrap_or_default()
        }
    }

    #[test]
    fn test_check_on_proof_verified_for_tuple() {
        <(Mock<1>, Mock<2>)>::on_proof_verified(Some(42), Some(24), H256::from_low_u64_be(123));

        assert_eq!(
            Mock::<1>::called(),
            (Some(42), Some(24), H256::from_low_u64_be(123))
        );
        assert_eq!(
            Mock::<2>::called(),
            (Some(42), Some(24), H256::from_low_u64_be(123))
        );
    }

    #[test]
    fn test_check_weight_for_tuple() {
        let w = <(Mock<1>, Mock<2>)>::weight(&Some(4));

        let expected = Weight::from_parts(1 + 2 + 2 * (4 * 1_000_000), 1002 + (4 * 1_000_000));

        assert_eq!(expected, w);

        let w = <(Mock<1>, Mock<2>)>::weight(&None);

        assert_eq!(Weight::default(), w);
    }

    #[test]
    fn default_impl() {
        // Compile is just enough to test that the default implementation works.
        <() as OnProofVerified<u64>>::on_proof_verified(
            Some(42),
            Some(24),
            H256::from_low_u64_be(123),
        );
        let w = <() as OnProofVerified<u64>>::weight(&None);
        assert_eq!(w, Default::default());

        let w = <() as OnProofVerified<u64>>::weight(&Some(3));
        assert_eq!(w, Default::default());
    }
}
