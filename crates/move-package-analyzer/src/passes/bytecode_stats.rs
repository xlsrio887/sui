// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    model::{
        global_env::GlobalEnv, model_utils::bytecode_to_string, move_model::Bytecode,
        walkers::walk_bytecodes,
    },
    write_to,
};
use move_binary_format::file_format::Visibility;
use std::{collections::BTreeMap, fs::File, io::Write, path::Path};
use tracing::error;

pub(crate) fn run(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("bytecodes.stats")).unwrap_or_else(|_| {
        panic!(
            "Unable to create file bytecode.stats in {}",
            output.display()
        )
    });
    summary(env, file);
    check_calls(env, file);
    check_bytecodes(env, file);
}

fn summary(env: &GlobalEnv, file: &mut File) {
    let mut native = 0usize;
    let mut public = 0usize;
    let mut friend = 0usize;
    let mut private = 0usize;
    let mut entry = 0usize;
    let mut private_entry = 0usize;
    let mut public_entry = 0usize;
    let mut friend_entry = 0usize;
    for function in &env.functions {
        match function.visibility {
            Visibility::Public => {
                public += 1;
                if function.is_entry {
                    entry += 1;
                    public_entry += 1;
                }
            }
            Visibility::Friend => {
                friend += 1;
                if function.is_entry {
                    entry += 1;
                    friend_entry += 1;
                }
            }
            Visibility::Private => {
                private += 1;
                if function.is_entry {
                    entry += 1;
                    private_entry += 1;
                }
            }
        }
        if function.code.is_none() {
            native += 1;
        }
    }
    write_to!(
        file,
        "* Total functions: {}, public: {}, friend: {}, private: {}, native: {}, \
        total entry: {}, public_entry: {}, friend_entry: {}, private_entry: {}",
        env.functions.len(),
        public,
        friend,
        private,
        native,
        entry,
        public_entry,
        friend_entry,
        private_entry,
    );
}

fn check_calls(env: &GlobalEnv, file: &mut File) {
    let mut total_calls = 0usize;
    let mut in_module_calls = 0usize;
    let mut in_package_calls = 0usize;
    let mut exteranl_calls = 0usize;
    walk_bytecodes(env, |env, func, bytecode| match bytecode {
        Bytecode::Call(func_idx) | Bytecode::CallGeneric(func_idx, _) => {
            total_calls += 1;
            let callee = &env.functions[*func_idx];
            if callee.module == func.module {
                in_module_calls += 1;
            } else if callee.package == func.package {
                in_package_calls += 1;
            } else {
                exteranl_calls += 1;
            }
        }
        _ => (),
    });
    write_to!(
        file,
        "* Total calls: {}, calls within module: {}, calls within package: {}, external calls: {}\n",
        total_calls, in_module_calls, in_package_calls, exteranl_calls,
    );
}

fn check_bytecodes(env: &GlobalEnv, file: &mut File) {
    let mut bytecodes = BTreeMap::new();
    walk_bytecodes(env, |_, _, bytecode| {
        insert_bytecode(&mut bytecodes, bytecode);
    });
    let mut entries: Vec<_> = bytecodes.into_iter().collect();
    entries.sort_by_key(|&(_, count)| count);
    write_to!(file, "Bytecode:\tCount");
    for (name, count) in entries.iter().rev() {
        write_to!(file, "{}:\t{}", name, count);
    }
}

fn insert_bytecode(bytecodes: &mut BTreeMap<String, usize>, bytecode: &Bytecode) {
    let name = bytecode_to_string(bytecode);
    let count = bytecodes.entry(name).or_insert(0);
    *count += 1;
}
