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

//! zkVerify service. Specialized wrapper over substrate service.

#![deny(unused_results)]
#![allow(clippy::type_complexity)]

pub mod benchmarking;
pub mod chain_spec;
mod grandpa_support;
mod parachains_db;
mod relay_chain_selection;
pub mod rpc;

#[cfg(feature = "full-node")]
pub mod builder;
#[cfg(feature = "full-node")]
pub mod overseer;
#[cfg(feature = "full-node")]
pub mod workers;

#[cfg(feature = "full-node")]
pub use self::overseer::{
    CollatorOverseerGen, ExtendedOverseerGenArgs, OverseerGen, OverseerGenArgs,
    ValidatorOverseerGen,
};

pub use sc_consensus_babe as babe;

#[cfg(test)]
mod tests;

#[cfg(feature = "full-node")]
use {
    polkadot_node_core_approval_voting::self as approval_voting_subsystem,
    polkadot_node_core_av_store::Config as AvailabilityConfig,
    polkadot_node_core_chain_selection::self as chain_selection_subsystem,
    sc_consensus_grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider},
    sc_transaction_pool_api::OffchainTransactionPoolFactory,
};

use polkadot_node_subsystem_util::database::Database;
use polkadot_overseer::SpawnGlue;

#[cfg(feature = "full-node")]
pub use {
    polkadot_overseer::{Handle, Overseer, OverseerConnector, OverseerHandle},
    polkadot_primitives::runtime_api::ParachainHost,
    relay_chain_selection::SelectRelayChain,
    sc_client_api::AuxStore,
    sp_authority_discovery::AuthorityDiscoveryApi,
    sp_blockchain::{HeaderBackend, HeaderMetadata},
    sp_consensus_babe::BabeApi,
    sp_consensus_babe::inherents::BabeCreateInherentDataProviders,
};

use std::{path::PathBuf, sync::Arc, time::Duration};

use prometheus_endpoint::Registry;
pub use sc_service as service;
#[cfg(feature = "full-node")]
use sc_service::KeystoreContainer;
use sc_service::{RpcHandlers, SpawnTaskHandle};
use sc_telemetry::TelemetryWorker;
#[cfg(feature = "full-node")]
use sc_telemetry::{Telemetry, TelemetryWorkerHandle};
use telemetry as sc_telemetry;

pub use polkadot_primitives::{BlockId, BlockNumber, CollatorPair, Hash, Id as ParaId};
pub use sc_client_api::{Backend, CallExecutor};
pub use sc_consensus::{BlockImport, LongestChain};
pub use sc_executor::NativeExecutionDispatch;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
pub use sc_service::{
    config::{DatabaseSource, PrometheusConfig},
    ChainSpec, Configuration, Error as SubstrateServiceError, PruningMode, Role, TFullBackend,
    TFullCallExecutor, TFullClient, TaskManager, TransactionPoolOptions,
};
pub use sp_api::{ApiRef, ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi};
pub use sp_consensus::{Proposal, SelectChain};
use sp_core::crypto::{default_ss58_version, Ss58AddressFormat};
pub use sp_runtime::{
    generic,
    traits::{self as runtime_traits, BlakeTwo256, Block as BlockT, Header as HeaderT, NumberFor},
};
pub use zkv_runtime::{self, opaque::Block, RuntimeApi};

#[cfg(feature = "full-node")]
pub type FullBackend = sc_service::TFullBackend<Block>;

pub type FullWasmExecutor = WasmExecutor<(
    sp_io::SubstrateHostFunctions,
    native::HLNativeHostFunctions,
    frame_benchmarking::benchmarking::HostFunctions,
)>;

#[cfg(feature = "full-node")]
pub type FullClient = sc_service::TFullClient<Block, RuntimeApi, FullWasmExecutor>;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Provides the header and block number for a hash.
///
/// Decouples `sc_client_api::Backend` and `sp_blockchain::HeaderBackend`.
pub trait HeaderProvider<Block, Error = sp_blockchain::Error>: Send + Sync + 'static
where
    Block: BlockT,
    Error: std::fmt::Debug + Send + Sync + 'static,
{
    /// Obtain the header for a hash.
    fn header(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> Result<Option<<Block as BlockT>::Header>, Error>;
    /// Obtain the block number for a hash.
    fn number(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> Result<Option<<<Block as BlockT>::Header as HeaderT>::Number>, Error>;
}

impl<Block, T> HeaderProvider<Block> for T
where
    Block: BlockT,
    T: sp_blockchain::HeaderBackend<Block> + 'static,
{
    fn header(
        &self,
        hash: Block::Hash,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Header>> {
        <Self as sp_blockchain::HeaderBackend<Block>>::header(self, hash)
    }
    fn number(
        &self,
        hash: Block::Hash,
    ) -> sp_blockchain::Result<Option<<<Block as BlockT>::Header as HeaderT>::Number>> {
        <Self as sp_blockchain::HeaderBackend<Block>>::number(self, hash)
    }
}

/// Decoupling the provider.
///
/// Mandated since `trait HeaderProvider` can only be
/// implemented once for a generic `T`.
pub trait HeaderProviderProvider<Block>: Send + Sync + 'static
where
    Block: BlockT,
{
    type Provider: HeaderProvider<Block> + 'static;

    fn header_provider(&self) -> &Self::Provider;
}

impl<Block, T> HeaderProviderProvider<Block> for T
where
    Block: BlockT,
    T: sc_client_api::Backend<Block> + 'static,
{
    type Provider = <T as sc_client_api::Backend<Block>>::Blockchain;

    fn header_provider(&self) -> &Self::Provider {
        self.blockchain()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sub(#[from] SubstrateServiceError),

    #[error(transparent)]
    Blockchain(#[from] sp_blockchain::Error),

    #[error(transparent)]
    Consensus(#[from] sp_consensus::Error),

    #[error("Failed to create an overseer")]
    Overseer(#[from] polkadot_overseer::SubsystemError),

    #[error(transparent)]
    Prometheus(#[from] prometheus_endpoint::PrometheusError),

    #[error(transparent)]
    Telemetry(#[from] sc_telemetry::Error),

    #[error("Authorities require the real overseer implementation")]
    AuthoritiesRequireRealOverseer,

    #[cfg(feature = "full-node")]
    #[error("Creating a custom database is required for validators")]
    DatabasePathRequired,

    #[cfg(feature = "full-node")]
    #[error("Worker binaries not executable, prepare binary: {prep_worker_path:?}, execute binary: {exec_worker_path:?}")]
    InvalidWorkerBinaries {
        prep_worker_path: PathBuf,
        exec_worker_path: PathBuf,
    },

    #[cfg(feature = "full-node")]
    #[error("Worker binaries could not be found, make sure polkadot was built and installed correctly. Please see the readme for the latest instructions (https://github.com/paritytech/polkadot-sdk/tree/master/polkadot). If you ran with `cargo run`, please run `cargo build` first. Searched given workers path ({given_workers_path:?}), polkadot binary path ({current_exe_path:?}), and lib path (/usr/lib/polkadot), workers names: {workers_names:?}")]
    MissingWorkerBinaries {
        given_workers_path: Option<PathBuf>,
        current_exe_path: PathBuf,
        workers_names: Option<(String, String)>,
    },

    #[cfg(feature = "full-node")]
    #[error("Version of worker binary ({worker_version}) is different from node version ({node_version}), worker_path: {worker_path}. If you ran with `cargo run`, please run `cargo build` first, otherwise try to `cargo clean`. TESTING ONLY: this check can be disabled with --disable-worker-version-check")]
    WorkerBinaryVersionMismatch {
        worker_version: String,
        node_version: String,
        worker_path: PathBuf,
    },
}

/// Identifies the variant of the chain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    /// zkVerify Testnet.
    Volta,
    /// zkVerify Mainnet.
    ZkVerify,
    /// Unknown chain?
    Unknown,
}

impl Chain {
    pub fn ss58_format(&self) -> Ss58AddressFormat {
        match self {
            Chain::Volta => zkv_runtime::SS58VoltaPrefix::get().into(),
            Chain::ZkVerify => zkv_runtime::SS58ZkvPrefix::get().into(),
            Chain::Unknown => default_ss58_version(),
        }
    }
}

/// Can be called for a `Configuration` to identify which network the configuration targets.
pub trait IdentifyVariant {
    /// Returns true if this is a configuration for the `Volta` network.
    fn is_volta(&self) -> bool;

    /// Returns true if this is a configuration for the `zkVerify` network.
    fn is_zkverify(&self) -> bool;

    /// Returns true if this configuration is for a development network.
    fn is_dev(&self) -> bool;

    /// Identifies the variant of the chain.
    fn identify_chain(&self) -> Chain;
}

impl IdentifyVariant for &dyn ChainSpec {
    fn is_volta(&self) -> bool {
        self.id().starts_with("nh_testnet")
            || self.id().starts_with("volta")
            || self.id().starts_with("zkv_testnet")
    }

    fn is_zkverify(&self) -> bool {
        self.id().starts_with("zkv_mainnet")
    }

    fn is_dev(&self) -> bool {
        self.id().ends_with("dev")
    }

    fn identify_chain(&self) -> Chain {
        if self.is_volta() {
            Chain::Volta
        } else if self.is_zkverify() {
            Chain::ZkVerify
        } else {
            Chain::Unknown
        }
    }
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_volta(&self) -> bool {
        self.as_ref().is_volta()
    }

    fn is_zkverify(&self) -> bool {
        self.as_ref().is_zkverify()
    }

    fn is_dev(&self) -> bool {
        self.as_ref().is_dev()
    }

    fn identify_chain(&self) -> Chain {
        self.as_ref().identify_chain()
    }
}

#[cfg(feature = "full-node")]
pub fn open_database(db_source: &DatabaseSource) -> Result<Arc<dyn Database>, Error> {
    let parachains_db = match db_source {
        DatabaseSource::RocksDb { path, .. } => parachains_db::open_creating_rocksdb(
            path.clone(),
            parachains_db::CacheSizes::default(),
        )?,
        DatabaseSource::ParityDb { path, .. } => parachains_db::open_creating_paritydb(
            path.parent().ok_or(Error::DatabasePathRequired)?.into(),
            parachains_db::CacheSizes::default(),
        )?,
        DatabaseSource::Auto {
            paritydb_path,
            rocksdb_path,
            ..
        } => {
            if paritydb_path.is_dir() && paritydb_path.exists() {
                parachains_db::open_creating_paritydb(
                    paritydb_path
                        .parent()
                        .ok_or(Error::DatabasePathRequired)?
                        .into(),
                    parachains_db::CacheSizes::default(),
                )?
            } else {
                parachains_db::open_creating_rocksdb(
                    rocksdb_path.clone(),
                    parachains_db::CacheSizes::default(),
                )?
            }
        }
        DatabaseSource::Custom { .. } => {
            unimplemented!("No polkadot subsystem db for custom source.");
        }
    };
    Ok(parachains_db)
}

#[cfg(feature = "full-node")]
type FullSelectChain = relay_chain_selection::SelectRelayChain<FullBackend>;
#[cfg(feature = "full-node")]
type FullGrandpaBlockImport<ChainSelection = FullSelectChain> =
    sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, ChainSelection>;
#[cfg(feature = "full-node")]
struct Basics {
    task_manager: TaskManager,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    keystore_container: KeystoreContainer,
    telemetry: Option<Telemetry>,
}

#[cfg(feature = "full-node")]
fn new_partial_basics(
    config: &mut Configuration,
    telemetry_worker_handle: Option<TelemetryWorkerHandle>,
) -> Result<Basics, Error> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(move |endpoints| -> Result<_, sc_telemetry::Error> {
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

#[cfg(feature = "full-node")]
fn new_partial<ChainSelection>(
    config: &mut Configuration,
    Basics {
        task_manager,
        backend,
        client,
        keystore_container,
        telemetry,
    }: Basics,
    select_chain: ChainSelection,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        ChainSelection,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
        (
            impl Fn(rpc::SubscriptionTaskExecutor) -> Result<rpc::RpcExtension, SubstrateServiceError>,
            (
                babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport<ChainSelection>, BabeCreateInherentDataProviders<Block>, ChainSelection>,
                sc_consensus_grandpa::LinkHalf<Block, FullClient, ChainSelection>,
                babe::BabeLink<Block>,
            ),
            sc_consensus_grandpa::SharedVoterState,
            sp_consensus_babe::SlotDuration,
            Option<Telemetry>,
        ),
    >,
    Error,
>
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

        move |
              subscription_executor: rpc::SubscriptionTaskExecutor|
              -> Result<rpc::RpcExtension, sc_service::Error> {
            let deps = rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                babe: rpc::BabeDeps {
                    babe_worker_handle: babe_worker_handle.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: rpc::GrandpaDeps {
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
            rpc_extensions_builder,
            import_setup,
            rpc_setup,
            slot_duration,
            telemetry,
        ),
    })
}

#[cfg(feature = "full-node")]
pub struct NewFullParams<OverseerGenerator: OverseerGen> {
    pub is_parachain_node: IsParachainNode,
    /// Whether to enable the block authoring backoff on production networks
    /// where it isn't enabled by default.
    pub force_authoring_backoff: bool,
    pub telemetry_worker_handle: Option<TelemetryWorkerHandle>,
    /// The version of the node. TESTING ONLY: `None` can be passed to skip the node/worker version
    /// check, both on startup and in the workers.
    pub node_version: Option<String>,
    /// Whether the node is attempting to run as a secure validator.
    pub secure_validator_mode: bool,
    /// An optional path to a directory containing the workers.
    pub workers_path: Option<std::path::PathBuf>,
    /// Optional custom names for the prepare and execute workers.
    pub workers_names: Option<(String, String)>,
    /// An optional number of the maximum number of pvf execute workers.
    pub execute_workers_max_num: Option<usize>,
    /// An optional maximum number of pvf workers that can be spawned in the pvf prepare pool for
    /// tasks with the priority below critical.
    pub prepare_workers_soft_max_num: Option<usize>,
    /// An optional absolute number of pvf workers that can be spawned in the pvf prepare pool.
    pub prepare_workers_hard_max_num: Option<usize>,
    /// How long finalized data should be kept in the availability store (in hours).
    pub keep_finalized_for: Option<u32>,
    pub overseer_gen: OverseerGenerator,
    pub overseer_message_channel_capacity_override: Option<usize>,
    #[allow(dead_code)]
    pub malus_finality_delay: Option<u32>,
    pub hwbench: Option<sc_sysinfo::HwBench>,
    /// Set of invulnerable AH collator `PeerId`s.
    pub invulnerable_ah_collators: std::collections::HashSet<polkadot_node_network_protocol::PeerId>,
    /// Override for `HOLD_OFF_DURATION` constant.
    pub collator_protocol_hold_off: Option<Duration>,
}

#[cfg(feature = "full-node")]
pub struct NewFull {
    pub task_manager: TaskManager,
    pub client: Arc<FullClient>,
    pub overseer_handle: Option<Handle>,
    pub network: Arc<dyn sc_network::service::traits::NetworkService>,
    pub sync_service: Arc<sc_network_sync::SyncingService<Block>>,
    pub rpc_handlers: RpcHandlers,
    pub backend: Arc<FullBackend>,
}

/// Is this node running as in-process node for a parachain node?
#[cfg(feature = "full-node")]
#[derive(Clone)]
pub enum IsParachainNode {
    /// This node is running as in-process node for a parachain collator.
    Collator(CollatorPair),
    /// This node is running as in-process node for a parachain full node.
    FullNode,
    /// This node is not running as in-process node for a parachain node, aka a normal relay chain
    /// node.
    No,
}

#[cfg(feature = "full-node")]
impl std::fmt::Debug for IsParachainNode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use sp_core::Pair;
        match self {
            IsParachainNode::Collator(pair) => write!(fmt, "Collator({})", pair.public()),
            IsParachainNode::FullNode => write!(fmt, "FullNode"),
            IsParachainNode::No => write!(fmt, "No"),
        }
    }
}

#[cfg(feature = "full-node")]
impl IsParachainNode {
    /// Is this running alongside a collator?
    fn is_collator(&self) -> bool {
        matches!(self, Self::Collator(_))
    }

    /// Is this running alongside a full node?
    fn is_full_node(&self) -> bool {
        matches!(self, Self::FullNode)
    }

    /// Is this node running alongside a relay chain node?
    fn is_running_alongside_parachain_node(&self) -> bool {
        self.is_collator() || self.is_full_node()
    }
}

/// The number of hours to keep finalized data in the availability store for live networks.
const KEEP_FINALIZED_FOR_LIVE_NETWORKS: u32 = 25;

pub fn availability_config(keep_finalized_for: Option<u32>) -> AvailabilityConfig {
    AvailabilityConfig {
        col_data: parachains_db::REAL_COLUMNS.col_availability_data,
        col_meta: parachains_db::REAL_COLUMNS.col_availability_meta,
        keep_finalized_for: keep_finalized_for.unwrap_or(KEEP_FINALIZED_FOR_LIVE_NETWORKS),
    }
}

/// Create a new full node of arbitrary runtime and executor.
///
/// This is an advanced feature and not recommended for general use. Generally, `build_full` is
/// a better choice.
///
/// `workers_path` is used to get the path to the directory where auxiliary worker binaries reside.
/// If not specified, the main binary's directory is searched first, then `/usr/lib/polkadot` is
/// searched. If the path points to an executable rather then directory, that executable is used
/// both as preparation and execution worker (supposed to be used for tests only).
#[cfg(feature = "full-node")]
pub fn new_full<
    OverseerGenerator: OverseerGen,
    Network: sc_network::NetworkBackend<Block, <Block as BlockT>::Hash>,
>(
    config: Configuration,
    params: NewFullParams<OverseerGenerator>,
) -> Result<NewFull, Error> {
    builder::PolkadotServiceBuilder::<OverseerGenerator, Network>::new(config, params)?.build()
}

#[cfg(feature = "full-node")]
macro_rules! chain_ops {
    ($config:expr, $telemetry_worker_handle:expr) => {{
        let telemetry_worker_handle = $telemetry_worker_handle;
        let mut config = $config;
        let basics = new_partial_basics(config, telemetry_worker_handle)?;

        use ::sc_consensus::LongestChain;
        // use the longest chain selection, since there is no overseer available
        let chain_selection = LongestChain::new(basics.backend.clone());

        let service::PartialComponents {
            client,
            backend,
            import_queue,
            task_manager,
            ..
        } = new_partial::<LongestChain<_, Block>>(&mut config, basics, chain_selection)?;
        Ok((client, backend, import_queue, task_manager))
    }};
}

/// Builds a new object suitable for chain operations.
#[cfg(feature = "full-node")]
pub fn new_chain_ops(
    config: &mut Configuration,
) -> Result<
    (
        Arc<FullClient>,
        Arc<FullBackend>,
        sc_consensus::BasicQueue<Block>,
        TaskManager,
    ),
    Error,
> {
    config.keystore = service::config::KeystoreConfig::InMemory;

    chain_ops!(config, None)
}

/// Build a full node.
///
/// The actual "flavor", aka if it will use `ZkvTestnet` or something else is determined based on
/// [`IdentifyVariant`] using the chain spec.
#[cfg(feature = "full-node")]
pub fn build_full<OverseerGenerator: OverseerGen>(
    config: Configuration,
    params: NewFullParams<OverseerGenerator>,
) -> Result<NewFull, Error> {
    new_full::<_, sc_network::NetworkWorker<Block, Hash>>(config, params)
}

/// Reverts the node state down to at most the last finalized block.
///
/// In particular this reverts:
/// - `ApprovalVotingSubsystem` data in the parachains-db;
/// - `ChainSelectionSubsystem` data in the parachains-db;
/// - Low level Babe and Grandpa consensus data.
#[cfg(feature = "full-node")]
pub fn revert_backend(
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    blocks: BlockNumber,
    config: Configuration,
    task_handle: SpawnTaskHandle,
) -> Result<(), Error> {
    let best_number = client.info().best_number;
    let finalized = client.info().finalized_number;
    let revertible = blocks.min(best_number - finalized);

    if revertible == 0 {
        return Ok(());
    }

    let number = best_number - revertible;
    let hash = client.block_hash_from_id(&BlockId::Number(number))?.ok_or(
        sp_blockchain::Error::Backend(format!(
            "Unexpected hash lookup failure for block number: {number}"
        )),
    )?;

    let parachains_db = open_database(&config.database)
        .map_err(|err| sp_blockchain::Error::Backend(err.to_string()))?;

    revert_approval_voting(parachains_db.clone(), hash, task_handle)?;
    revert_chain_selection(parachains_db, hash)?;
    // Revert Substrate consensus related components
    babe::revert(client.clone(), backend, blocks)?;
    sc_consensus_grandpa::revert(client, blocks)?;

    Ok(())
}

fn revert_chain_selection(db: Arc<dyn Database>, hash: Hash) -> sp_blockchain::Result<()> {
    let config = chain_selection_subsystem::Config {
        col_data: parachains_db::REAL_COLUMNS.col_chain_selection_data,
        stagnant_check_interval: chain_selection_subsystem::StagnantCheckInterval::never(),
        stagnant_check_mode: chain_selection_subsystem::StagnantCheckMode::PruneOnly,
    };

    let chain_selection = chain_selection_subsystem::ChainSelectionSubsystem::new(config, db);

    chain_selection
        .revert_to(hash)
        .map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
}

fn revert_approval_voting(
    db: Arc<dyn Database>,
    hash: Hash,
    task_handle: SpawnTaskHandle,
) -> sp_blockchain::Result<()> {
    let config = approval_voting_subsystem::Config {
        col_approval_data: parachains_db::REAL_COLUMNS.col_approval_data,
        slot_duration_millis: Default::default(),
    };

    let approval_voting = approval_voting_subsystem::ApprovalVotingSubsystem::with_config(
        config,
        db,
        Arc::new(sc_keystore::LocalKeystore::in_memory()),
        Box::new(sp_consensus::NoNetwork),
        approval_voting_subsystem::Metrics::default(),
        Arc::new(SpawnGlue(task_handle)),
    );

    approval_voting
        .revert_to(hash)
        .map_err(|err| sp_blockchain::Error::Backend(err.to_string()))
}
