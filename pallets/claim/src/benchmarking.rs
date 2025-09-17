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

use super::*;
use crate::utils::*;
use codec::{Decode, Encode};
use frame_benchmarking::v2::*;
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_core::{ecdsa::*, Pair as TraitPair};
use sp_runtime::{
    traits::{IdentifyAccount, ValidateUnsigned},
    MultiSignature, MultiSigner, Saturating,
};

pub trait BenchmarkHelper<Signature, Signer> {
    fn sign_claim(message: &[u8]) -> (Signature, Signer);
}

impl BenchmarkHelper<MultiSignature, MultiSigner> for () {

    fn sign_claim(message: &[u8]) -> (MultiSignature, MultiSigner) {
        let pair = Pair::from_seed_slice(b"//BenchSeed").unwrap();
        let signer = MultiSigner::Ecdsa(pair.public());

        // Generate Signature
        let raw_signature = pair.sign(message);
        let signature = MultiSignature::Ecdsa(raw_signature);
        (signature, signer)
    }
}

fn get_claim_message<T: Config>() -> BoundedVec<u8, <T as Config>::MaxClaimMessageLength> {
    BoundedVec::try_from(vec![
        1u8;
        <T as Config>::MaxClaimMessageLength::get() as usize
    ])
    .unwrap()
}

fn init_claim_state<T: Config>(n: u32, begin_claim: bool) -> BTreeMap<T::AccountId, BalanceOf<T>> {
    let (beneficiaries, total_amount) = get_beneficiaries_map::<T>(n);
    let _ = T::Currency::mint_into(
        &Pallet::<T>::account_id(),
        total_amount.saturating_mul(2u32.into()), // Just to be extra safe
    )
    .unwrap();

    if begin_claim {
        Pallet::<T>::begin_claim(
            RawOrigin::Root.into(),
            beneficiaries.clone(),
            get_claim_message::<T>(),
        )
        .unwrap();
    }
    beneficiaries
}

#[benchmarks]
mod benchmarks {

    use super::*;

    #[benchmark]
    fn begin_claim(n: Linear<0, <T as Config>::MAX_OP_BENEFICIARIES>) {
        let beneficiaries = init_claim_state::<T>(n, false);

        #[extrinsic_call]
        begin_claim(RawOrigin::Root, beneficiaries, get_claim_message::<T>());
    }

    #[benchmark]
    fn claim() {
        let _ = init_claim_state::<T>(<T as Config>::MAX_OP_BENEFICIARIES - 1, true);

        let msg = [ETH_PREFIX, get_claim_message::<T>().as_slice()].concat(); // Maximally sized message
        let (signature, signer) = T::BenchmarkHelper::sign_claim(msg.as_slice());
        let beneficiary = signer.clone().into_account();

        // Insert beneficiary
        let amount =
            BalanceOf::<T>::from(T::Currency::minimum_balance().saturating_add(1u32.into()));
        Beneficiaries::<T>::insert(beneficiary.clone(), amount);
        assert!(Beneficiaries::<T>::get(beneficiary.clone()).is_some());

        let call_enc = Call::<T>::claim {
            beneficiary: signer,
            signature: signature,
        }
        .encode();
        let source = sp_runtime::transaction_validity::TransactionSource::External;

        #[block]
        {
            let call = <Call<T> as Decode>::decode(&mut &*call_enc).unwrap();
            super::Pallet::<T>::validate_unsigned(source, &call).unwrap();
            call.dispatch_bypass_filter(RawOrigin::None.into()).unwrap();
        }

        // sanity check
        assert!(Beneficiaries::<T>::get(beneficiary).is_none());
    }

    #[benchmark]
    fn claim_for() {
        let _ = init_claim_state::<T>(<T as Config>::MAX_OP_BENEFICIARIES, true);

        let beneficiary: T::AccountId = account("", 10, 10);
        assert!(Beneficiaries::<T>::get(beneficiary.clone()).is_some());

        #[extrinsic_call]
        claim_for(RawOrigin::Root, beneficiary.clone());

        // sanity check
        assert!(Beneficiaries::<T>::get(beneficiary).is_none());
    }

    #[benchmark]
    fn add_beneficiaries(n: Linear<1, <T as Config>::MAX_OP_BENEFICIARIES>) {
        // Init claim
        Pallet::<T>::begin_claim(
            RawOrigin::Root.into(),
            BTreeMap::new(),
            get_claim_message::<T>(),
        )
        .unwrap();

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
    fn end_claim() {
        let _ = init_claim_state::<T>(0, true);

        // Mint some tokens into pallet account just to trigger a transfer
        let _ = T::Currency::mint_into(
            &Pallet::<T>::account_id(),
            T::Currency::minimum_balance().saturating_mul(100u32.into()),
        )
        .unwrap();

        #[extrinsic_call]
        end_claim(RawOrigin::Root);

        assert_eq!(Pallet::<T>::pot(), BalanceOf::<T>::zero());
    }

    #[benchmark]
    fn remove_beneficiaries(n: Linear<1, <T as Config>::MAX_OP_BENEFICIARIES>) {
        let (beneficiaries, _) =
            get_beneficiaries_map::<T>(n + <T as Config>::MAX_OP_BENEFICIARIES);
        beneficiaries
            .into_iter()
            .for_each(|(account, amount)| Beneficiaries::<T>::insert(account, amount));
        assert_eq!(
            Beneficiaries::<T>::count(),
            n + <T as Config>::MAX_OP_BENEFICIARIES
        );

        #[extrinsic_call]
        remove_beneficiaries(RawOrigin::Root);

        assert_eq!(Beneficiaries::<T>::count(), 0);
    }

    #[cfg(test)]
    use crate::Pallet as Claim;
    impl_benchmark_test_suite!(Claim, crate::mock::test(), crate::mock::Test,);
}
