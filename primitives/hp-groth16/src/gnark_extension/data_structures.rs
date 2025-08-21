extern crate alloc;

use crate::data_structures::{vec_max_encoded_len, G1, G2};
use alloc::{vec, vec::Vec};
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_ff::PrimeField;
use ark_serialize::SerializationError;
use codec::{Decode, Encode, MaxEncodedLen};
use core::fmt::Debug;
use scale_info::TypeInfo;
use sp_runtime_interface::pass_by::{PassByCodec, PassByInner};

/// A Groth16 Proof with wire commitments, according to Gnark-Groth16 extension
#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo, PassByCodec)]
pub struct Proof {
    /// `a` point
    pub a: G1,
    /// `b` point
    pub b: G2,
    /// `c` point
    pub c: G1,
    /// Pedersen commitments a la https://eprint.iacr.org/2022/1072
    pub commitments: Vec<G1>,
    /// Batched proof of knowledge of the commitments
    pub commitments_pok: G1,
}

pub(crate) struct ArkProof<E: Pairing> {
    pub proof: ark_groth16::Proof<E>,
    pub commitments: Vec<E::G1Affine>,
    pub commitments_pok: E::G1Affine,
}

impl<E: Pairing> TryInto<ArkProof<E>> for Proof {
    type Error = SerializationError;

    fn try_into(self) -> Result<ArkProof<E>, Self::Error> {
        let ark_proof = ark_groth16::Proof {
            a: self.a.try_into_affine::<E::G1Affine>()?,
            b: self.b.try_into_affine::<E::G2Affine>()?,
            c: self.c.try_into_affine::<E::G1Affine>()?,
        };
        let commitments = self
            .commitments
            .into_iter()
            .map(|v| v.try_into_affine::<E::G1Affine>())
            .collect::<Result<Vec<_>, _>>()?;
        let commitments_pok = self.commitments_pok.try_into_affine::<E::G1Affine>()?;
        Ok(ArkProof {
            proof: ark_proof,
            commitments,
            commitments_pok,
        })
    }
}

impl<E: Pairing> TryFrom<ArkProof<E>> for Proof {
    type Error = SerializationError;

    fn try_from(value: ArkProof<E>) -> Result<Self, Self::Error> {
        Ok(Proof {
            a: G1::try_from_affine(value.proof.a)?,
            b: G2::try_from_affine(value.proof.b)?,
            c: G1::try_from_affine(value.proof.c)?,
            commitments: value
                .commitments
                .into_iter()
                .map(G1::try_from_affine)
                .collect::<Result<Vec<_>, _>>()?,
            commitments_pok: G1::try_from_affine(value.commitments_pok)?,
        })
    }
}

/// Verification key for Pedersen commitment
#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo, PassByCodec)]
pub struct PedersenVerificationKey {
    /// Base point
    pub g: G2,
    /// g^{-σ}
    pub g_sigma_neg: G2,
}

pub(crate) struct ArkPedersenVerificationKey<E: Pairing> {
    pub g: E::G2Affine,
    pub g_sigma_neg: E::G2Affine,
}

impl<E: Pairing> TryInto<ArkPedersenVerificationKey<E>> for PedersenVerificationKey {
    type Error = SerializationError;

    fn try_into(self) -> Result<ArkPedersenVerificationKey<E>, Self::Error> {
        Ok(ArkPedersenVerificationKey {
            g: self.g.try_into_affine::<E::G2Affine>()?,
            g_sigma_neg: self.g_sigma_neg.try_into_affine::<E::G2Affine>()?,
        })
    }
}

impl<E: Pairing> TryFrom<ArkPedersenVerificationKey<E>> for PedersenVerificationKey {
    type Error = SerializationError;

    fn try_from(value: ArkPedersenVerificationKey<E>) -> Result<Self, Self::Error> {
        Ok(PedersenVerificationKey {
            g: G2::try_from_affine(value.g)?,
            g_sigma_neg: G2::try_from_affine(value.g_sigma_neg)?,
        })
    }
}

impl PedersenVerificationKey {
    /// Convert a `VerificationKey` into a `ark_groth16::VerifyingKey` without checking
    /// that points are on the curve.
    pub(crate) fn try_into_ark_unchecked<E: Pairing>(
        self,
    ) -> Result<ArkPedersenVerificationKey<E>, SerializationError> {
        Ok(ArkPedersenVerificationKey {
            g: self.g.try_into_affine_unchecked::<E::G2Affine>()?,
            g_sigma_neg: self
                .g_sigma_neg
                .try_into_affine_unchecked::<E::G2Affine>()?,
        })
    }
}

/// A Groth16 Verification Key
#[derive(Clone, Debug, PartialEq, Encode, Decode, TypeInfo, PassByCodec)]
pub struct VerificationKey {
    /// `alpha_g1` point
    pub alpha_g1: G1,
    /// `beta_g2` point
    pub beta_g2: G2,
    /// `gamma_g2` point
    pub gamma_g2: G2,
    /// `delta_g2` point
    pub delta_g2: G2,
    /// `gamma_abc_g1` points
    pub gamma_abc_g1: Vec<G1>,
    /// Verification keys for Pedersen commitments
    pub commitment_keys: Vec<PedersenVerificationKey>,
    /// Indexes of public/commitment committed variables
    pub public_and_commitment_committed: Vec<Vec<usize>>,
}

pub(crate) struct ArkVerificationKey<E: Pairing> {
    pub vk: ark_groth16::VerifyingKey<E>,
    pub commitment_keys: Vec<ArkPedersenVerificationKey<E>>,
    pub public_and_commitment_committed: Vec<Vec<usize>>,
}

impl<E: Pairing> TryFrom<VerificationKey> for ArkVerificationKey<E> {
    type Error = SerializationError;

    fn try_from(value: VerificationKey) -> Result<Self, Self::Error> {
        let vk = ark_groth16::VerifyingKey {
            alpha_g1: value.alpha_g1.try_into_affine::<E::G1Affine>()?,
            beta_g2: value.beta_g2.try_into_affine::<E::G2Affine>()?,
            gamma_g2: value.gamma_g2.try_into_affine::<E::G2Affine>()?,
            delta_g2: value.delta_g2.try_into_affine::<E::G2Affine>()?,
            gamma_abc_g1: value
                .gamma_abc_g1
                .into_iter()
                .map(|v| v.try_into_affine::<E::G1Affine>())
                .collect::<Result<Vec<_>, _>>()?,
        };
        let commitment_keys = value
            .commitment_keys
            .into_iter()
            .map(|v| PedersenVerificationKey::try_into(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ArkVerificationKey {
            vk,
            commitment_keys,
            public_and_commitment_committed: value.public_and_commitment_committed,
        })
    }
}

impl<E: Pairing> TryFrom<ArkVerificationKey<E>> for VerificationKey {
    type Error = SerializationError;

    fn try_from(value: ArkVerificationKey<E>) -> Result<Self, Self::Error> {
        let alpha_g1 = G1::try_from_affine(value.vk.alpha_g1)?;
        let beta_g2 = G2::try_from_affine(value.vk.beta_g2)?;
        let gamma_g2 = G2::try_from_affine(value.vk.gamma_g2)?;
        let delta_g2 = G2::try_from_affine(value.vk.delta_g2)?;
        let gamma_abc_g1 = value
            .vk
            .gamma_abc_g1
            .into_iter()
            .map(G1::try_from_affine)
            .collect::<Result<Vec<_>, _>>()?;
        let commitment_keys = value
            .commitment_keys
            .into_iter()
            .map(|v| PedersenVerificationKey::try_from(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VerificationKey {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
            commitment_keys,
            public_and_commitment_committed: value.public_and_commitment_committed,
        })
    }
}

impl VerificationKey {
    /// Convert a `VerificationKey` into a `ark_groth16::VerifyingKey` without checking
    /// that points are on the curve.
    pub(crate) fn try_into_ark_unchecked<E: Pairing>(
        self,
    ) -> Result<ArkVerificationKey<E>, SerializationError> {
        let vk = ark_groth16::VerifyingKey {
            alpha_g1: self.alpha_g1.try_into_affine_unchecked::<E::G1Affine>()?,
            beta_g2: self.beta_g2.try_into_affine_unchecked::<E::G2Affine>()?,
            gamma_g2: self.gamma_g2.try_into_affine_unchecked::<E::G2Affine>()?,
            delta_g2: self.delta_g2.try_into_affine_unchecked::<E::G2Affine>()?,
            gamma_abc_g1: self
                .gamma_abc_g1
                .into_iter()
                .map(|v| v.try_into_affine_unchecked::<E::G1Affine>())
                .collect::<Result<Vec<_>, _>>()?,
        };
        let commitment_keys = self
            .commitment_keys
            .into_iter()
            .map(|v| PedersenVerificationKey::try_into_ark_unchecked(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ArkVerificationKey {
            vk,
            commitment_keys,
        })
    }
}
