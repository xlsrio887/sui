// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::model::model_utils::type_name;
use crate::{
    model::{
        global_env::GlobalEnv,
        move_model::{Function, Module},
    },
    write_to,
};
use std::{fs::File, io::Write, path::Path};
use tracing::error;

pub(crate) fn run(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("otw.csv"))
        .unwrap_or_else(|_| panic!("Unable to create file otw.csv in {}", output.display()));
    write_to!(
        file,
        "package, module_id, module_name, struct, type_params, fields, init, init_args, init_type"
    );
    env.modules.iter().for_each(|module| {
        let module_name = env.module_name(module);
        module.structs.iter().for_each(|struct_idx| {
            let struct_ = &env.structs[*struct_idx];
            let struct_name = env.struct_name(struct_);
            if struct_name == module_name.to_uppercase() {
                let init_func = find_init(env, module);
                let (init_name, init_args, init_first_arg_type) = if let Some(init) = init_func {
                    (
                        "init",
                        init.parameters.len(),
                        type_name(env, &init.parameters[0]),
                    )
                } else {
                    ("N/A", 0, "N/A".to_string())
                };
                write_to!(
                    file,
                    "{}, {}, {}, {}, {}, {}, {}, {}, {}",
                    env.packages[module.package].id,
                    module.module_id,
                    module_name,
                    struct_name,
                    struct_.type_parameters.len(),
                    struct_.fields.len(),
                    init_name,
                    init_args,
                    init_first_arg_type
                );
            }
        })
    });
}

fn find_init<'a>(env: &'a GlobalEnv, module: &Module) -> Option<&'a Function> {
    module
        .functions
        .iter()
        .find(|func_idx| &env.function_name_from_idx(**func_idx) == "init")
        .map(|func_idx| &env.functions[*func_idx])
}
