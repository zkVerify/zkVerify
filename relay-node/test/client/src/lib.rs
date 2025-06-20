// Copyright 2024, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

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

//! A Polkadot test client.
//!
//! This test client is using the Polkadot test runtime.

mod block_builder;

use polkadot_primitives::Block;
use sp_runtime::BuildStorage;
use std::sync::Arc;

pub use block_builder::*;
pub use substrate_test_client::*;
pub use test_runtime as runtime;
pub use test_service::{construct_extrinsic, construct_transfer_extrinsic, Client, FullBackend};

/// Test client executor.
pub type Executor = client::LocalCallExecutor<Block, FullBackend, test_service::WasmExecutor>;

/// Test client builder for Polkadot.
pub type TestClientBuilder =
    substrate_test_client::TestClientBuilder<Block, Executor, FullBackend, GenesisParameters>;

/// `LongestChain` type for the test runtime/client.
pub type LongestChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Parameters of test-client builder with test-runtime.
#[derive(Default)]
pub struct GenesisParameters;

impl substrate_test_client::GenesisInit for GenesisParameters {
    fn genesis_storage(&self) -> Storage {
        test_service::chain_spec::polkadot_local_testnet_config()
            .build_storage()
            .expect("Builds test runtime genesis storage")
    }
}

/// A `test-runtime` extensions to `TestClientBuilder`.
pub trait TestClientBuilderExt: Sized {
    /// Build the test client.
    fn build(self) -> Client {
        self.build_with_longest_chain().0
    }

    /// Build the test client and longest chain selector.
    fn build_with_longest_chain(self) -> (Client, LongestChain);
}

impl TestClientBuilderExt for TestClientBuilder {
    fn build_with_longest_chain(self) -> (Client, LongestChain) {
        let executor = test_service::WasmExecutor::builder().build();
        let executor = client::LocalCallExecutor::new(
            self.backend().clone(),
            executor.clone(),
            Default::default(),
            ExecutionExtensions::new(Default::default(), Arc::new(executor)),
        )
        .unwrap();

        self.build_with_executor(executor)
    }
}

/// A `TestClientBuilder` with default backend and executor.
pub trait DefaultTestClientBuilderExt: Sized {
    /// Create new `TestClientBuilder`
    fn new() -> Self;
}

impl DefaultTestClientBuilderExt for TestClientBuilder {
    fn new() -> Self {
        Self::with_default_backend()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_consensus::BlockOrigin;

    #[test]
    fn ensure_test_client_can_build_and_import_block() {
        let client = TestClientBuilder::new().build();

        let block_builder = client.init_polkadot_block_builder();
        let block = block_builder.build().expect("Finalizes the block").block;

        futures::executor::block_on(client.import(BlockOrigin::Own, block))
            .expect("Imports the block");
    }

    #[test]
    fn ensure_test_client_can_push_extrinsic() {
        let client = TestClientBuilder::new().build();

        let transfer = construct_transfer_extrinsic(
            &client,
            sp_keyring::Sr25519Keyring::Alice,
            sp_keyring::Sr25519Keyring::Bob,
            1000,
        );
        let mut block_builder = client.init_polkadot_block_builder();
        block_builder
            .push_polkadot_extrinsic(transfer)
            .expect("Pushes extrinsic");

        let block = block_builder.build().expect("Finalizes the block").block;

        futures::executor::block_on(client.import(BlockOrigin::Own, block))
            .expect("Imports the block");
    }
}
