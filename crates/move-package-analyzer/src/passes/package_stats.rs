// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::model::walkers::walk_functions;
use crate::{
    model::{
        global_env::GlobalEnv,
        walkers::{walk_modules, walk_packages},
    },
    write_to,
};
use std::{fs::File, io::Write, path::Path};
use tracing::error;

pub(crate) fn run(env: &GlobalEnv, output: &Path) {
    packages(env, output);
    modules(env, output);
    binary_modules(env, output);
    functions(env, output);
}

fn packages(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("packages.csv"))
        .unwrap_or_else(|_| panic!("Unable to create file packages.csv in {}", output.display()));
    write_to!(
        file,
        "package, version, dependencies, origin_tables, modules, structs, functions, constants"
    );
    walk_packages(env, |env, package| {
        let mut struct_count = 0;
        let mut func_count = 0;
        let mut const_count = 0;
        for module in env.modules_in_package(package) {
            struct_count += module.structs.len();
            func_count += module.functions.len();
            const_count += module.constants.len();
        }
        write_to!(
            file,
            "{}, {}, {}, {}, {}, {}, {}, {}",
            package.id,
            package.package.as_ref().unwrap().version(),
            package.package.as_ref().unwrap().linkage_table().len(),
            package.type_origin.len(),
            package.modules.len(),
            struct_count,
            func_count,
            const_count,
        );
    });
}

fn modules(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("modules.csv"))
        .unwrap_or_else(|_| panic!("Unable to create file modules.csv in {}", output.display()));
    write_to!(file, "package, module, structs, functions, constants");
    walk_modules(env, |_env, module| {
        let struct_count = module.structs.len();
        let func_count = module.functions.len();
        let const_count = module.constants.len();
        write_to!(
            file,
            "{}, {}, {}, {}, {}",
            env.packages[module.package].id,
            module.module_id,
            struct_count,
            func_count,
            const_count,
        );
    });
}

fn binary_modules(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("binary_modules.csv")).unwrap_or_else(|_| {
        panic!(
            "Unable to create file bynary_module.csv in {}",
            output.display()
        )
    });
    write_to!(
        file,
        "package, module, \
        module_handles, struct_handles, function_handles, field_handles, \
        struct_def_instantiations, function_instantiations, field_instantiations, \
        signatures, identifiers, address_identifiers, constant_pool, \
        struct_defs, function_defs"
    );
    walk_modules(env, |env, module| {
        if let Some(compiled_module) = module.module.as_ref() {
            write_to!(
                file,
                "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                env.packages[module.package].id,
                module.module_id,
                compiled_module.module_handles.len(),
                compiled_module.struct_handles.len(),
                compiled_module.function_handles.len(),
                compiled_module.field_handles.len(),
                compiled_module.struct_def_instantiations.len(),
                compiled_module.function_instantiations.len(),
                compiled_module.field_instantiations.len(),
                compiled_module.signatures.len(),
                compiled_module.identifiers.len(),
                compiled_module.address_identifiers.len(),
                compiled_module.constant_pool.len(),
                compiled_module.struct_defs.len(),
                compiled_module.function_defs.len(),
            );
        }
    });
}

fn functions(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("functions.csv")).unwrap_or_else(|_| {
        panic!(
            "Unable to create file functions.csv in {}",
            output.display()
        )
    });
    write_to!(
        file,
        "package, function, type_parameters, parameters, returns, instructions"
    );
    walk_functions(env, |env, func| {
        write_to!(
            file,
            "{}, {}, {}, {}, {}, {}",
            env.packages[func.package].id,
            env.function_name(func),
            func.type_parameters.len(),
            func.parameters.len(),
            func.returns.len(),
            func.code.as_ref().map_or(0, |code| code.code.len()),
        );
    })
}
