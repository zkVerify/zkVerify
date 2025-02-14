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

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{ErrorObject, ErrorObjectOwned},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait VKHashApi<ResponseType> {
    #[method(name = "compute_vk_hash_ultraplonk")]
    fn compute_vk_hash_ultraplonk(&self, vk: Ultraplonk::Vk) -> RpcResult<ResponseType>;
}

pub struct VKHash<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> VKHash<C, P> {
    // Creates a new instance of the vk-hash Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

// Error type of this RPC api.
pub enum Error {
    /// Statement not found
    StatementNotFound,
    //// Aggregate Receipt not published yet
    ReceiptNotPublished,
    /// The call to runtime failed.
    RuntimeError,
    /// The transaction was not decodable.
    DecodeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::StatementNotFound => 1,
            Error::ReceiptNotPublished => 2,
            Error::RuntimeError => 3,
            Error::DecodeError => 4,
        }
    }
}

impl VKHashApiServer for VKHash {
    fn compute_vk_hash_ultraplonk(&self, vk: &Ultraplonk::vk) -> RpcResult<H256> {
        Ultraplonk::vk_hash(vk)
    }
}

// fn convert_attestation_error(e: PathRequestError) -> ErrorObjectOwned {
//     match e {
//         PathRequestError::NotFound(domain_id, id, h) => ErrorObject::owned(
//             Error::StatementNotFound.into(),
//             "Statement not found in this aggregation",
//             Some(format!(
//                 "Statement {h} not found in Storage for aggregation ({domain_id},{id})"
//             )),
//         ),
//         PathRequestError::ReceiptNotPublished(domain_id, id) => ErrorObject::owned(
//             Error::ReceiptNotPublished.into(),
//             "Receipt not published in this block",
//             Some(format!("Receipt ({domain_id},{id}) not published yet")),
//         ),
//     }
// }
