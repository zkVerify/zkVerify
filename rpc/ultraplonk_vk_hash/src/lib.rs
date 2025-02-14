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



use std::sync::Arc;

use hp_verifiers::Verifier;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use pallet_ultraplonk_verifier::Ultraplonk;
use sp_core::{Get, H256};

struct MockType;

struct MaxPubs;

impl pallet_ultraplonk_verifier::Config for MockType {
    type MaxPubs = MaxPubs;
}

impl Get<u32> for MaxPubs {
    fn get() -> u32 {
        32
    }
}

type VkOf<V> = <V as hp_verifiers::Verifier>::Vk;

#[rpc(client, server)]
pub trait VKHashApi<ResponseType> {
    #[method(name = "compute_vk_hash_ultraplonk")]
    fn compute_vk_hash_ultraplonk(
        &self,
        vk: &VkOf<Ultraplonk<MockType>>,
    ) -> RpcResult<ResponseType>;
}

pub struct VKHash<C> {
    client: Arc<C>,
}

impl<C> VKHash<C> {
    // Creates a new instance of the vk-hash Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> VKHashApiServer<H256> for VKHash<C>
where
    C: Send + Sync + 'static,
{
    fn compute_vk_hash_ultraplonk(
        &self,
        vk: &VkOf<Ultraplonk<MockType>>,
    ) -> RpcResult<H256> {
        Ok(Ultraplonk::<MockType>::vk_hash(vk))
    }
}
