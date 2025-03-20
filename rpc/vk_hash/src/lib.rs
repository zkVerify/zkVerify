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
use pallet_groth16_verifier::{Curve, Groth16};
use pallet_plonky2_verifier::{Plonky2, Plonky2Config};
use pallet_proofofsql_verifier::ProofOfSql;
use pallet_ultraplonk_verifier::{Ultraplonk, VK_SIZE};
use sp_core::{serde::Deserialize, serde::Serialize, Bytes, H256};

type VkOf<V> = <V as hp_verifiers::Verifier>::Vk;

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

#[rpc(client, server, namespace = "vk_hash")]
pub trait VKHashApi<ResponseType> {
    #[method(name = "groth16")]
    fn groth16(&self, vk: Groth16Vk) -> RpcResult<ResponseType>;
    #[method(name = "plonky2")]
    fn plonky2(&self, vk: Plonky2Vk) -> RpcResult<ResponseType>;
    #[method(name = "proofofsql")]
    fn proofofsql(&self, vk: Bytes) -> RpcResult<ResponseType>;
    #[method(name = "risc0")]
    fn risc0(&self, vk: H256) -> RpcResult<ResponseType>;
    #[method(name = "ultraplonk")]
    fn ultraplonk(&self, vk: Bytes) -> RpcResult<ResponseType>;
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
    fn groth16(&self, vk: Groth16Vk) -> RpcResult<H256> {
        let vk: VkOf<Groth16<zkv_runtime::Runtime>> = pallet_groth16_verifier::Vk {
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
        };

        Ok(Groth16::<zkv_runtime::Runtime>::vk_hash(&vk))
    }

    fn plonky2(&self, vk: Plonky2Vk) -> RpcResult<H256> {
        let config = vk.config;
        let bytes = vk.bytes.0;
        let vk_with_config: VkOf<Plonky2<zkv_runtime::Runtime>> =
            pallet_plonky2_verifier::Vk::new(config, bytes);
        Ok(Plonky2::<zkv_runtime::Runtime>::vk_hash(&vk_with_config))
    }

    fn proofofsql(&self, vk: Bytes) -> RpcResult<H256> {
        let vk: VkOf<ProofOfSql<zkv_runtime::Runtime>> = pallet_proofofsql_verifier::Vk::from(vk.0);
        Ok(ProofOfSql::vk_hash(&vk))
    }

    fn risc0(&self, vk: H256) -> RpcResult<H256> {
        Ok(vk)
    }

    fn ultraplonk(&self, vk: Bytes) -> RpcResult<H256> {
        if vk.len() != VK_SIZE {
            return Err(ErrorObject::owned(
                1,
                "Incorrect Slice Length",
                Some("Incorrect Slice Length".to_string()),
            ));
        }
        let vk: VkOf<Ultraplonk<zkv_runtime::Runtime>> = vk.0.try_into().map_err(|_| {
            ErrorObject::owned(
                2,
                "Deserialize error",
                Some("Deserialize error".to_string()),
            )
        })?;
        Ok(Ultraplonk::<zkv_runtime::Runtime>::vk_hash(&vk))
    }
}
