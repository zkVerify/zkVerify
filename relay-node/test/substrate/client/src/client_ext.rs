// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Client extension for tests.

use sc_client_api::{backend::Finalizer, client::BlockBackend};
use sc_consensus::{BlockImport, BlockImportParams, ForkChoiceStrategy};
use sc_service::client::Client;
use sp_consensus::Error as ConsensusError;
use sp_runtime::{traits::Block as BlockT, Justification, Justifications};

pub use sp_consensus::BlockOrigin;

/// Extension trait for a test client.
pub trait ClientExt<Block: BlockT>: Sized {
    /// Finalize a block.
    fn finalize_block(
        &self,
        hash: Block::Hash,
        justification: Option<Justification>,
    ) -> sp_blockchain::Result<()>;

    /// Returns hash of the genesis block.
    fn genesis_hash(&self) -> <Block as BlockT>::Hash;
}

/// Extension trait for a test client around block importing.
#[async_trait::async_trait]
pub trait ClientBlockImportExt<Block: BlockT>: Sized {
    /// Import block to the chain. No finality.
    async fn import(&self, origin: BlockOrigin, block: Block) -> Result<(), ConsensusError>;

    /// Import a block and make it our best block if possible.
    async fn import_as_best(&self, origin: BlockOrigin, block: Block)
        -> Result<(), ConsensusError>;

    /// Import a block and finalize it.
    async fn import_as_final(
        &self,
        origin: BlockOrigin,
        block: Block,
    ) -> Result<(), ConsensusError>;

    /// Import block with justification(s), finalizes block.
    async fn import_justified(
        &self,
        origin: BlockOrigin,
        block: Block,
        justifications: Justifications,
    ) -> Result<(), ConsensusError>;
}

impl<B, E, RA, Block> ClientExt<Block> for Client<B, E, Block, RA>
where
    B: sc_client_api::backend::Backend<Block>,
    E: sc_client_api::CallExecutor<Block> + sc_executor::RuntimeVersionOf + 'static,
    Self: BlockImport<Block, Error = ConsensusError>,
    Block: BlockT,
{
    fn finalize_block(
        &self,
        hash: Block::Hash,
        justification: Option<Justification>,
    ) -> sp_blockchain::Result<()> {
        Finalizer::finalize_block(self, hash, justification, true)
    }

    fn genesis_hash(&self) -> <Block as BlockT>::Hash {
        self.block_hash(0u32.into()).unwrap().unwrap()
    }
}

/// This implementation is required, because of the weird api requirements around `BlockImport`.
#[async_trait::async_trait]
impl<Block: BlockT, T> ClientBlockImportExt<Block> for std::sync::Arc<T>
where
    for<'r> &'r T: BlockImport<Block, Error = ConsensusError>,
    T: Send + Sync,
{
    async fn import(&self, origin: BlockOrigin, block: Block) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_as_best(
        &self,
        origin: BlockOrigin,
        block: Block,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::Custom(true));

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_as_final(
        &self,
        origin: BlockOrigin,
        block: Block,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.finalized = true;
        import.fork_choice = Some(ForkChoiceStrategy::Custom(true));

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_justified(
        &self,
        origin: BlockOrigin,
        block: Block,
        justifications: Justifications,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.justifications = Some(justifications);
        import.body = Some(extrinsics);
        import.finalized = true;
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);

        BlockImport::import_block(self, import).await.map(|_| ())
    }
}

#[async_trait::async_trait]
impl<B, E, RA, Block: BlockT> ClientBlockImportExt<Block> for Client<B, E, Block, RA>
where
    Self: BlockImport<Block, Error = ConsensusError>,
    RA: Send + Sync,
    B: Send + Sync,
    E: Send + Sync,
{
    async fn import(&self, origin: BlockOrigin, block: Block) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_as_best(
        &self,
        origin: BlockOrigin,
        block: Block,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::Custom(true));

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_as_final(
        &self,
        origin: BlockOrigin,
        block: Block,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.body = Some(extrinsics);
        import.finalized = true;
        import.fork_choice = Some(ForkChoiceStrategy::Custom(true));

        BlockImport::import_block(self, import).await.map(|_| ())
    }

    async fn import_justified(
        &self,
        origin: BlockOrigin,
        block: Block,
        justifications: Justifications,
    ) -> Result<(), ConsensusError> {
        let (header, extrinsics) = block.deconstruct();
        let mut import = BlockImportParams::new(origin, header);
        import.justifications = Some(justifications);
        import.body = Some(extrinsics);
        import.finalized = true;
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);

        BlockImport::import_block(self, import).await.map(|_| ())
    }
}
