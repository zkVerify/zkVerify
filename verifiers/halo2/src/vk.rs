// Copyright 2024, Horizen Labs, Inc.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Debug;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::U256;
use halo2_proofs::halo2curves::bn256;

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct Fr(U256);
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct Fq(U256);
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct Fq2(Fq, Fq);
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct G1Affine(Fq, Fq);
#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
struct G2Affine(Fq2, Fq2);

#[derive(Clone, Debug, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Vk {
    pub commitments: Vec<G1Affine>,
}

trait IntoBytes {
    fn into_bytes(self) -> [u8; 32];
}

impl IntoBytes for U256 {
    fn into_bytes(self) -> [u8; 32] {
        let mut out = [0; 32];
        self.to_big_endian(&mut out);
        out
    }
}

impl From<Fr> for bn256::Fr {
    fn from(value: Fr) -> Self {
        bn256::Fr::from_bytes(&value.0.into_bytes()).expect("BUG: should be hardcoded. qed")
    }
}

impl TryInto<bn256::Fq> for Fq {
    type Error = ConvertError;

    fn try_into(self) -> Result<substrate_bn::Fq, Self::Error> {
        bn256::Fq::from_bytes(&self.0.into_bytes()).ok_or(ConvertError::NotAMemberFq)
    }
}

impl TryInto<bn256::Fq2> for Fq2 {
    type Error = ConvertError;

    fn try_into(self) -> Result<bn256::Fq2, Self::Error> {
        Ok(bn256::Fq2::new(
            self.0.try_into()?,
            self.1.try_into()?,
        ))
    }
}

#[derive(Debug)]
pub enum ConvertError {
    NotAMemberFq,
    InvalidG1Point,
    InvalidG2Point,
}

impl TryInto<bn256::G1Affine> for G1Affine {
    type Error = ConvertError;

    fn try_into(self) -> Result<bn256::G1Affine, Self::Error> {
        let g1 = bn256::G1Affine { x: self.0.try_into()?, y: self.1.try_into()? };
        Ok(g1)
    }
}

impl TryInto<bn256::G2Affine> for G2Affine {
    type Error = ConvertError;

    fn try_into(self) -> Result<bn256::G2Affine, Self::Error> {
        let g1 = bn256::G2Affine { x: self.0.try_into()?, y: self.1.try_into()? };
        Ok(g1)
    }
}

impl TryInto<halo2_proofs::plonk::VerifyingKey<bn256::G1Affine>> for Vk {
    type Error = ConvertError;

    fn try_into(self) -> Result<halo2_proofs::plonk::VerifyingKey<bn256::G1Affine>, Self::Error> {

        Ok(halo2_proofs::plonk::VerifyingKey::<bn256::G1Affine> {
        })
    }
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod test_utils {
    use super::*;

    impl From<substrate_bn::Fr> for Fr {
        fn from(value: substrate_bn::Fr) -> Self {
            let mut buf = [0; 32];
            value.into_u256().to_big_endian(&mut buf).unwrap();
            Self(U256::from_big_endian(&buf))
        }
    }

    impl From<substrate_bn::Fq> for Fq {
        fn from(value: substrate_bn::Fq) -> Self {
            let mut buf = [0; 32];
            value.to_big_endian(&mut buf).unwrap();
            Self(buf.into())
        }
    }

    impl From<substrate_bn::Fq2> for Fq2 {
        fn from(value: substrate_bn::Fq2) -> Self {
            Self(value.real().into(), value.imaginary().into())
        }
    }

    impl From<substrate_bn::G1> for G1 {
        fn from(value: substrate_bn::G1) -> Self {
            Self(value.x().into(), value.y().into(), value.z().into())
        }
    }

    impl From<substrate_bn::G2> for G2 {
        fn from(value: substrate_bn::G2) -> Self {
            Self(value.x().into(), value.y().into(), value.z().into())
        }
    }

    impl From<fflonk_verifier::VerificationKey> for Vk {
        fn from(value: fflonk_verifier::VerificationKey) -> Self {
            Self {
                power: value.power,
                k1: value.k1.into(),
                k2: value.k2.into(),
                w: value.w.into(),
                w3: value.w3.into(),
                w4: value.w4.into(),
                w8: value.w8.into(),
                wr: value.wr.into(),
                x2: value.x2.into(),
                c0: value.c0.into(),
            }
        }
    }

    impl AsMut<U256> for Fr {
        fn as_mut(&mut self) -> &mut U256 {
            &mut self.0
        }
    }

    impl AsMut<U256> for Fq {
        fn as_mut(&mut self) -> &mut U256 {
            &mut self.0
        }
    }

    impl Vk {
        pub fn mut_k1(&mut self) -> &mut U256 {
            self.k1.as_mut()
        }
        pub fn mut_x2_x_real(&mut self) -> &mut U256 {
            &mut self.x2.0.0.0
        }
        pub fn mut_c0_x(&mut self) -> &mut U256 {
            &mut self.c0.0.0
        }
    }
}
