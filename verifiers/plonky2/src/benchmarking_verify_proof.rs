#![cfg(feature = "runtime-benchmarks")]

use crate::Plonky2 as Verifier;
use crate::{Plonky2Config, Proof, Vk};
use frame_benchmarking::v2::*;
use hp_verifiers::Verifier as _;
use pallet_verifiers::benchmarking_utils;

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub trait Config: crate::Config {}
impl<T: crate::Config> Config for T {}
pub type Call<T> = pallet_verifiers::Call<T, Verifier<T>>;

#[allow(clippy::multiple_bound_locations)]
#[benchmarks(where T: pallet_verifiers::Config<Verifier<T>>)]
mod benchmarks {
    use super::*;

    benchmarking_utils!(Verifier<T>, crate::Config);

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_2() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_2/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_2/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_2/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_2() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_2/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_2/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_2/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_2() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_2/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_2/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_2/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_2() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_2/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_2/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_2/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_3() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_3/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_3/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_3/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_3() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_3/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_3/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_3/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_3() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_3/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_3/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_3/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_3() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_3/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_3/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_3/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_4() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_4/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_4/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_4/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_4() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_4/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_4/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_4/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_4() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_4/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_4/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_4/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_4() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_4/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_4/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_4/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_5() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_5/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_5/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_5/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_5() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_5/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_5/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_5/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_5() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_5/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_5/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_5/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_5() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_5/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_5/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_5/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_6() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_6/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_6/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_6/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_6() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_6/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_6/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_6/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_6() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_6/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_6/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_6/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_6() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_6/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_6/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_6/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_7() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_7/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_7/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_7/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_7() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_7/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_7/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_7/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_7() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_7/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_7/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_7/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_7() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_7/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_7/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_7/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_8() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_8/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_8/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_8/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_8() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_8/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_8/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_8/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_8() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_8/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_8/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_8/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_8() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_8/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_8/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_8/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_9() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_9/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_9/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_9/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_9() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_9/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes = include_bytes!("resources/degree_9/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_9/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_9() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_9/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_9/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_9/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_9() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_9/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_9/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_9/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_10() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_10/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_10/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_10/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_10() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_10/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_10/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_10/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_10() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_10/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_10/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_10/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_10() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_10/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_10/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_10/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_11() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_11/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_11/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_11/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_11() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_11/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_11/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_11/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_11() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_11/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_11/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_11/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_11() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_11/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_11/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_11/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_12() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_12/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_12/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_12/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_12() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_12/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_12/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_12/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_12() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_12/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_12/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_12/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_12() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_12/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_12/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_12/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_13() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_13/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_13/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_13/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_13() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_13/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_13/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_13/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_13() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_13/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_13/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_13/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_13() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_13/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_13/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_13/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_14() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_14/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_14/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_14/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_14() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_14/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_14/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_14/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_14() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_14/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_14/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_14/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_14() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_14/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_14/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_14/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_15() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_15/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_15/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_15/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_15() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_15/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_15/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_15/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_15() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_15/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_15/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_15/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_15() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_15/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_15/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_15/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_16() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_16/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_16/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_16/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_16() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_16/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_16/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_16/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_16() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_16/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_16/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_16/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_16() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_16/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_16/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_16/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_17() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_17/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_17/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_17/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_17() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_17/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_17/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_17/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_17() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_17/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_17/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_17/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_17() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_17/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_17/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_17/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_18() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_18/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_18/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_18/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_18() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_18/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_18/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_18/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_18() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_18/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_18/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_18/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_18() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_18/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_18/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_18/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    // #[benchmark]
    // fn verify_proof_poseidon_compressed_19() {
    //     let vk = Vk::new(
    //         Plonky2Config::Poseidon,
    //         include_bytes!("resources/degree_19/compressed/poseidon/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_19/compressed/poseidon/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_19/compressed/poseidon/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    // #[benchmark]
    // fn verify_proof_keccak_compressed_19() {
    //     let vk = Vk::new(
    //         Plonky2Config::Keccak,
    //         include_bytes!("resources/degree_19/compressed/keccak/vk.bin").to_vec(),
    //     );
    //     let proof_bytes =
    //         include_bytes!("resources/degree_19/compressed/keccak/proof.bin").to_vec();
    //     let proof = Proof::new(true, proof_bytes);
    //     let pubs = include_bytes!("resources/degree_19/compressed/keccak/pubs.bin").to_vec();
    //     let r;
    //     #[block]
    //     {
    //         r = do_verify_proof::<T>(&vk, &proof, &pubs)
    //     };
    //     assert!(r.is_ok());
    // }

    #[benchmark]
    fn verify_proof_poseidon_uncompressed_19() {
        let vk = Vk::new(
            Plonky2Config::Poseidon,
            include_bytes!("resources/degree_19/uncompressed/poseidon/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_19/uncompressed/poseidon/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_19/uncompressed/poseidon/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    #[benchmark]
    fn verify_proof_keccak_uncompressed_19() {
        let vk = Vk::new(
            Plonky2Config::Keccak,
            include_bytes!("resources/degree_19/uncompressed/keccak/vk.bin").to_vec(),
        );
        let proof_bytes =
            include_bytes!("resources/degree_19/uncompressed/keccak/proof.bin").to_vec();
        // let proof = Proof::new(false, proof_bytes);
        let proof = Proof::new(proof_bytes);
        let pubs = include_bytes!("resources/degree_19/uncompressed/keccak/pubs.bin").to_vec();
        let r;
        #[block]
        {
            r = do_verify_proof::<T>(&vk, &proof, &pubs)
        };
        assert!(r.is_ok());
    }

    impl_benchmark_test_suite!(Pallet, super::mock::test_ext(), super::mock::Test);
}

#[cfg(test)]
pub mod mock {
    use frame_support::{
        derive_impl, parameter_types,
        sp_runtime::{traits::IdentityLookup, BuildStorage},
        traits::{fungible::HoldConsideration, LinearStoragePrice},
    };
    use sp_core::{ConstU128, ConstU32};

    type Balance = u128;
    type AccountId = u64;

    // Configure a mock runtime to test the pallet.
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
        type MaxProofSize = ConstU32<1000000>;
        type MaxPubsSize = ConstU32<1000000>;
        type MaxVkSize = ConstU32<1000000>;
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

    impl pallet_verifiers::Config<crate::Plonky2<Test>> for Test {
        type RuntimeEvent = RuntimeEvent;
        type OnProofVerified = ();
        type WeightInfo = crate::Plonky2Weight<()>;
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
    }

    impl pallet_verifiers::common::Config for Test {
        type CommonWeightInfo = Test;
    }

    /// Build genesis storage according to the mock runtime.
    pub fn test_ext() -> sp_io::TestExternalities {
        let mut ext = sp_io::TestExternalities::from(
            frame_system::GenesisConfig::<Test>::default()
                .build_storage()
                .unwrap(),
        );
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
