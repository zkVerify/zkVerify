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

//! zkVerify CLI library.

/// Node version **Should be the same of the one in `relay-node/Cargo.toml`**.
pub const NODE_VERSION: &str = "0.10.0";

use clap::Parser;
use std::path::PathBuf;

#[allow(missing_docs)]
#[derive(Debug, Parser)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    /// The pallet benchmarking moved to the `pallet` sub-command.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Key management CLI utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}

#[allow(missing_docs)]
#[derive(Debug, Parser)]
#[group(skip)]
pub struct RunCmd {
    #[clap(flatten)]
    pub base: sc_cli::RunCmd,

    /// Allows a validator to run insecurely outside of Secure Validator Mode. Security features
    /// are still enabled on a best-effort basis, but missing features are no longer required. For
    /// more information see <https://github.com/w3f/polkadot-wiki/issues/4881>.
    #[arg(long = "insecure-validator-i-know-what-i-do", requires = "validator")]
    pub insecure_validator: bool,

    /// Enable the block authoring backoff that is triggered when finality is lagging.
    #[arg(long)]
    pub force_authoring_backoff: bool,

    /// Add the destination address to the `pyroscope` agent.
    ///
    /// Must be valid socket address, of format `IP:Port` (commonly `127.0.0.1:4040`).
    #[arg(long)]
    pub pyroscope_server: Option<String>,

    /// Disable automatic hardware benchmarks.
    ///
    /// By default these benchmarks are automatically ran at startup and measure
    /// the CPU speed, the memory bandwidth and the disk speed.
    ///
    /// The results are then printed out in the logs, and also sent as part of
    /// telemetry, if telemetry is enabled.
    #[arg(long)]
    pub no_hardware_benchmarks: bool,

    /// Overseer message capacity override.
    ///
    /// **Dangerous!** Do not touch unless explicitly advised to.
    #[arg(long)]
    pub overseer_channel_capacity_override: Option<usize>,

    /// Path to the directory where auxiliary worker binaries reside.
    ///
    /// If not specified, the main binary's directory is searched first, then
    /// `/usr/lib/polkadot` is searched.
    ///
    /// TESTING ONLY: if the path points to an executable rather then directory,
    /// that executable is used both as preparation and execution worker.
    #[arg(long, value_name = "PATH")]
    pub workers_path: Option<PathBuf>,

    /// TESTING ONLY: disable the version check between nodes and workers.
    #[arg(long, hide = true)]
    pub disable_worker_version_check: bool,
}

#[allow(missing_docs)]
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,

    #[clap(flatten)]
    pub storage_monitor: sc_storage_monitor::StorageMonitorParams,
}
