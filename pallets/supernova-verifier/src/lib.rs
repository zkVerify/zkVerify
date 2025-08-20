#![cfg_attr(not(feature = "std"), no_std)]

//! SuperNova verifier pallet (no-std compatible)

pub mod verifier {
    use nova_snark::{
        nova::{PublicParams, RecursiveSNARK},
        provider::{PallasEngine, VestaEngine},
        traits::Engine,
    };
    use ark_std::vec::Vec;

    use crate::verifier::FibStep;

    use super::*;

    // StepCircuit implementation (from the CLI)
    use nova_snark::frontend::{num::AllocatedNum, ConstraintSystem, SynthesisError};
    use nova_snark::traits::circuit::StepCircuit;
    use ff::PrimeField;

    #[derive(Clone, Default)]
    pub struct FibStep;

    impl<F: PrimeField> StepCircuit<F> for FibStep {
        fn arity(&self) -> usize { 2 }

        fn synthesize<CS: ConstraintSystem<F>>(
            &self,
            cs: &mut CS,
            z: &[AllocatedNum<F>],
        ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
            assert_eq!(z.len(), 2, "FibStep expects arity=2");
            let z0 = &z[0];
            let z1 = &z[1];

            let sum = AllocatedNum::alloc(cs.namespace(|| "sum"), || {
                let a = z0.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                let b = z1.get_value().ok_or(SynthesisError::AssignmentMissing)?;
                Ok(a + b)
            })?;

            cs.enforce(
                || "z0 + z1 = sum",
                |lc| lc + z0.get_variable() + z1.get_variable(),
                |lc| lc + CS::one(),
                |lc| lc + sum.get_variable(),
            );

            Ok(vec![z1.clone(), sum])
        }
    }

    /// Core no-std verifier (base64 → bincode → verify)
    pub fn verify_proof(
        vk_b64: &[u8],
        proof_b64: &[u8],
        num_steps: u32,
    ) -> Result<bool, ()> {
        type E1 = PallasEngine;
        type E2 = VestaEngine;
        type F1 = <E1 as Engine>::Scalar;

        let vk_str = core::str::from_utf8(vk_b64).map_err(|_| ())?;
        let pp: PublicParams<E1, E2, FibStep> =
            crate::verifier::decode_b64(vk_str).map_err(|_| ())?;

        let proof_str = core::str::from_utf8(proof_b64).map_err(|_| ())?;
        let rs: RecursiveSNARK<E1, E2, FibStep> =
            crate::verifier::decode_b64(proof_str).map_err(|_| ())?;

        // z0 = [1,1] (same as in CLI)
        let z0 = Vec::<F1>::from([F1::ONE, F1::ONE]);

        match rs.verify(&pp, num_steps as usize, &z0) {
            Ok(_z_final) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    // ===== Reuse CLI helpers (decode_b64, read_json, f_to_hex etc.) here if needed =====
    // You can port them later as additional functions.
}

use frame_support::{pallet_prelude::*, ensure};
use frame_system::pallet_prelude::*;

#[pallet::pallet]
pub struct Pallet<T>(_);

#[pallet::config]
pub trait Config: frame_system::Config {}

#[pallet::error]
pub enum Error<T> {
    VerificationFailed,
    ProofInvalid,
}

#[pallet::call]
impl<T: Config> Pallet<T> {
    #[pallet::weight(10_000)]
    pub fn verify(
        origin: OriginFor<T>,
        vk_b64: Vec<u8>,
        proof_b64: Vec<u8>,
        num_steps: u32,
    ) -> DispatchResult {
        // Only root (sudo) may call this
        ensure_root(origin)?;

        // call the no-std verifier
        let ok = verifier::verify_proof(&vk_b64, &proof_b64, num_steps)
            .map_err(|_| Error::<T>::VerificationFailed)?;

        // simple check
        ensure!(ok, Error::<T>::ProofInvalid);

        Ok(())
    }
}
