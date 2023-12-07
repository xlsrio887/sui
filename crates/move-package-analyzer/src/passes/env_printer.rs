// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    model::{global_env::GlobalEnv, model_utils::type_name},
    write_to,
};
use move_binary_format::file_format::{AbilitySet, Visibility};
use std::{fs::File, io::Write, path::Path};
use tracing::error;

/// Write `GlobalEnv` to `package.env` file.
pub fn run(env: &GlobalEnv, output: &Path) {
    let file = &mut File::create(output.join("packages.env"))
        .unwrap_or_else(|_| panic!("Unable to create file packages.env in {}", output.display()));
    for package in env.packages.iter() {
        let move_package = package.package.as_ref().unwrap();
        write_to!(file, "package {}", package.id);
        write_to!(file, "version {}", move_package.version());
        write_to!(file, "type origin {:#?}", move_package.type_origin_table());
        write_to!(file, "linkage table {:#?}", move_package.linkage_table());
        for module_idx in &package.modules {
            let module = &env.modules[*module_idx];
            write_to!(file, "\tmodule {}", env.module_name(module));
            for struct_idx in &module.structs {
                let struct_ = &env.structs[*struct_idx];
                let abilities = if struct_.abilities != AbilitySet::EMPTY {
                    format!("has {}", pretty_abilities(struct_.abilities))
                } else {
                    "".to_string()
                };
                if struct_.type_parameters.is_empty() {
                    write_to!(
                        file,
                        "\t\tstruct {} {}",
                        env.struct_name(struct_),
                        abilities
                    );
                } else {
                    write_to!(
                        file,
                        "\t\tstruct {}<{}> {}",
                        env.struct_name(struct_),
                        struct_
                            .type_parameters
                            .iter()
                            .enumerate()
                            .map(|(idx, abilities)| {
                                let phantom = if abilities.is_phantom { "phantom " } else { "" };
                                if abilities.constraints == AbilitySet::EMPTY {
                                    format!("{} {}", phantom, idx)
                                } else {
                                    format!(
                                        "{}{}: {}",
                                        phantom,
                                        idx,
                                        pretty_abilities(abilities.constraints)
                                    )
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        abilities,
                    );
                }
                for field in &struct_.fields {
                    write_to!(
                        file,
                        "\t\t\t{}: {}",
                        env.field_name(field),
                        type_name(env, &field.type_),
                    );
                }
            }
            for func_idx in &module.functions {
                let func = &env.functions[*func_idx];
                let func_name = if func.is_entry {
                    "entry fun".to_string()
                } else {
                    "fun".to_string()
                };
                let func_name = match func.visibility {
                    Visibility::Private => func_name,
                    Visibility::Public => format!("public {}", func_name),
                    Visibility::Friend => format!("friend {}", func_name),
                };
                let params = if func.parameters.is_empty() {
                    "".to_string()
                } else {
                    func.parameters
                        .iter()
                        .map(|type_| type_name(env, type_))
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_string()
                };
                let func_proto = if func.type_parameters.is_empty() {
                    format!("\t\tfun {}({})", env.function_name(func), params)
                } else {
                    format!(
                        "\t\t{} {}<{}>({})",
                        func_name,
                        env.function_name(func),
                        func.type_parameters
                            .iter()
                            .enumerate()
                            .map(|(idx, ability_set)| {
                                if ability_set == &AbilitySet::EMPTY {
                                    format!("{}", idx)
                                } else {
                                    format!("{}: {}", idx, pretty_abilities(*ability_set))
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        params,
                    )
                };
                if func.returns.is_empty() {
                    write_to!(file, "{}", func_proto);
                } else {
                    write_to!(
                        file,
                        "{}: {}",
                        func_proto,
                        func.returns
                            .iter()
                            .map(|type_| type_name(env, type_))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                }
            }
        }
    }
}

fn pretty_abilities(ability_set: AbilitySet) -> String {
    let mut abilities = vec![];
    if ability_set == AbilitySet::EMPTY {
        return "".to_string();
    }
    if ability_set.has_key() {
        abilities.push("key");
    }
    if ability_set.has_store() {
        abilities.push("store");
    }
    if ability_set.has_copy() {
        abilities.push("copy");
    }
    if ability_set.has_drop() {
        abilities.push("drop");
    }
    abilities.join(", ")
}
