// Copyright 2026, Horizen Labs, Inc.
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

use crate::{
    Config as VerifierConfig, Fp, Kimchi as Verifier, KimchiSrsId, NativeProof,
    NativeVerifierIndex, Proof, Pubs, Vk, PUB_SIZE,
};
use alloc::vec::Vec;
use frame_benchmarking::v2::*;
use pallet_verifiers::benchmarking_utils;
use pallet_verifiers::traits::Verifier as _;
use poly_commitment::SRS as _;
use rand_chacha::ChaCha20Rng;

pub trait Config: crate::Config {}
pub struct Pallet<T: Config>(crate::Pallet<T>);
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

const DOMAIN_4096_PROOF: &[u8] = include_bytes!("resources/generated_4096/proof.bin");
const DOMAIN_4096_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_4096/verifier_index.bin");
const DOMAIN_65536_PUBS_64_PROOF: &[u8] =
    include_bytes!("resources/generated_65536_pubs_64/proof.bin");
const DOMAIN_65536_PUBS_64_VERIFIER_INDEX: &[u8] =
    include_bytes!("resources/generated_65536_pubs_64/verifier_index.bin");
const DOMAIN_65536_PUBS_64_PUBS: &[u8] =
    include_bytes!("resources/generated_65536_pubs_64/pubs.bin");

fn benchmark_data<T: VerifierConfig>(
    proof: &[u8],
    verifier_index: &[u8],
    pubs: &[u8],
    srs_id: KimchiSrsId,
) -> (Proof, Vk<T>, Pubs) {
    (
        proof.to_vec(),
        Vk::new(verifier_index.to_vec(), srs_id),
        decode_pubs(pubs),
    )
}

fn domain_4096_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_4096_PROOF,
        DOMAIN_4096_VERIFIER_INDEX,
        &[],
        KimchiSrsId::Vesta16,
    )
}

fn domain_65536_pubs_64_data<T: VerifierConfig>() -> (Proof, Vk<T>, Pubs) {
    benchmark_data::<T>(
        DOMAIN_65536_PUBS_64_PROOF,
        DOMAIN_65536_PUBS_64_VERIFIER_INDEX,
        DOMAIN_65536_PUBS_64_PUBS,
        KimchiSrsId::Vesta16,
    )
}

fn decode_pubs(bytes: &[u8]) -> Pubs {
    assert!(
        bytes.len().is_multiple_of(PUB_SIZE),
        "Kimchi public input fixture must be a sequence of {PUB_SIZE}-byte fields"
    );
    bytes
        .chunks_exact(PUB_SIZE)
        .map(|chunk| {
            chunk
                .try_into()
                .expect("chunks_exact always returns PUB_SIZE bytes")
        })
        .collect()
}

fn prepared_data<T: VerifierConfig>(
    data: impl FnOnce() -> (Proof, Vk<T>, Pubs),
) -> (NativeVerifierIndex, NativeProof, Vec<Fp>, ChaCha20Rng) {
    let (raw_proof, vk, raw_pubs) = data();
    let proof = crate::decode_proof(&raw_proof).expect("benchmark proof should decode");
    let public_input =
        crate::decode_public_input(&raw_pubs).expect("benchmark public input should decode");
    let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
    crate::prepare_verifier_index(&mut verifier_index, vk.srs_id)
        .expect("benchmark verifier index should prepare");
    let rng = crate::make_rng(&vk, &raw_proof, &raw_pubs);

    (verifier_index, proof, public_input, rng)
}

fn prepared_domain_4096_data<T: VerifierConfig>(
) -> (NativeVerifierIndex, NativeProof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_4096_data::<T>)
}

fn prepared_domain_65536_pubs_64_data<T: VerifierConfig>(
) -> (NativeVerifierIndex, NativeProof, Vec<Fp>, ChaCha20Rng) {
    prepared_data(domain_65536_pubs_64_data::<T>)
}

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    #[benchmark]
    fn verify_proof_domain_4096() {
        let (proof, vk, pubs) = domain_4096_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_domain_65536_pubs_64() {
        let (proof, vk, pubs) = domain_65536_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn decode_proof_domain_4096() {
        let proof = DOMAIN_4096_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark]
    fn decode_proof_domain_65536_pubs_64() {
        let proof = DOMAIN_65536_PUBS_64_PROOF;

        let decoded;
        #[block]
        {
            decoded = crate::decode_proof(proof)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark]
    fn decode_vk_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark]
    fn decode_vk_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();

        let decoded;
        #[block]
        {
            decoded = crate::decode_vk(&vk)
        };
        assert!(decoded.is_ok());
    }

    #[benchmark]
    fn builtin_srs_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.srs_id,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark]
    fn builtin_srs_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        let domain_size = usize::try_from(verifier_index.domain.size)
            .expect("benchmark domain size should fit usize");

        let srs;
        #[block]
        {
            srs = crate::builtin_srs(
                vk.srs_id,
                verifier_index.max_poly_size,
                domain_size,
                verifier_index.public,
            )
        };
        assert!(srs.is_ok());
    }

    #[benchmark]
    fn prepare_verifier_index_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.srs_id)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark]
    fn prepare_verifier_index_domain_65536_pubs_64() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");

        let prepared;
        #[block]
        {
            prepared = crate::prepare_verifier_index(&mut verifier_index, vk.srs_id)
        };
        assert!(prepared.is_ok());
    }

    #[benchmark]
    fn lagrange_basis_domain_4096() {
        let (_, vk, _) = domain_4096_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.srs_id)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark]
    fn lagrange_basis_domain_65536() {
        let (_, vk, _) = domain_65536_pubs_64_data::<T>();
        let mut verifier_index = crate::decode_vk(&vk).expect("benchmark VK should decode");
        crate::prepare_verifier_index(&mut verifier_index, vk.srs_id)
            .expect("benchmark verifier index should prepare");

        let basis_len;
        #[block]
        {
            basis_len = verifier_index
                .srs()
                .get_lagrange_basis(verifier_index.domain)
                .len()
        };
        assert_eq!(basis_len, verifier_index.public);
    }

    #[benchmark]
    fn verify_prepared_domain_4096() {
        let (verifier_index, proof, public_input, mut rng) = prepared_domain_4096_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_native_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_prepared_domain_65536_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_65536_pubs_64_data::<T>();

        let r;
        #[block]
        {
            r = crate::verify_native_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_prepared_warmed_domain_4096() {
        let (verifier_index, proof, public_input, mut rng) = prepared_domain_4096_data::<T>();

        let basis_len = verifier_index
            .srs()
            .get_lagrange_basis(verifier_index.domain)
            .len();
        assert_eq!(basis_len, verifier_index.public);

        let r;
        #[block]
        {
            r = crate::verify_native_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_prepared_warmed_domain_65536_pubs_64() {
        let (verifier_index, proof, public_input, mut rng) =
            prepared_domain_65536_pubs_64_data::<T>();

        let basis_len = verifier_index
            .srs()
            .get_lagrange_basis(verifier_index.domain)
            .len();
        assert_eq!(basis_len, verifier_index.public);

        let r;
        #[block]
        {
            r = crate::verify_native_proof_with_rng(
                &verifier_index,
                &proof,
                &public_input,
                &mut rng,
            )
        };
        assert!(r.is_ok());
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice},
    };
    use sp_core::{ConstU128, ConstU32};

    type Balance = u128;
    type AccountId = u64;

    frame_support::construct_runtime!(
        pub enum Test
        {
            System: frame_system,
            Balances: pallet_balances,
            CommonVerifiersPallet: pallet_verifiers::common,
            VerifierPallet: crate,
        }
    );

    impl crate::Config for Test {
        type MaxProofSize = ConstU32<262144>;
        type MaxPubs = ConstU32<64>;
        type MaxVkSize = ConstU32<65536>;
        type WeightInfo = ();
    }

    #[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
    impl frame_system::Config for Test {
        type Block = frame_system::mocking::MockBlockU32<Test>;
        type AccountId = AccountId;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const BaseDeposit: Balance = 1;
        pub const PerByteDeposit: Balance = 2;
        pub const HoldReasonVkRegistration: RuntimeHoldReason = RuntimeHoldReason::CommonVerifiersPallet(pallet_verifiers::common::HoldReason::VkRegistration);
    }

    impl pallet_verifiers::Config<crate::Kimchi<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::KimchiWeight<()>;
        type Ticket = HoldConsideration<
            AccountId,
            Balances,
            HoldReasonVkRegistration,
            LinearStoragePrice<BaseDeposit, PerByteDeposit, Balance>,
        >;
        type Currency = Balances;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type WeightInfo = ();
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxLocks = ConstU32<10>;
        type MaxReserves = ConstU32<10>;
        type MaxFreezes = ConstU32<10>;
        type DoneSlashHandler = ();
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .expect("mock genesis storage should build"),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
