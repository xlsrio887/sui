// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    model::global_env::GlobalEnv,
    passes::{bytecode_stats, env_printer, init_reporter, one_time_witness, package_stats},
    Pass, PassesConfig,
};
use std::{env, path::Path, time::Instant};
use tracing::info;

pub fn run(passes: &PassesConfig, env: &GlobalEnv) {
    let output_path = if let Some(path) = passes.output_dir.as_ref() {
        Path::new(path).to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| panic!("Cannot get current directory: {}", e))
            .unwrap()
    };
    for pass in passes.passes.iter() {
        let pass_time_start = Instant::now();
        match pass {
            Pass::PackageStats => package_stats::run(env, &output_path),
            Pass::BytecodeStats => bytecode_stats::run(env, &output_path),
            Pass::PrintEnv => env_printer::run(env, &output_path),
            Pass::OneTimeWitness => one_time_witness::run(env, &output_path),
            Pass::InitReporter => init_reporter::run(env, &output_path),
        }
        let pass_time_end = Instant::now();
        info!(
            "Run {:?} pass in {}ms",
            pass,
            pass_time_end.duration_since(pass_time_start).as_millis(),
        );
    }
}
