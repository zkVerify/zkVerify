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

use jsonrpsee::RpcModule;
use polkadot_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Nonce};
use sc_client_api::AuxStore;
use sc_consensus_grandpa::FinalityProofProvider;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::KeystorePtr;
use std::sync::Arc;
use substrate_frame_rpc_system as frame_rpc_system;

/// A type representing all RPC extensions.
pub type RpcExtension = RpcModule<()>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// A handle to the BABE worker for issuing requests.
    pub babe_worker_handle: sc_consensus_babe::BabeWorkerHandle<Block>,
    /// The keystore that manages the keys of the node.
    pub keystore: KeystorePtr,
}

/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: sc_consensus_grandpa::SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: sc_consensus_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: sc_consensus_grandpa::GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The [`SelectChain`] Strategy
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
    /// Backend used by the node.
    pub backend: Arc<B>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, B>(
    FullDeps {
        client,
        pool,
        select_chain,
        chain_spec,
        deny_unsafe,
        babe,
        grandpa,
        backend,
    }: FullDeps<C, P, SC, B>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
    C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: proof_of_existence_rpc::PoERuntimeApi<Block>,
    C::Api: aggregate_rpc::AggregateRuntimeApi<Block>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: sc_transaction_pool_api::TransactionPool + Sync + Send + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::StateBackend<sp_runtime::traits::HashingFor<Block>>,
{
    use aggregate_rpc::{Aggregate, AggregateApiServer};
    use frame_rpc_system::{System, SystemApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use proof_of_existence_rpc::{PoE, PoEApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use sc_rpc_spec_v2::chain_spec::{ChainSpec, ChainSpecApiServer};
    use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};

    let mut io = RpcModule::new(());
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

    let chain_name = chain_spec.name().to_string();
    let genesis_hash = client
        .hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let properties = chain_spec.properties();

    io.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;
    io.merge(StateMigration::new(client.clone(), backend.clone(), deny_unsafe).into_rpc())?;
    io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    io.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle.clone(),
            keystore,
            select_chain,
            deny_unsafe,
        )
        .into_rpc(),
    )?;
    io.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;
    io.merge(PoE::new(client.clone()).into_rpc())?;
    io.merge(Aggregate::new(client).into_rpc())?;

    Ok(io)
}
