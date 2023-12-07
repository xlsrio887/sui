// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use crate::{
    model::move_model::{
        Constant, Field, Function, FunctionIndex, IdentifierIndex, Module, ModuleIndex, Package,
        PackageIndex, Struct, StructIndex,
    },
    DEFAULT_CAPACITY,
};
use move_binary_format::file_format::ConstantPoolIndex;
use std::{collections::BTreeMap, default::Default};
use sui_types::base_types::ObjectID;

#[derive(Debug)]
pub struct GlobalEnv {
    //
    // pools of Move "entities" across packages.
    // All entries are unique. Everything is interned.
    //
    pub packages: Vec<Package>,
    pub modules: Vec<Module>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
    pub identifiers: Vec<String>,

    //
    // maps of Move "entities" across packages
    //

    // All keys in the maps are based on the ObjectID of the package the entity
    // is defined in. There is no reconciliation of modules, types or functions
    // as related to versioning. In other words a module with all its types
    // and functions is repeated for as many versioned packages the module is
    // "redefined", even if the module and all its content is unchanged.
    pub package_map: BTreeMap<ObjectID, PackageIndex>,
    pub module_map: BTreeMap<String, ModuleIndex>,
    pub function_map: BTreeMap<String, FunctionIndex>,
    pub struct_map: BTreeMap<String, StructIndex>,
    // interns all identifiers
    pub identifier_map: BTreeMap<String, IdentifierIndex>,
}

impl GlobalEnv {
    pub fn module_name_from_idx(&self, idx: ModuleIndex) -> String {
        let module = &self.modules[idx];
        self.module_name(module)
    }

    pub fn module_name(&self, module: &Module) -> String {
        let name = module.name;
        self.identifiers[name].clone()
    }

    pub fn struct_name_from_idx(&self, idx: StructIndex) -> String {
        let struct_ = &self.structs[idx];
        self.struct_name(struct_)
    }

    pub fn struct_name(&self, struct_: &Struct) -> String {
        let name = struct_.name;
        self.identifiers[name].clone()
    }

    pub fn function_name_from_idx(&self, idx: FunctionIndex) -> String {
        let func = &self.functions[idx];
        self.function_name(func)
    }

    pub fn function_name(&self, func: &Function) -> String {
        let name = func.name;
        self.identifiers[name].clone()
    }

    pub fn field_name(&self, field: &Field) -> String {
        let name = field.name;
        self.identifiers[name].clone()
    }

    pub fn modules_in_package<'a>(
        &'a self,
        package: &'a Package,
    ) -> impl Iterator<Item = &Module> + 'a {
        package
            .modules
            .iter()
            .map(move |module_idx| &self.modules[*module_idx])
    }
}
