// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    model::{
        global_env::GlobalEnv,
        model_utils::type_name,
        move_model::{Function, Module},
    },
    write_to,
};
use std::{fs::File, io::Write, path::Path};
use tracing::error;

pub(crate) fn run(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("init.csv"))
        .unwrap_or_else(|_| panic!("Unable to create file init.csv in {}", output.display()));
    write_to!(
        file,
        "package, version, modules, structs, functions, module_id, module_name, init_args, \
        init_type_params, init_first_arg_type, init_instructions"
    );
    env.modules.iter().for_each(|module| {
        if let Some(init) = find_init(env, module) {
            let package = &env.packages[module.package];
            let init_args = init.parameters.len();
            let init_first_arg_type = if init_args > 0 {
                type_name(env, &init.parameters[0])
            } else {
                "N/A".to_string()
            };
            let init_instructions = if let Some(code) = init.code.as_ref() {
                code.code.len()
            } else {
                0
            };
            write_to!(
                file,
                "{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}",
                package.id,                                  // package
                package.package.as_ref().unwrap().version(), // version
                package.modules.len(),                       // modules
                package.struct_count(env),                   // structs
                package.function_count(env),                 // functions
                module.module_id,                            // module
                env.module_name(module),                     // module_name
                init_args,                                   // init_args
                init.type_parameters.len(),                  // init_type_params
                init_first_arg_type,                         // init_first_arg_type
                init_instructions,                           // init_instructions
            );
        }
    });
}

fn find_init<'a>(env: &'a GlobalEnv, module: &Module) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|func_idx| &env.function_name_from_idx(**func_idx) == "init")
        .map(|func_idx| &env.functions[*func_idx])
}
