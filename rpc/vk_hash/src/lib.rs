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

use hp_verifiers::Verifier;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObject};
use pallet_proofofsql_verifier::ProofOfSql;
use pallet_ultraplonk_verifier::{Ultraplonk, VK_SIZE};
use sp_core::{Bytes, Get, H256};

// use jsonrpc_core::Result;
// use jsonrpc_derive::rpc as other_rpc;
// use serde::Deserialize;
// use sp_std::vec::Vec;

type VkOf<V> = <V as hp_verifiers::Verifier>::Vk;

struct MockType;
struct MaxPubs;

impl Get<u32> for MaxPubs {
    fn get() -> u32 {
        32
    }
}

impl pallet_ultraplonk_verifier::Config for MockType {
    type MaxPubs = MaxPubs;
}

impl pallet_proofofsql_verifier::Config for MockType {
    type LargestMaxNu = MaxPubs; // this shouldn't matter
}

#[rpc(client, server)]
pub trait VKHashApi<ResponseType> {
    #[method(name = "compute_ultraplonk")]
    fn ultraplonk(&self, vk: Bytes) -> RpcResult<ResponseType>;
    #[method(name = "compute_proofofsql")]
    fn proofofsql(&self, vk: Bytes) -> RpcResult<ResponseType>;
}

pub struct VKHash;

impl VKHash {
    // Creates a new instance of the vk-hash Rpc helper.
    pub fn new() -> Self {
        Self {}
    }
}

impl VKHashApiServer<H256> for VKHash {
    fn ultraplonk(&self, vk: Bytes) -> RpcResult<H256> {
        if vk.len() != VK_SIZE {
            return Err(ErrorObject::owned(
                1,
                "Incorrect Slice Length",
                Some("Incorrect Slice Length".to_string()),
            ));
        }
        let vk: VkOf<Ultraplonk<MockType>> = vk.0.try_into().map_err(|_| {
            ErrorObject::owned(
                2,
                "Deserialize error",
                Some("Deserialize error".to_string()),
            )
        })?;
        Ok(Ultraplonk::<MockType>::vk_hash(&vk))
    }

    fn proofofsql(&self, vk: Bytes) -> RpcResult<H256> {
        let vk: VkOf<ProofOfSql<MockType>> = pallet_proofofsql_verifier::Vk::from(vk.0);
        Ok(ProofOfSql::vk_hash(&vk))
    }
}
