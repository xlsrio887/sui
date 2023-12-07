// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::PackageAnalyzerError;
use serde::Deserialize;
use std::{fs, path::Path};

pub mod errors;
pub mod load_from_dir;
pub mod model;
pub mod passes;
pub mod query_indexer;

// Global constants
const DEFAULT_CAPACITY: usize = 16 * 1024;
const PACKAGE_BCS: &str = "package.bcs";

#[derive(Debug, Deserialize)]
pub enum Pass {
    PackageStats,
    BytecodeStats,
    PrintEnv,
    OneTimeWitness,
    InitReporter,
}

#[derive(Debug, Deserialize)]
pub struct PassesConfig {
    pub passes: Vec<Pass>,
    pub output_dir: Option<String>,
}

pub fn load_config(path: &Path) -> Result<PassesConfig, PackageAnalyzerError> {
    let reader = fs::File::open(path).map_err(|e| {
        PackageAnalyzerError::BadConfig(format!(
            "Cannot open config file {}: {}",
            path.display(),
            e
        ))
    })?;
    let config: PassesConfig = serde_yaml::from_reader(reader).map_err(|e| {
        PackageAnalyzerError::BadConfig(format!(
            "Cannot parse config file {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(config)
}

#[macro_export]
macro_rules! write_to {
    ($file:expr, $($arg:tt)*) => {{
        writeln!($file, $($arg)*).unwrap_or_else(|e| error!(
            "Unable to write to file: {}",
            e.to_string()
        ))
    }};
}
