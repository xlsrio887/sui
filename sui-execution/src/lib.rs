// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// DO NOT MODIFY, Generated by ./scripts/execution-layer

use std::sync::Arc;

use sui_protocol_config::ProtocolConfig;
use sui_types::{error::SuiResult, metrics::BytecodeVerifierMetrics};

use move_vm_types::natives::native_functions::NativeFunctionTable;

pub use executor::Executor;
pub use verifier::UnmeteredVerifier;
pub use verifier::Verifier;

pub mod executor;
pub mod verifier;

mod latest;
mod v0;
mod vm_rework;

#[cfg(test)]
mod tests;

pub const VM_REWORK: u64 = u64::MAX;
pub fn executor(
    protocol_config: &ProtocolConfig,
    paranoid_type_checks: bool,
    silent: bool,
) -> SuiResult<Arc<dyn Executor + Send + Sync>> {
    let version = protocol_config.execution_version_as_option().unwrap_or(0);
    Ok(match version {
        0 => Arc::new(v0::Executor::new(
            protocol_config,
            paranoid_type_checks,
            silent,
        )?),

        1 => Arc::new(latest::Executor::new(
            protocol_config,
            paranoid_type_checks,
            silent,
        )?),

        VM_REWORK => Arc::new(vm_rework::Executor::new(
            protocol_config,
            paranoid_type_checks,
            silent,
        )?),

        v => panic!("Unsupported execution version {v}"),
    })
}

pub fn verifier<'m>(
    protocol_config: &ProtocolConfig,
    is_metered: bool,
    metrics: &'m Arc<BytecodeVerifierMetrics>,
) -> Box<dyn Verifier + 'm> {
    let version = protocol_config.execution_version_as_option().unwrap_or(0);
    match version {
        0 => Box::new(v0::Verifier::new(protocol_config, is_metered, metrics)),
        1 => Box::new(latest::Verifier::new(protocol_config, is_metered, metrics)),
        VM_REWORK => Box::new(vm_rework::Verifier::new(
            protocol_config,
            is_metered,
            metrics,
        )),
        v => panic!("Unsupported execution version {v}"),
    }
}

pub fn unmetered_verifier<'m>(execution_version: u64) -> Box<dyn UnmeteredVerifier + 'm> {
    match execution_version {
        0 => Box::new(v0::UnmeteredVerifier::new()),
        1 => Box::new(latest::UnmeteredVerifier::new()),
        VM_REWORK => Box::new(vm_rework::UnmeteredVerifier::new()),
        v => panic!("Unsupported execution version {v}"),
    }
}

pub fn all_natives(execution_version: u64, silent: bool) -> NativeFunctionTable {
    match execution_version {
        0 => v0::all_natives(silent),
        1 => latest::all_natives(silent),
        VM_REWORK => vm_rework::all_natives(silent),
        v => panic!("Unsupported execution version {v}"),
    }
}
