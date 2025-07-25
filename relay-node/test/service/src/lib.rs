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

//! zkVerify test service only.

#![warn(missing_docs)]

pub mod chain_spec;

pub use chain_spec::*;
use futures::{future::Future, stream::StreamExt};
use polkadot_node_primitives::{CollationGenerationConfig, CollatorFn};
use polkadot_node_subsystem::messages::{CollationGenerationMessage, CollatorProtocolMessage};
use polkadot_overseer::Handle;
use polkadot_primitives::{Balance, CollatorPair, HeadData, Id as ParaId, ValidationCode};
use polkadot_runtime_common::BlockHashCount;
use polkadot_runtime_parachains::paras::{ParaGenesisArgs, ParaKind};
use service::{Error, FullClient, FullWasmExecutor, IsParachainNode, NewFull, PrometheusConfig};
use test_runtime::{
    ParasCall, ParasSudoWrapperCall, Runtime, SignedPayload, SudoCall, UncheckedExtrinsic, VERSION,
};

pub use service::rpc;

use sc_chain_spec::ChainSpec;
use sc_client_api::BlockchainEvents;
use sc_network::{
    config::{NetworkConfiguration, TransportConfig},
    multiaddr, NetworkStateInfo,
};
use sc_service::{
    config::{
        DatabaseSource, ExecutorConfiguration, KeystoreConfig, MultiaddrWithPeerId,
        RpcBatchRequestConfig, RpcConfiguration, WasmExecutionMethod,
        WasmtimeInstantiationStrategy,
    },
    BasePath, BlocksPruning, Configuration, Role, RpcHandlers, TaskManager,
};
use sp_arithmetic::traits::SaturatedConversion;
use sp_blockchain::HeaderBackend;
use sp_keyring::Sr25519Keyring;
use sp_runtime::{codec::Encode, generic, traits::IdentifyAccount, MultiSigner};
use sp_state_machine::BasicExternalities;
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use substrate_test_client::{
    BlockchainEventsExt, RpcHandlersExt, RpcTransactionError, RpcTransactionOutput,
};

/// The client type being used by the test service.
pub type Client = FullClient;

/// WasmExecutor type.
pub type WasmExecutor = FullWasmExecutor;

pub use service::FullBackend;

/// Create a new full node.
#[sc_tracing::logging::prefix_logs_with(config.network.node_name.as_str())]
pub fn new_full(
    config: Configuration,
    is_parachain_node: IsParachainNode,
    workers_path: Option<PathBuf>,
) -> Result<NewFull, Error> {
    let workers_path = Some(workers_path.unwrap_or_else(get_relative_workers_path_for_test));

    service::new_full::<_, sc_network::NetworkWorker<_, _>>(
        config,
        service::NewFullParams {
            is_parachain_node,
            force_authoring_backoff: false,
            telemetry_worker_handle: None,
            node_version: None,
            secure_validator_mode: false,
            workers_path,
            workers_names: None,
            overseer_gen: service::ValidatorOverseerGen,
            overseer_message_channel_capacity_override: None,
            malus_finality_delay: None,
            hwbench: None,
            execute_workers_max_num: None,
            prepare_workers_hard_max_num: None,
            prepare_workers_soft_max_num: None,
            enable_approval_voting_parallel: false,
        },
    )
}

fn get_relative_workers_path_for_test() -> PathBuf {
    // If no explicit worker path is passed in, we need to specify it ourselves as test binaries
    // are in the "deps/" directory, one level below where the worker binaries are generated.
    let mut exe_path = std::env::current_exe()
        .expect("for test purposes it's reasonable to expect that this will not fail");
    let _ = exe_path.pop();
    let _ = exe_path.pop();
    exe_path
}

/// Returns a prometheus config usable for testing.
pub fn test_prometheus_config(port: u16) -> PrometheusConfig {
    PrometheusConfig::new_with_default_registry(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port),
        "test-chain".to_string(),
    )
}

/// Create a zkVerify `Configuration`.
///
/// By default an in-memory socket will be used, therefore you need to provide boot
/// nodes if you want the future node to be connected to other nodes.
///
/// The `storage_update_func` function will be executed in an externalities provided environment
/// and can be used to make adjustments to the runtime genesis storage.
pub fn node_config(
    storage_update_func: impl Fn(),
    tokio_handle: tokio::runtime::Handle,
    key: Sr25519Keyring,
    boot_nodes: Vec<MultiaddrWithPeerId>,
    is_validator: bool,
) -> Configuration {
    let base_path = BasePath::new_temp_dir().expect("could not create temporary directory");
    let root = base_path.path().join(key.to_string());
    let role = if is_validator {
        Role::Authority
    } else {
        Role::Full
    };
    let key_seed = key.to_seed();
    let mut spec = polkadot_local_testnet_config();
    let mut storage = spec
        .as_storage_builder()
        .build_storage()
        .expect("could not build storage");

    BasicExternalities::execute_with_storage(&mut storage, storage_update_func);
    spec.set_storage(storage);

    let mut network_config = NetworkConfiguration::new(
        key_seed.to_string(),
        "network/test/0.1",
        Default::default(),
        None,
    );

    network_config.boot_nodes = boot_nodes;

    network_config.allow_non_globals_in_dht = true;

    let addr: multiaddr::Multiaddr = multiaddr::Protocol::Memory(rand::random()).into();
    network_config.listen_addresses.push(addr.clone());

    network_config.public_addresses.push(addr);

    network_config.transport = TransportConfig::MemoryOnly;

    Configuration {
        impl_name: "polkadot-test-node".to_string(),
        impl_version: "0.1".to_string(),
        role,
        tokio_handle,
        transaction_pool: Default::default(),
        network: network_config,
        keystore: KeystoreConfig::InMemory,
        database: DatabaseSource::RocksDb {
            path: root.join("db"),
            cache_size: 128,
        },
        trie_cache_maximum_size: Some(64 * 1024 * 1024),
        state_pruning: Default::default(),
        blocks_pruning: BlocksPruning::KeepFinalized,
        chain_spec: Box::new(spec),
        executor: ExecutorConfiguration {
            wasm_method: WasmExecutionMethod::Compiled {
                instantiation_strategy: WasmtimeInstantiationStrategy::PoolingCopyOnWrite,
            },
            ..ExecutorConfiguration::default()
        },
        wasm_runtime_overrides: Default::default(),
        rpc: RpcConfiguration {
            addr: Default::default(),
            max_request_size: Default::default(),
            max_response_size: Default::default(),
            max_connections: Default::default(),
            cors: None,
            methods: Default::default(),
            id_provider: None,
            max_subs_per_conn: Default::default(),
            port: 9944,
            message_buffer_capacity: Default::default(),
            batch_config: RpcBatchRequestConfig::Unlimited,
            rate_limit: None,
            rate_limit_whitelisted_ips: Default::default(),
            rate_limit_trust_proxy_headers: Default::default(),
        },
        prometheus_config: None,
        telemetry_endpoints: None,
        offchain_worker: Default::default(),
        force_authoring: false,
        disable_grandpa: false,
        dev_key_seed: Some(key_seed),
        tracing_targets: None,
        tracing_receiver: Default::default(),
        announce_block: true,
        data_path: root,
        base_path,
    }
}

/// Run a test validator node that uses the test runtime and specified `config`.
pub fn run_validator_node(
    config: Configuration,
    worker_program_path: Option<PathBuf>,
) -> PolkadotTestNode {
    let multiaddr = config.network.listen_addresses[0].clone();
    let NewFull {
        task_manager,
        client,
        network,
        rpc_handlers,
        overseer_handle,
        ..
    } = new_full(config, IsParachainNode::No, worker_program_path)
        .expect("could not create zkVerify test service");

    let overseer_handle = overseer_handle.expect("test node must have an overseer handle");
    let peer_id = network.local_peer_id();
    let addr = MultiaddrWithPeerId { multiaddr, peer_id };

    PolkadotTestNode {
        task_manager,
        client,
        overseer_handle,
        addr,
        rpc_handlers,
    }
}

/// Run a test collator node that uses the test runtime.
///
/// The node will be using an in-memory socket, therefore you need to provide boot nodes if you
/// want it to be connected to other nodes.
///
/// The `storage_update_func` function will be executed in an externalities provided environment
/// and can be used to make adjustments to the runtime genesis storage.
///
/// # Note
///
/// The collator functionality still needs to be registered at the node! This can be done using
/// [`PolkadotTestNode::register_collator`].
pub fn run_collator_node(
    tokio_handle: tokio::runtime::Handle,
    key: Sr25519Keyring,
    storage_update_func: impl Fn(),
    boot_nodes: Vec<MultiaddrWithPeerId>,
    collator_pair: CollatorPair,
) -> PolkadotTestNode {
    let config = node_config(storage_update_func, tokio_handle, key, boot_nodes, false);
    let multiaddr = config.network.listen_addresses[0].clone();
    let NewFull {
        task_manager,
        client,
        network,
        rpc_handlers,
        overseer_handle,
        ..
    } = new_full(
        config,
        IsParachainNode::Collator(Box::new(collator_pair)),
        None,
    )
    .expect("could not create zkVerify test service");

    let overseer_handle = overseer_handle.expect("test node must have an overseer handle");
    let peer_id = network.local_peer_id();
    let addr = MultiaddrWithPeerId { multiaddr, peer_id };

    PolkadotTestNode {
        task_manager,
        client,
        overseer_handle,
        addr,
        rpc_handlers,
    }
}

/// A zkVerify test node instance used for testing.
pub struct PolkadotTestNode {
    /// `TaskManager`'s instance.
    pub task_manager: TaskManager,
    /// Client's instance.
    pub client: Arc<Client>,
    /// A handle to Overseer.
    pub overseer_handle: Handle,
    /// The `MultiaddrWithPeerId` to this node. This is useful if you want to pass it as "boot
    /// node" to other nodes.
    pub addr: MultiaddrWithPeerId,
    /// `RPCHandlers` to make RPC queries.
    pub rpc_handlers: RpcHandlers,
}

impl PolkadotTestNode {
    /// Send a sudo call to this node.
    async fn send_sudo(
        &self,
        call: impl Into<test_runtime::RuntimeCall>,
        caller: Sr25519Keyring,
        nonce: u32,
    ) -> Result<(), RpcTransactionError> {
        let sudo = SudoCall::sudo {
            call: Box::new(call.into()),
        };

        let extrinsic = construct_extrinsic(&self.client, sudo, caller, nonce);
        self.rpc_handlers
            .send_transaction(extrinsic.into())
            .await
            .map(drop)
    }

    /// Send an extrinsic to this node.
    pub async fn send_extrinsic(
        &self,
        function: impl Into<test_runtime::RuntimeCall>,
        caller: Sr25519Keyring,
    ) -> Result<RpcTransactionOutput, RpcTransactionError> {
        let extrinsic = construct_extrinsic(&self.client, function, caller, 0);

        self.rpc_handlers.send_transaction(extrinsic.into()).await
    }

    /// Register a parachain at this relay chain.
    pub async fn register_parachain(
        &self,
        id: ParaId,
        validation_code: impl Into<ValidationCode>,
        genesis_head: impl Into<HeadData>,
    ) -> Result<(), RpcTransactionError> {
        let validation_code: ValidationCode = validation_code.into();
        let call = ParasSudoWrapperCall::sudo_schedule_para_initialize {
            id,
            genesis: ParaGenesisArgs {
                genesis_head: genesis_head.into(),
                validation_code: validation_code.clone(),
                para_kind: ParaKind::Parachain,
            },
        };

        self.send_sudo(call, Sr25519Keyring::Alice, 0).await?;

        // Bypass pvf-checking.
        let call = ParasCall::add_trusted_validation_code { validation_code };
        self.send_sudo(call, Sr25519Keyring::Alice, 1).await
    }

    /// Wait for `count` blocks to be imported in the node and then exit. This function will not
    /// return if no blocks are ever created, thus you should restrict the maximum amount of time of
    /// the test execution.
    pub fn wait_for_blocks(&self, count: usize) -> impl Future<Output = ()> {
        self.client.wait_for_blocks(count)
    }

    /// Wait for `count` blocks to be finalized and then exit. Similarly with `wait_for_blocks` this
    /// function will not return if no block are ever finalized.
    pub async fn wait_for_finalized_blocks(&self, count: usize) {
        let mut import_notification_stream = self.client.finality_notification_stream();
        let mut blocks = HashSet::new();

        while let Some(notification) = import_notification_stream.next().await {
            blocks.insert(notification.hash);
            if blocks.len() == count {
                break;
            }
        }
    }

    /// Register the collator functionality in the overseer of this node.
    pub async fn register_collator(
        &mut self,
        collator_key: CollatorPair,
        para_id: ParaId,
        collator: CollatorFn,
    ) {
        let config = CollationGenerationConfig {
            key: collator_key,
            collator: Some(collator),
            para_id,
        };

        self.overseer_handle
            .send_msg(CollationGenerationMessage::Initialize(config), "Collator")
            .await;

        self.overseer_handle
            .send_msg(CollatorProtocolMessage::CollateOn(para_id), "Collator")
            .await;
    }
}

/// Construct an extrinsic that can be applied to the test runtime.
pub fn construct_extrinsic(
    client: &Client,
    function: impl Into<test_runtime::RuntimeCall>,
    caller: Sr25519Keyring,
    nonce: u32,
) -> UncheckedExtrinsic {
    let function = function.into();
    let current_block_hash = client.info().best_hash;
    let current_block = client.info().best_number.saturated_into();
    let genesis_block = client.hash(0).unwrap().unwrap();
    let period = BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let tip = 0;
    let tx_ext: test_runtime::TxExtension = (
        frame_system::CheckNonZeroSender::<Runtime>::new(),
        frame_system::CheckSpecVersion::<Runtime>::new(),
        frame_system::CheckTxVersion::<Runtime>::new(),
        frame_system::CheckGenesis::<Runtime>::new(),
        frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
        frame_system::CheckNonce::<Runtime>::from(nonce),
        frame_system::CheckWeight::<Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
    );
    let raw_payload = SignedPayload::from_raw(
        function.clone(),
        tx_ext.clone(),
        (
            (),
            VERSION.spec_version,
            VERSION.transaction_version,
            genesis_block,
            current_block_hash,
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| caller.sign(e));
    UncheckedExtrinsic::new_signed(
        function.clone(),
        test_runtime::Address::Id(caller.public().into()),
        polkadot_primitives::Signature::Sr25519(signature),
        tx_ext.clone(),
    )
}

/// Construct a transfer extrinsic.
pub fn construct_transfer_extrinsic(
    client: &Client,
    origin: sp_keyring::AccountKeyring,
    dest: sp_keyring::AccountKeyring,
    value: Balance,
) -> UncheckedExtrinsic {
    let function =
        test_runtime::RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: MultiSigner::from(dest.public()).into_account().into(),
            value,
        });

    construct_extrinsic(client, function, origin, 0)
}
