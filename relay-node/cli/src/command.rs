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

use crate::cli::{Cli, Subcommand, NODE_VERSION};
use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory};
use futures::future::TryFutureExt;
use native::HLNativeHostFunctions;
use sc_cli::SubstrateCli;
use service::{
    self,
    benchmarking::{benchmark_inherent_data, RemarkBuilder, TransferKeepAliveBuilder},
    HeaderBackend, IdentifyVariant,
};
use sp_keyring::Sr25519Keyring;
use zkv_benchmarks::hardware::zkv_reference_hardware;

pub use crate::{error::Error, service::BlockId};
#[cfg(feature = "pyroscope")]
use pyroscope_pprofrs::{pprof_backend, PprofConfig};
#[cfg(feature = "pyroscope")]
use std::net::ToSocketAddrs;

type Result<T> = std::result::Result<T, Error>;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "ZkVerify Relay".into()
    }

    fn impl_version() -> String {
        let commit_hash = env!("SUBSTRATE_CLI_COMMIT_HASH");
        format!("{NODE_VERSION}-{commit_hash}")
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/zkVerify/zkVerify".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    fn executable_name() -> String {
        "zkv-relay".into()
    }

    fn load_spec(
        &self,
        id: &str,
    ) -> std::result::Result<Box<dyn sc_service::ChainSpec + 'static>, String> {
        Ok(match id {
            "dev" | "development" => Box::new(service::chain_spec::development_config()?),
            "local" => Box::new(service::chain_spec::local_config()?),
            "" | "test" | "testnet" => Box::new(service::chain_spec::ChainSpec::from_json_bytes(
                &include_bytes!("../chain-specs/zkverify_volta.json")[..],
            )?),
            "testnet_build" => Box::new(service::chain_spec::testnet_config()?),
            path => Box::new(service::chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

fn set_default_ss58_version(_spec: &dyn service::ChainSpec) {
    sp_core::crypto::set_default_ss58_version(zkv_runtime::SS58Prefix::get().into());
}

fn run_node_inner<F>(
    cli: Cli,
    overseer_gen: impl service::OverseerGen,
    maybe_malus_finality_delay: Option<u32>,
    logger_hook: F,
) -> Result<()>
where
    F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
{
    let runner = cli
        .create_runner_with_logger_hook::<_, _, F>(&cli.run.base, logger_hook)
        .map_err(Error::from)?;
    let chain_spec = &runner.config().chain_spec;

    set_default_ss58_version(chain_spec.as_ref());

    let node_version = if cli.run.disable_worker_version_check {
        None
    } else {
        Some(NODE_VERSION.to_string())
    };

    let secure_validator_mode = cli.run.base.validator && !cli.run.insecure_validator;

    runner.run_node_until_exit(move |config| async move {
        let hwbench = (!cli.run.no_hardware_benchmarks)
            .then_some(config.database.path().map(|database_path| {
                let _ = std::fs::create_dir_all(database_path);
                sc_sysinfo::gather_hwbench(Some(database_path), zkv_reference_hardware())
            }))
            .flatten();

        let database_source = config.database.clone();
        let task_manager = service::build_full(
            config,
            service::NewFullParams {
                is_parachain_node: service::IsParachainNode::No,
                force_authoring_backoff: cli.run.force_authoring_backoff,
                telemetry_worker_handle: None,
                node_version,
                secure_validator_mode,
                workers_path: cli.run.workers_path,
                workers_names: None,
                overseer_gen,
                overseer_message_channel_capacity_override: cli
                    .run
                    .overseer_channel_capacity_override,
                malus_finality_delay: maybe_malus_finality_delay,
                hwbench,
                execute_workers_max_num: None,
                prepare_workers_hard_max_num: None,
                prepare_workers_soft_max_num: None,
                enable_approval_voting_parallel: false,
            },
        )
        .map(|full| full.task_manager)?;

        if let Some(path) = database_source.path() {
            sc_storage_monitor::StorageMonitorService::try_spawn(
                cli.storage_monitor,
                path.to_path_buf(),
                &task_manager.spawn_essential_handle(),
            )?;
        }

        Ok(task_manager)
    })
}

/// Parses polkadot specific CLI arguments and run the service.
pub fn run() -> Result<()> {
    let cli: Cli = Cli::from_args();

    #[cfg(feature = "pyroscope")]
    let mut pyroscope_agent_maybe = if let Some(ref agent_addr) = cli.run.pyroscope_server {
        let address = agent_addr
            .to_socket_addrs()
            .map_err(Error::AddressResolutionFailure)?
            .next()
            .ok_or_else(|| Error::AddressResolutionMissing)?;
        // The pyroscope agent requires a `http://` prefix, so we just do that.
        let agent = pyro::PyroscopeAgent::builder(
            "http://".to_owned() + address.to_string().as_str(),
            "polkadot".to_owned(),
        )
        .backend(pprof_backend(PprofConfig::new().sample_rate(113)))
        .build()?;
        Some(agent.start()?)
    } else {
        None
    };

    #[cfg(not(feature = "pyroscope"))]
    if cli.run.pyroscope_server.is_some() {
        return Err(Error::PyroscopeNotCompiledIn);
    }

    match &cli.subcommand {
        None => run_node_inner(
            cli,
            service::ValidatorOverseerGen,
            None,
            polkadot_node_metrics::logger_hook(),
        ),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            Ok(runner.sync_run(|config| cmd.run(config.chain_spec, config.network))?)
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd).map_err(Error::SubstrateCli)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec.as_ref());

            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((
                    cmd.run(client, import_queue).map_err(Error::SubstrateCli),
                    task_manager,
                ))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec.as_ref());

            Ok(runner.async_run(|mut config| {
                let (client, _, _, task_manager) =
                    service::new_chain_ops(&mut config).map_err(Error::ZKVService)?;
                Ok((
                    cmd.run(client, config.database)
                        .map_err(Error::SubstrateCli),
                    task_manager,
                ))
            })?)
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec.as_ref());

            Ok(runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((
                    cmd.run(client, config.chain_spec)
                        .map_err(Error::SubstrateCli),
                    task_manager,
                ))
            })?)
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec.as_ref());

            Ok(runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
                Ok((
                    cmd.run(client, import_queue).map_err(Error::SubstrateCli),
                    task_manager,
                ))
            })?)
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            Ok(runner.sync_run(|config| cmd.run(config.database))?)
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            set_default_ss58_version(chain_spec.as_ref());

            Ok(runner.async_run(|mut config| {
                let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
                let task_handle = task_manager.spawn_handle();
                let aux_revert = Box::new(|client, backend, blocks| {
                    service::revert_backend(client, backend, blocks, config, task_handle).map_err(
                        |err| {
                            match err {
                                service::Error::Blockchain(err) => err.into(),
                                // Generic application-specific error.
                                err => sc_cli::Error::Application(err.into()),
                            }
                        },
                    )
                });
                Ok((
                    cmd.run(client, backend, Some(aux_revert))
                        .map_err(Error::SubstrateCli),
                    task_manager,
                ))
            })?)
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            match cmd {
                #[cfg(not(feature = "runtime-benchmarks"))]
                BenchmarkCmd::Storage(_) => {
                    return Err(sc_cli::Error::Input(
                        "Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
                            .into(),
                    )
                    .into())
                }
                #[cfg(feature = "runtime-benchmarks")]
                BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
                    let (client, backend, _, _) = service::new_chain_ops(&mut config)?;
                    let db = backend.expose_db();
                    let storage = backend.expose_storage();

                    cmd.run(config, client.clone(), db, storage)
                        .map_err(Error::SubstrateCli)
                }),
                BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config)?;

                    cmd.run(client.clone()).map_err(Error::SubstrateCli)
                }),
                // These commands are very similar and can be handled in nearly the same way.
                BenchmarkCmd::Extrinsic(_) | BenchmarkCmd::Overhead(_) => {
                    runner.sync_run(|mut config| {
                        let (client, _, _, _) = service::new_chain_ops(&mut config)?;
                        let header = client.header(client.info().genesis_hash).unwrap().unwrap();
                        let inherent_data = benchmark_inherent_data(header)
                            .map_err(|e| format!("generating inherent data: {e:?}"))?;
                        let remark_builder =
                            RemarkBuilder::new(client.clone(), config.chain_spec.identify_chain());

                        match cmd {
                            BenchmarkCmd::Extrinsic(cmd) => {
                                let tka_builder = TransferKeepAliveBuilder::new(
                                    client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    config.chain_spec.identify_chain(),
                                );

                                let ext_factory = ExtrinsicFactory(vec![
                                    Box::new(remark_builder),
                                    Box::new(tka_builder),
                                ]);

                                cmd.run(client.clone(), inherent_data, Vec::new(), &ext_factory)
                                    .map_err(Error::SubstrateCli)
                            }
                            BenchmarkCmd::Overhead(cmd) => cmd
                                .run(
                                    config.chain_spec.name().into(),
                                    client.clone(),
                                    inherent_data,
                                    Vec::new(),
                                    &remark_builder,
                                    false,
                                )
                                .map_err(Error::SubstrateCli),
                            _ => unreachable!("Ensured by the outside match; qed"),
                        }
                    })
                }
                BenchmarkCmd::Pallet(cmd) => {
                    set_default_ss58_version(chain_spec.as_ref());

                    if cfg!(feature = "runtime-benchmarks") {
                        runner.sync_run(|config| {
                            cmd.run_with_spec::<sp_runtime::traits::HashingFor<service::Block>, HLNativeHostFunctions>(Some(config.chain_spec))
                                .map_err(Error::SubstrateCli)
                        })
                    } else {
                        Err(sc_cli::Error::Input(
                            "Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                                .into(),
                        )
                        .into())
                    }
                }
                BenchmarkCmd::Machine(cmd) => runner.sync_run(|config| {
                    cmd.run(&config, zkv_reference_hardware().clone())
                        .map_err(Error::SubstrateCli)
                }),
                // NOTE: this allows the zkVerify client to leniently implement
                // new benchmark commands.
                #[allow(unreachable_patterns)]
                _ => Err(Error::CommandNotImplemented),
            }
        }
        Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            Ok(runner.sync_run(|config| cmd.run::<service::Block>(&config))?)
        }
    }?;

    #[cfg(feature = "pyroscope")]
    if let Some(pyroscope_agent) = pyroscope_agent_maybe.take() {
        let agent = pyroscope_agent.stop()?;
        agent.shutdown();
    }
    Ok(())
}
