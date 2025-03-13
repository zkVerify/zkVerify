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

//! This crate provides a tool to handle native dependency caching. The main entry point are
//! [`handle_dependency`] and [`handle_dependencies`] functions.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use config::Config;
use walkdir::WalkDir;

// Reexport
pub use dependency::{Boxed, Dependency};
pub use helpers::{is_dyn_or_static_lib, is_name};
pub use lib_dependency::{DependencyImpl, LibFilesDependency, ProfileLibFilesDependencyBuilder};
#[cfg(feature = "rocksdb")]
pub use rocksdb::rocksdb;

mod config;
mod dependency;
mod helpers;
mod lib_dependency;
mod rocksdb;

/// Enable logging.
pub const ENABLE_LOGS: bool = false;

/// Handle a single dependency
/// - `target_root` is the path to the target directory where cargo places build artifacts.
/// - `dependency` that should define how to cache by implement [`Dependency`].
/// - `profile` is the compilation profile; In your build script you can use `PROFILE`
///    environment variable that cargo set: `let profile = env::var("PROFILE").unwrap();`
pub fn handle_dependency(
    target_root: impl AsRef<Path>,
    dependency: &impl Dependency,
    profile: &str,
) -> anyhow::Result<()> {
    let mut config = cargo_config()?;
    handle_dependency_inner(&mut config, target_root, dependency, profile)
}

fn handle_dependency_inner(
    config: &mut Config,
    target_root: impl AsRef<Path>,
    dependency: &impl Dependency,
    profile: &str,
) -> anyhow::Result<()> {
    if skip_native_cache() {
        return Ok(());
    }
    let target_path = target_path(target_root, profile);
    let cache = config.get(dependency).map(PathBuf::from);

    if let Some(cache) = cache {
        if dependency.is_valid_cache(&cache) {
            dependency.rerun_if(&cache);
            return Ok(());
        }
        // Otherwise ignore it and try with the default cache.
    }

    let cache = dependency.default_cache_path();
    let valid = if dependency.is_valid_cache(cache) {
        true
    } else {
        fill_cache(&target_path, cache, dependency)?
    };
    if valid {
        dependency.rerun_if(cache);
    } else {
        println!("cargo::rerun-if-changed={}", target_path.display());
    }
    set_env_path(config, dependency, &format!("{}", cache.display()), !valid)
}

/// Handle a set of dependencies
/// - `target_root` is the path to the target directory where cargo places build artifacts.
/// - `dependencies` a set of dependencies to be cached; every item should implement [`Dependency`] trait.
/// - `profile` is the compilation profile; In your build script you can use `PROFILE`
///    environment variable that cargo set: `let profile = env::var("PROFILE").unwrap();`
pub fn handle_dependencies<'a>(
    target_root: impl AsRef<Path>,
    dependencies: impl IntoIterator<Item = &'a Box<dyn Dependency>>,
    profile: &str,
) -> anyhow::Result<()> {
    if skip_native_cache() {
        return Ok(());
    }
    let mut config = cargo_config()?;
    for dependency in dependencies {
        handle_dependency_inner(&mut config, target_root.as_ref(), dependency, profile)?
    }
    Ok(())
}

fn target_path(target_root: impl AsRef<Path>, profile: &str) -> PathBuf {
    target_root
        .as_ref()
        .to_path_buf()
        .join(profile)
        .join("build")
}

fn fill_cache(
    target_path: impl AsRef<Path>,
    cache: impl AsRef<Path>,
    dependency: &impl Dependency,
) -> Result<bool, anyhow::Error> {
    let target_path = target_path.as_ref();
    // We ignore the error because doesn't matter if the cache folder doesn't exist.
    let _ = fs::remove_dir_all(cache.as_ref());
    log!("Rebuild from {}", target_path.display());
    for entry in WalkDir::new(target_path).max_depth(1).into_iter().flatten() {
        let path = entry.path();
        log!("folder {}", path.display());
        if dependency.folder_match(path) {
            log!("folder {} MATCH", path.display());
            dependency
                .cache_files(path, cache.as_ref())
                .context("Unable to copy dependency")?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn set_env_path(
    config: &mut Config,
    dependency: &impl Dependency,
    value: &str,
    reset: bool,
) -> anyhow::Result<()> {
    if reset {
        config.remove(dependency);
    } else {
        config.add(dependency, value);
    }
    config.store()
}

fn skip_native_cache() -> bool {
    "true"
        == &env::var("DONT_CACHE_NATIVE")
            .unwrap_or_default()
            .to_lowercase()
}

fn cargo_config() -> Result<Config, anyhow::Error> {
    Config::load(PathBuf::from(env!("CARGO_HOME")).join("config.toml"))
}
