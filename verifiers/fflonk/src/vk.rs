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

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::U256;
#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct Fr(pub U256);
#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct Fq(pub U256);
#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct Fq2(pub Fq, pub Fq);
#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct G1(pub Fq, pub Fq, pub Fq);
#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct G2(pub Fq2, pub Fq2, pub Fq2);

#[derive(
    Clone, Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, TypeInfo, MaxEncodedLen,
)]
pub struct Vk {
    pub power: u8,
    pub k1: Fr,
    pub k2: Fr,
    pub w: Fr,
    pub w3: Fr,
    pub w4: Fr,
    pub w8: Fr,
    pub wr: Fr,
    pub x2: G2,
    pub c0: G1,
}

trait IntoBytes {
    fn into_bytes(self) -> [u8; 32];
}

impl IntoBytes for U256 {
    fn into_bytes(self) -> [u8; 32] {
        self.to_big_endian()
    }
}

impl From<Fr> for substrate_bn::Fr {
    fn from(value: Fr) -> Self {
        substrate_bn::Fr::from_slice(&value.0.into_bytes()).expect("BUG: should be hardcoded. qed")
    }
}

impl TryInto<substrate_bn::Fq> for Fq {
    type Error = ConvertError;

    fn try_into(self) -> Result<substrate_bn::Fq, Self::Error> {
        substrate_bn::Fq::from_slice(&self.0.into_bytes()).map_err(|e| match e {
            substrate_bn::FieldError::NotMember => ConvertError::NotAMemberFq,
            _ => unreachable!("Slice length is fixed. qed"),
        })
    }
}

impl TryInto<substrate_bn::Fq2> for Fq2 {
    type Error = ConvertError;

    fn try_into(self) -> Result<substrate_bn::Fq2, Self::Error> {
        Ok(substrate_bn::Fq2::new(
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

impl TryInto<substrate_bn::G1> for G1 {
    type Error = ConvertError;

    fn try_into(self) -> Result<substrate_bn::G1, Self::Error> {
        let g1 = substrate_bn::G1::new(self.0.try_into()?, self.1.try_into()?, self.2.try_into()?);
        let mut check = g1;
        use substrate_bn::Group;
        check.normalize();
        substrate_bn::AffineG1::new(check.x(), check.y())
            .map_err(|_e| ConvertError::InvalidG1Point)?;
        Ok(g1)
    }
}

impl G1 {
    pub fn try_into_substrate_bn_unchecked(self) -> Result<substrate_bn::G1, ConvertError> {
        let mut g1 =
            substrate_bn::G1::new(self.0.try_into()?, self.1.try_into()?, self.2.try_into()?);
        use substrate_bn::Group;
        g1.normalize();
        Ok(g1)
    }
}

impl TryInto<substrate_bn::G2> for G2 {
    type Error = ConvertError;

    fn try_into(self) -> Result<substrate_bn::G2, Self::Error> {
        let g2 = substrate_bn::G2::new(self.0.try_into()?, self.1.try_into()?, self.2.try_into()?);
        let mut check = g2;
        use substrate_bn::Group;
        check.normalize();
        substrate_bn::AffineG2::new(check.x(), check.y())
            .map_err(|_e| ConvertError::InvalidG2Point)?;
        Ok(g2)
    }
}

impl G2 {
    pub fn try_into_substrate_bn_unchecked(self) -> Result<substrate_bn::G2, ConvertError> {
        let mut g2 =
            substrate_bn::G2::new(self.0.try_into()?, self.1.try_into()?, self.2.try_into()?);
        use substrate_bn::Group;
        g2.normalize();
        Ok(g2)
    }
}

impl TryInto<fflonk_verifier::VerificationKey> for Vk {
    type Error = ConvertError;

    fn try_into(self) -> Result<fflonk_verifier::VerificationKey, Self::Error> {
        Ok(fflonk_verifier::VerificationKey {
            power: self.power,
            k1: self.k1.into(),
            k2: self.k2.into(),
            w: self.w.into(),
            w3: self.w3.into(),
            w4: self.w4.into(),
            w8: self.w8.into(),
            wr: self.wr.into(),
            x2: self.x2.try_into()?,
            c0: self.c0.try_into()?,
        })
    }
}

impl Vk {
    pub fn try_into_fflonk_vk_unchecked(
        self,
    ) -> Result<fflonk_verifier::VerificationKey, ConvertError> {
        Ok(fflonk_verifier::VerificationKey {
            power: self.power,
            k1: self.k1.into(),
            k2: self.k2.into(),
            w: self.w.into(),
            w3: self.w3.into(),
            w4: self.w4.into(),
            w8: self.w8.into(),
            wr: self.wr.into(),
            x2: self.x2.try_into_substrate_bn_unchecked()?,
            c0: self.c0.try_into_substrate_bn_unchecked()?,
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
            Self(U256::from_big_endian(&buf))
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
            &mut self.x2.0 .0 .0
        }
        pub fn mut_c0_x(&mut self) -> &mut U256 {
            &mut self.c0.0 .0
        }
    }
}
