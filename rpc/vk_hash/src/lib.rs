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

use codec::{Decode, Encode};
use hp_verifiers::Verifier;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObject};
use pallet_ezkl_verifier::{Ezkl, MAX_VK_LENGTH};
use pallet_fflonk_verifier::{
    vk::{Fq, Fq2, Fr, G1, G2},
    Fflonk,
};
use pallet_groth16_verifier::{Curve, Groth16};
use pallet_plonky2_verifier::{Plonky2, Plonky2Config};
use pallet_ultrahonk_verifier::{Ultrahonk, VK_SIZE as ULTRAHONK_VK_SIZE};
use pallet_ultraplonk_verifier::{Ultraplonk, VK_SIZE};
use sp_core::{serde::Deserialize, serde::Serialize, Bytes, H256, U256};

// In order to implement the vk-hash Rpc we need to use the Runtime definition of the Verifier.
// The testnet verifier should be at least a superset of mainnet ones and the vk hash never really
// depends on from the verifier configuration.
// Anyway, a no-trivial refactoring is needed to remove the configuration dependency so we decided
// to use the volta runtime as reference.
// The case when a verifier was removed from testnet runtime but is still present in mainnet should
// never happen, at least is wired because we should remove it from mainnet before. But, if that's
// the case a use can run an older node and use it to compute the hash.
use volta_runtime as runtime;

type VkOf<V> = <V as Verifier>::Vk;

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(remote = "Curve")]
pub enum Groth16Curve {
    Bn254,
    Bls12_381,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Groth16Vk {
    #[serde(with = "Groth16Curve")]
    pub curve: Curve,
    pub alpha_g1: Bytes,
    pub beta_g2: Bytes,
    pub gamma_g2: Bytes,
    pub delta_g2: Bytes,
    pub gamma_abc_g1: Vec<Bytes>,
}

impl From<Groth16Vk> for pallet_groth16_verifier::Vk {
    fn from(vk: Groth16Vk) -> Self {
        Self {
            curve: vk.curve,
            alpha_g1: hp_groth16::G1(vk.alpha_g1.0),
            beta_g2: hp_groth16::G2(vk.beta_g2.0),
            gamma_g2: hp_groth16::G2(vk.gamma_g2.0),
            delta_g2: hp_groth16::G2(vk.delta_g2.0),
            gamma_abc_g1: vk
                .gamma_abc_g1
                .iter()
                .map(|v| hp_groth16::G1(v.0.clone()))
                .collect(),
        }
    }
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(remote = "Plonky2Config")]
pub enum Plonky2ConfigDef {
    Keccak,
    Poseidon,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plonky2Vk {
    #[serde(with = "Plonky2ConfigDef")]
    pub config: Plonky2Config,
    pub bytes: Bytes,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EzklVk {
    pub vk_bytes: Bytes,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct FflonkVk {
    pub power: u8,
    pub k1: U256,
    pub k2: U256,
    pub w: U256,
    pub w3: U256,
    pub w4: U256,
    pub w8: U256,
    pub wr: U256,
    pub x2: [[U256; 2]; 3],
    pub c0: [U256; 3],
}

impl From<FflonkVk> for pallet_fflonk_verifier::vk::Vk {
    fn from(vk: FflonkVk) -> Self {
        Self {
            power: vk.power,
            k1: Fr(vk.k1),
            k2: Fr(vk.k2),
            w: Fr(vk.w),
            w3: Fr(vk.w3),
            w4: Fr(vk.w4),
            w8: Fr(vk.w8),
            wr: Fr(vk.wr),
            x2: G2(
                Fq2(Fq(vk.x2[0][0]), Fq(vk.x2[0][1])),
                Fq2(Fq(vk.x2[1][0]), Fq(vk.x2[1][1])),
                Fq2(Fq(vk.x2[2][0]), Fq(vk.x2[2][1])),
            ),
            c0: G1(Fq(vk.c0[0]), Fq(vk.c0[1]), Fq(vk.c0[2])),
        }
    }
}

#[rpc(client, server, namespace = "vk_hash")]
pub trait VKHashApi<ResponseType> {
    #[method(name = "ezkl")]
    fn ezkl(&self, vk: EzklVk) -> RpcResult<ResponseType>;
    #[method(name = "fflonk")]
    fn fflonk(&self, vk: FflonkVk) -> RpcResult<ResponseType>;
    #[method(name = "groth16")]
    fn groth16(&self, vk: Groth16Vk) -> RpcResult<ResponseType>;
    #[method(name = "plonky2")]
    fn plonky2(&self, vk: Plonky2Vk) -> RpcResult<ResponseType>;
    #[method(name = "risc0")]
    fn risc0(&self, vk: H256) -> RpcResult<ResponseType>;
    #[method(name = "ultrahonk")]
    fn ultrahonk(&self, vk: Bytes) -> RpcResult<ResponseType>;
    #[method(name = "ultraplonk")]
    fn ultraplonk(&self, vk: Bytes) -> RpcResult<ResponseType>;
    #[method(name = "sp1")]
    fn sp1(&self, vk: H256) -> RpcResult<ResponseType>;
}

#[derive(Default)]
pub struct VKHash;

impl VKHash {
    // Creates a new instance of the vk-hash Rpc helper.
    pub fn new() -> Self {
        Self {}
    }
}

impl VKHashApiServer<H256> for VKHash {
    fn ezkl(&self, vk: EzklVk) -> RpcResult<H256> {
        let vk_bytes = vk.vk_bytes;
        if vk_bytes.is_empty()
            || vk_bytes.len() & 31 != 0
            || vk_bytes.len() > MAX_VK_LENGTH as usize
        {
            return Err(ErrorObject::owned(
                1,
                "Incorrect Slice Length",
                Some("Incorrect Slice Length".to_string()),
            ));
        }
        let vk: VkOf<Ezkl<runtime::Runtime>> = pallet_ezkl_verifier::Vk::new(vk_bytes.to_vec());
        Ok(Ezkl::<runtime::Runtime>::vk_hash(&vk))
    }

    fn fflonk(&self, vk: FflonkVk) -> RpcResult<H256> {
        Ok(Fflonk::vk_hash(&vk.into()))
    }

    fn groth16(&self, vk: Groth16Vk) -> RpcResult<H256> {
        Ok(Groth16::<runtime::Runtime>::vk_hash(&vk.into()))
    }

    fn plonky2(&self, vk: Plonky2Vk) -> RpcResult<H256> {
        let config = vk.config;
        let bytes = vk.bytes.0;
        let vk_with_config: VkOf<Plonky2<runtime::Runtime>> =
            pallet_plonky2_verifier::Vk::new(config, bytes);
        Ok(Plonky2::<runtime::Runtime>::vk_hash(&vk_with_config))
    }

    fn risc0(&self, vk: H256) -> RpcResult<H256> {
        Ok(vk)
    }

    fn ultrahonk(&self, vk: Bytes) -> RpcResult<H256> {
        if vk.len() != ULTRAHONK_VK_SIZE {
            return Err(ErrorObject::owned(
                1,
                "Incorrect Slice Length",
                Some("Incorrect Slice Length".to_string()),
            ));
        }
        let vk: VkOf<Ultrahonk<runtime::Runtime>> = vk.0.try_into().map_err(|_| {
            ErrorObject::owned(
                2,
                "Deserialize error",
                Some("Deserialize error".to_string()),
            )
        })?;
        Ok(Ultrahonk::<runtime::Runtime>::vk_hash(&vk))
    }

    fn ultraplonk(&self, vk: Bytes) -> RpcResult<H256> {
        if vk.len() != VK_SIZE {
            return Err(ErrorObject::owned(
                1,
                "Incorrect Slice Length",
                Some("Incorrect Slice Length".to_string()),
            ));
        }
        let vk: VkOf<Ultraplonk<runtime::Runtime>> = vk.0.try_into().map_err(|_| {
            ErrorObject::owned(
                2,
                "Deserialize error",
                Some("Deserialize error".to_string()),
            )
        })?;
        Ok(Ultraplonk::<runtime::Runtime>::vk_hash(&vk))
    }

    fn sp1(&self, vk: H256) -> RpcResult<H256> {
        Ok(vk)
    }
}
