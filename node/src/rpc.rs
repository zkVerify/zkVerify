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

//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use pallet_ismp_rpc::{IsmpApiServer, IsmpRpcHandler};
use sc_client_api::{BlockBackend, ProofProvider};
use sc_consensus_babe::{BabeApi, BabeWorkerHandle};
use sc_consensus_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
pub use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_core::H256;
use sp_keystore::KeystorePtr;
use vk_hash::{VKHash, VKHashApiServer};
use zkv_runtime::{currency::Balance, opaque::Block, AccountId, BlockNumber, Hash, Nonce};

pub use sc_rpc_api::DenyUnsafe;

#[derive(Clone)]
pub struct BabeDeps {
    pub babe_worker_handle: BabeWorkerHandle<Block>,
    pub keystore: KeystorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies
    pub grandpa: GrandpaDeps<B>,
    // Backend used by the node.
    pub backend: Arc<B>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, SC, B>(
    deps: FullDeps<C, P, SC, B>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C: BlockBackend<Block>,
    C: ProofProvider<Block>,
    C::Api: BabeApi<Block>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: proof_of_existence_rpc::PoERuntimeApi<Block>,
    C::Api: aggregate_rpc::AggregateRuntimeApi<Block>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>,
    P: TransactionPool + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashingFor<Block>>,
{
    use aggregate_rpc::{Aggregate, AggregateApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use proof_of_existence_rpc::{PoE, PoEApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        select_chain,
        deny_unsafe,
        babe,
        grandpa,
        backend,
    } = deps;

    let BabeDeps {
        babe_worker_handle,
        keystore,
    } = babe;

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    module.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle.clone(),
            keystore,
            select_chain,
            deny_unsafe,
        )
        .into_rpc(),
    )?;
    module.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;
    module.merge(PoE::new(client.clone()).into_rpc())?;
    module.merge(Aggregate::new(client.clone()).into_rpc())?
    module.merge(IsmpRpcHandler::new(client, backend.clone())?.into_rpc())?;
    module.merge(VKHash::new().into_rpc())?;

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

    Ok(module)
}
