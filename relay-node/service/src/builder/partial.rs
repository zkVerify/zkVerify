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

//! Partial service builder types and functions.

use crate::{
    relay_chain_selection::SelectRelayChain,
    Block, Error, FullBackend, FullClient, GRANDPA_JUSTIFICATION_PERIOD,
};
use sc_consensus_grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_service::{Configuration, KeystoreContainer, TaskManager};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_consensus::SelectChain;
use sp_consensus_babe::inherents::BabeCreateInherentDataProviders;
use std::sync::Arc;
use telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use zkv_runtime::RuntimeApi;

use crate::babe;

// ── Type aliases ────────────────────────────────────────────────────────────

pub(crate) type FullSelectChain = SelectRelayChain<FullBackend>;
pub(crate) type FullGrandpaBlockImport<ChainSelection = FullSelectChain> =
    sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, ChainSelection>;

// ── Basics / new_partial_basics ─────────────────────────────────────────────

pub(crate) struct Basics {
    pub task_manager: TaskManager,
    pub client: Arc<FullClient>,
    pub backend: Arc<FullBackend>,
    pub keystore_container: KeystoreContainer,
    pub telemetry: Option<Telemetry>,
}

pub(crate) fn new_partial_basics(
    config: &mut Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
) -> Result<Basics, Error> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(move |endpoints| -> Result<_, telemetry::Error> {
            let (worker, mut worker_handle) = if let Some(worker_handle) = telemetry_worker_handle {
                (None, worker_handle)
            } else {
                let worker = TelemetryWorker::new(16)?;
                let worker_handle = worker.handle();
                (Some(worker), worker_handle)
            };
            let telemetry = worker_handle.new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let heap_pages = config
        .executor
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let executor = WasmExecutor::builder()
        .with_execution_method(config.executor.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.executor.max_runtime_instances)
        .with_runtime_cache_size(config.executor.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        if let Some(worker) = worker {
            task_manager.spawn_handle().spawn(
                "telemetry",
                Some("telemetry"),
                Box::pin(worker.run()),
            );
        }
        telemetry
    });

    Ok(Basics {
        task_manager,
        client,
        backend,
        keystore_container,
        telemetry,
    })
}

// ── new_partial ─────────────────────────────────────────────────────────────

/// Type alias for the partial components used by the service builder.
pub(crate) type ServicePartialComponents<ChainSelection> = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    ChainSelection,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
    (
        Box<
            dyn Fn(
                    crate::rpc::SubscriptionTaskExecutor,
                )
                    -> Result<crate::rpc::RpcExtension, crate::SubstrateServiceError>
                + Send,
        >,
        (
            babe::BabeBlockImport<
                Block,
                FullClient,
                FullGrandpaBlockImport<ChainSelection>,
                BabeCreateInherentDataProviders<Block>,
                ChainSelection,
            >,
            sc_consensus_grandpa::LinkHalf<Block, FullClient, ChainSelection>,
            babe::BabeLink<Block>,
        ),
        sc_consensus_grandpa::SharedVoterState,
        sp_consensus_babe::SlotDuration,
        Option<Telemetry>,
    ),
>;

pub(crate) fn new_partial<ChainSelection>(
    config: &mut Configuration,
    Basics {
        task_manager,
        backend,
        client,
        keystore_container,
        telemetry,
    }: Basics,
    select_chain: ChainSelection,
) -> Result<ServicePartialComponents<ChainSelection>, Error>
where
    ChainSelection: 'static + SelectChain<Block>,
{
    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let grandpa_hard_forks = Vec::new();

    let (grandpa_block_import, grandpa_link) =
        sc_consensus_grandpa::block_import_with_authority_set_hard_forks(
            client.clone(),
            GRANDPA_JUSTIFICATION_PERIOD,
            &(client.clone() as Arc<_>),
            select_chain.clone(),
            grandpa_hard_forks,
            telemetry.as_ref().map(|x| x.handle()),
        )?;
    let justification_import = grandpa_block_import.clone();

    let babe_config = babe::configuration(&*client)?;
    let slot_duration = babe_config.slot_duration();
    let create_inherent_data_providers: BabeCreateInherentDataProviders<Block> =
        Arc::new(move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot =
                sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );
            Ok((slot, timestamp))
        });
    let (block_import, babe_link) = sc_consensus_babe::block_import(
        babe_config.clone(),
        grandpa_block_import,
        client.clone(),
        create_inherent_data_providers,
        select_chain.clone(),
        OffchainTransactionPoolFactory::new(transaction_pool.clone()),
    )?;

    let (import_queue, babe_worker_handle) = sc_consensus_babe::import_queue(
        sc_consensus_babe::ImportQueueParams {
            link: babe_link.clone(),
            block_import: block_import.clone(),
            justification_import: Some(Box::new(justification_import)),
            client: client.clone(),
            slot_duration: babe_link.config().slot_duration(),
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        },
    )?;

    let justification_stream = grandpa_link.justification_stream();
    let shared_authority_set = grandpa_link.shared_authority_set().clone();
    let shared_voter_state = sc_consensus_grandpa::SharedVoterState::empty();
    let finality_proof_provider = GrandpaFinalityProofProvider::new_for_service(
        backend.clone(),
        Some(shared_authority_set.clone()),
    );

    let import_setup = (block_import, grandpa_link, babe_link);
    let rpc_setup = shared_voter_state.clone();

    let rpc_extensions_builder = {
        let client = client.clone();
        let keystore = keystore_container.keystore();
        let transaction_pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let chain_spec = config.chain_spec.cloned_box();
        let backend = backend.clone();

        move |subscription_executor: crate::rpc::SubscriptionTaskExecutor|
              -> Result<crate::rpc::RpcExtension, sc_service::Error> {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                babe: crate::rpc::BabeDeps {
                    babe_worker_handle: babe_worker_handle.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: crate::rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor: subscription_executor.clone(),
                    finality_provider: finality_proof_provider.clone(),
                },
                backend: backend.clone(),
            };

            crate::rpc::create_full(deps).map_err(Into::into)
        }
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        keystore_container,
        select_chain,
        import_queue,
        transaction_pool,
        other: (
            Box::new(rpc_extensions_builder),
            import_setup,
            rpc_setup,
            slot_duration,
            telemetry,
        ),
    })
}
