use sp_std::fmt::Debug;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;
use halo2_proofs::halo2curves::bn256::Bn256;
use crate::vk::{ConvertError, G1Affine, G2Affine};

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)] // MaxEncodedLen
pub struct ParamsKZG {
    pub k: u32,
    pub n: u64,
    pub g: G1Affine,
    pub g2: G2Affine,
    pub s_g2: G2Affine,

    // pub g_lagrange: Vec<G1Affine>,
}

impl TryFrom<ParamsKZG> for halo2_proofs::poly::kzg::commitment::ParamsKZG<Bn256> {
    type Error = ConvertError;

    fn try_from(params: ParamsKZG) -> Result<Self, Self::Error> {
        let g = params.g.try_into().unwrap();
        Ok(Self {
            k: params.k,
            n: params.n,
            g: vec![g], // only g[0] is used in Shplonk and GWC
            g_lagrange: vec![], // not used in Shplonk and GWC
            g2: params.g2.try_into().unwrap(),
            s_g2: params.s_g2.try_into().unwrap(),
        })
    }
}

impl TryFrom<halo2_proofs::poly::kzg::commitment::ParamsKZG<Bn256>> for ParamsKZG {
    type Error = ConvertError;

    fn try_from(params: halo2_proofs::poly::kzg::commitment::ParamsKZG<Bn256>) -> Result<Self, Self::Error> {
        let g = params.g[0].try_into()?;
        Ok(Self {
            k: params.k,
            n: params.n,
            g,
            g2: params.g2.try_into()?,
            s_g2: params.s_g2.try_into()?,
        })
    }
}
