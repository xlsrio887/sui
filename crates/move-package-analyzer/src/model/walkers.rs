// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::model::{
    global_env::GlobalEnv,
    move_model::{Bytecode, Function, Module, Package, Struct},
};

pub fn walk_packages<F>(env: &GlobalEnv, mut walker: F)
where
    F: FnMut(&GlobalEnv, &Package),
{
    env.packages.iter().for_each(|package| walker(env, package));
}

pub fn walk_modules<F>(env: &GlobalEnv, mut walker: F)
where
    F: FnMut(&GlobalEnv, &Module),
{
    env.modules.iter().for_each(|module| walker(env, module));
}

pub fn walk_structs<F>(env: &GlobalEnv, mut walker: F)
where
    F: FnMut(&GlobalEnv, &Struct),
{
    env.structs.iter().for_each(|struct_| walker(env, struct_));
}

pub fn walk_functions<F>(env: &GlobalEnv, mut walker: F)
where
    F: FnMut(&GlobalEnv, &Function),
{
    env.functions.iter().for_each(|func| walker(env, func));
}

pub fn walk_bytecodes<F>(env: &GlobalEnv, mut walker: F)
where
    F: FnMut(&GlobalEnv, &Function, &Bytecode),
{
    walk_functions(env, |env, func| {
        if let Some(code) = func.code.as_ref() {
            code.code
                .iter()
                .for_each(|bytecode| walker(env, func, bytecode));
        }
    });
}
