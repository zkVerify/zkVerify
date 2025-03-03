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

pub use aggregate_rpc_runtime_api::AggregateApi as AggregateRuntimeApi;
use aggregate_rpc_runtime_api::{MerkleProof, PathRequestError};

#[rpc(client, server)]
pub trait AggregateApi<BlockHash, ResponseType> {
    #[method(name = "aggregate_statementPath")]
    fn get_statement_path(
        &self,
        at: BlockHash,
        domain_id: u32,
        aggregation_id: u64,
        statement: H256,
    ) -> RpcResult<ResponseType>;
}

pub struct Aggregate<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> Aggregate<C, P> {
    // Creates a new instance of the Aggregate Rpc helper.
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

impl<C, Block> AggregateApiServer<<Block as BlockT>::Hash, MerkleProof> for Aggregate<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: AggregateRuntimeApi<Block>,
{
    fn get_statement_path(
        &self,
        at: Block::Hash,
        domain_id: u32,
        aggregation_id: u64,
        statement: H256,
    ) -> RpcResult<MerkleProof> {
        let api = self.client.runtime_api();

        fn map_err(error: impl ToString, desc: &'static str) -> ErrorObjectOwned {
            ErrorObject::owned(Error::RuntimeError.into(), desc, Some(error.to_string()))
        }

        api.get_statement_path(at, domain_id, aggregation_id, statement)
            .map_err(|e| map_err(e, "Unable to query dispatch info."))
            .and_then(|r| r.map_err(convert_attestation_error))
    }
}

fn convert_attestation_error(e: PathRequestError) -> ErrorObjectOwned {
    match e {
        PathRequestError::NotFound(domain_id, id, h) => ErrorObject::owned(
            Error::StatementNotFound.into(),
            "Statement not found in this aggregation",
            Some(format!(
                "Statement {h} not found in Storage for aggregation ({domain_id},{id})"
            )),
        ),
        PathRequestError::ReceiptNotPublished(domain_id, id) => ErrorObject::owned(
            Error::ReceiptNotPublished.into(),
            "Receipt not published in this block",
            Some(format!("Receipt ({domain_id},{id}) not published yet")),
        ),
    }
}
