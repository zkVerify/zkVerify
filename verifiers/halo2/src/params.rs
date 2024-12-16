use codec::{Decode, Encode};
use frame_support::pallet_prelude::TypeInfo;
use halo2_proofs::halo2curves::bn256::Bn256;
use halo2_proofs::halo2curves::pairing::Engine;
use crate::vk::{G1Affine, G2Affine};

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, )] // MaxEncodedLen
pub struct ParamsKZG {
    pub k: u32,
    pub n: u64,
    pub g: G1Affine,
    // pub g_lagrange: Vec<G1Affine>,
    pub g2: G2Affine,
    pub s_g2: G2Affine,
}

impl<E: Engine> From<ParamsKZG> for halo2_proofs::poly::kzg::commitment::ParamsKZG<Bn256> {
    fn from(params: ParamsKZG) -> Self {
        Self {
            k: params.k,
            n: params.n,
            g: vec![params.g], // only g[0] is used in Shplonk and GWC
            g_lagrange: vec![], // not used in Shplonk and GWC
            g2: params.g2,
            s_g2: params.s_g2,
        }
    }
}