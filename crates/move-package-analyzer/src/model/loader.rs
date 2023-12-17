// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::errors::PackageAnalyzerError;
use crate::model::compiled_module_util::{
    get_package_from_function_handle, get_package_from_struct_def, get_package_from_struct_handle,
    module_function_name_from_def, module_function_name_from_handle, module_struct_name_from_def,
    module_struct_name_from_handle,
};
use crate::{
    model::{
        global_env::GlobalEnv,
        move_model::{
            Bytecode, Code, Constant, Field, FieldRef, Function, FunctionIndex, IdentifierIndex,
            Module, ModuleIndex, Package, PackageIndex, Struct, StructIndex, Type,
        },
    },
    DEFAULT_CAPACITY,
};
use move_binary_format::{
    file_format::{
        Bytecode as MoveBytecode, ConstantPoolIndex, FunctionDefinitionIndex, FunctionHandleIndex,
        SignatureToken, StructDefinitionIndex, StructFieldInformation, StructHandleIndex,
    },
    CompiledModule,
};
use std::collections::BTreeMap;
use sui_types::{base_types::ObjectID, move_package::MovePackage};

/// Single entry point into this module, builds a `GlobalEnv` from a collection of `MovePackage`'s.
/// API in `GlobalEnv` and `walkers` can be used to traverse the environment.
pub fn build_environment(packages: Vec<MovePackage>) -> GlobalEnv {
    let mut identifier_map = IdentifierMap::new();

    // reserve indexes for all entities, so indexes are stable.
    // Enforce basic invariants in the process.
    let (mut packages, package_map) = load_packages(packages);
    let (mut modules, module_map) = load_modules(&mut identifier_map, &mut packages);
    let (mut structs, struct_map) = load_structs(&mut identifier_map, &mut modules, &packages);

    // now all types are known

    // load fields and constants
    let type_builder = TypeBuilder {
        packages,
        struct_map,
    };
    load_fields(&mut structs, &mut identifier_map, &type_builder, &modules);
    load_constants(&type_builder, &mut modules);

    // load functions and code
    let (mut functions, function_map) =
        load_functions(&mut identifier_map, &mut modules, &type_builder);
    load_code(&mut functions, &type_builder, &modules, &function_map);

    // build and return the environment
    let IdentifierMap {
        identifiers,
        identifier_map,
    } = identifier_map;
    let TypeBuilder {
        packages,
        struct_map,
    } = type_builder;

    GlobalEnv {
        packages,
        modules,
        functions,
        structs,
        identifiers,
        package_map,
        module_map,
        function_map,
        struct_map,
        identifier_map,
    }
}

// Intern table for identifiers.
#[derive(Debug)]
struct IdentifierMap {
    identifiers: Vec<String>,
    identifier_map: BTreeMap<String, IdentifierIndex>,
}

impl IdentifierMap {
    fn new() -> Self {
        Self {
            identifiers: Vec::with_capacity(DEFAULT_CAPACITY),
            identifier_map: BTreeMap::new(),
        }
    }

    // Itern an identifier and return its index in the intern table.
    fn get_identifier_idx(&mut self, ident: &String) -> IdentifierIndex {
        if let Some(idx) = self.identifier_map.get(ident) {
            return *idx;
        }
        let idx = self.identifiers.len();
        self.identifiers.push(ident.clone());
        self.identifier_map.insert(ident.clone(), idx);
        idx
    }

    // Get an identifier from its index.
    fn get_identifier(&self, idx: IdentifierIndex) -> &str {
        self.identifiers[idx].as_str()
    }
}

// Build types from signatures.
// All signatures are expanded to vector of types.
struct TypeBuilder {
    packages: Vec<Package>,
    struct_map: BTreeMap<String, StructIndex>,
}

impl TypeBuilder {
    fn make_type(&self, module: &Module, type_: &SignatureToken) -> Type {
        match type_ {
            SignatureToken::Bool => Type::Bool,
            SignatureToken::U8 => Type::U8,
            SignatureToken::U16 => Type::U16,
            SignatureToken::U32 => Type::U32,
            SignatureToken::U64 => Type::U64,
            SignatureToken::U128 => Type::U128,
            SignatureToken::U256 => Type::U256,
            SignatureToken::Address => Type::Address,
            SignatureToken::Vector(inner) => Type::Vector(Box::new(self.make_type(module, inner))),
            SignatureToken::Struct(struct_handle_idx) => {
                let idx = self
                    .get_struct_idx(module, *struct_handle_idx)
                    .unwrap_or_else(|err| {
                        panic!(
                            "\nFailure getting struct index for struct handle {:?}: {:?}\nPackage {}, module id {}\n",
                            struct_handle_idx,
                            err,
                            self.packages[module.package].id,
                            module.module_id,
                        )
                    });
                Type::Struct(idx)
            }
            SignatureToken::StructInstantiation(struct_handle_idx, type_arguments) => {
                let idx = self
                    .get_struct_idx(module, *struct_handle_idx)
                    .unwrap_or_else(|err| {
                        panic!(
                            "\nFailure getting struct index for struct handle {:?}: {:?}\nPackage {}, module id {}\n",
                            struct_handle_idx,
                            err,
                            self.packages[module.package].id,
                            module.module_id,
                        )
                    });
                let type_arguments = type_arguments
                    .iter()
                    .map(|type_| self.make_type(module, type_))
                    .collect::<Vec<_>>();
                Type::StructInstantiation(idx, type_arguments)
            }
            SignatureToken::Reference(inner) => {
                Type::Reference(Box::new(self.make_type(module, inner)))
            }
            SignatureToken::MutableReference(inner) => {
                Type::MutableReference(Box::new(self.make_type(module, inner)))
            }
            SignatureToken::TypeParameter(idx) => Type::TypeParameter(*idx),
            _ => panic!("Invalid type found: {:?}", type_),
        }
    }

    fn get_struct_idx(
        &self,
        module: &Module,
        struct_handle_idx: StructHandleIndex,
    ) -> Result<StructIndex, PackageAnalyzerError> {
        let compiled_module = module.module.as_ref().unwrap();
        let (module_name, struct_name) =
            module_struct_name_from_handle(compiled_module, struct_handle_idx);
        let package_id = get_package_from_struct_handle(compiled_module, struct_handle_idx);
        let package = &self.packages[module.package];
        let key = (module_name.to_string(), struct_name.to_string());
        let package_id = match package.type_origin.get(&key) {
            None => ObjectID::from(package_id),
            Some(package_id) => *package_id,
        };
        let package_id = match package
            .package
            .as_ref()
            .unwrap()
            .linkage_table()
            .get(&package_id)
        {
            None => package_id,
            Some(upgrade_info) => upgrade_info.upgraded_id,
        };
        let struct_key = format!("{}::{}::{}", package_id, module_name, struct_name);
        get_value(&struct_key, &self.struct_map)
    }
}

// Build a `GlobalEnv` from a collection of `MovePackage`'s.

fn load_packages(packages: Vec<MovePackage>) -> (Vec<Package>, BTreeMap<ObjectID, PackageIndex>) {
    let packages = packages
        .into_iter()
        .enumerate()
        .map(|(self_idx, package)| {
            let type_origin = package.type_origin_map();
            let version = package.version();
            Package {
                self_idx,
                id: package.id(),
                package: Some(package),
                version: Some(version),
                type_origin,
                modules: vec![],
            }
        })
        .collect::<Vec<_>>();
    let package_map = packages
        .iter()
        .enumerate()
        .map(|(idx, package)| (package.id, idx))
        .collect::<BTreeMap<_, _>>();
    (packages, package_map)
}

fn load_modules(
    identifier_map: &mut IdentifierMap,
    packages: &mut [Package],
) -> (Vec<Module>, BTreeMap<String, ModuleIndex>) {
    let mut modules = packages
        .iter()
        .enumerate()
        .flat_map(|(pkg_idx, package)| {
            let package_id = package.id;
            let move_package = package.package.as_ref().unwrap();
            let modules: Vec<(&str, CompiledModule)> = move_package
                .serialized_module_map()
                .iter()
                .map(|(name, bytes)| {
                    (
                        name.as_str(),
                        CompiledModule::deserialize_with_defaults(bytes).unwrap_or_else(|err| {
                            panic!(
                                "Failure deserializing module {} in package {}: {:?}",
                                name, package_id, err,
                            )
                        }),
                    )
                })
                .collect::<Vec<_>>();
            modules
                .into_iter()
                .map(|(name, module)| {
                    let module_id = module.self_id();
                    let module_name = module_id.name().as_str();
                    assert_eq!(
                        name, module_name,
                        "Mismatch in package {}: module name {} and name in ModuleId {}",
                        package_id, name, module_id,
                    );
                    let name = identifier_map.get_identifier_idx(&module_name.to_string());
                    Module {
                        self_idx: 0,
                        package: pkg_idx,
                        module: Some(module),
                        name,
                        module_id,
                        dependencies: vec![],
                        structs: vec![],
                        functions: vec![],
                        constants: vec![],
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let module_map = modules
        .iter_mut()
        .enumerate()
        .map(|(idx, module)| {
            // update module and packages
            module.self_idx = idx;
            let package = &mut packages[module.package];
            package.modules.push(idx);
            let package_id = package.id;
            let module_name = identifier_map.get_identifier(module.name);
            assert_eq!(
                module_name,
                module.module_id.name().as_str(),
                "Mismatch in package {}: module name {} and name in ModuleId {}",
                package_id,
                module_name,
                module.module_id,
            );
            let key = format!("{}::{}", package_id, module_name);
            (key, idx)
        })
        .collect::<BTreeMap<_, _>>();
    (modules, module_map)
}

// Set up all `Struct`s in GlobalEnv
fn load_structs(
    identifier_map: &mut IdentifierMap,
    modules: &mut [Module],
    packages: &[Package],
) -> (Vec<Struct>, BTreeMap<String, StructIndex>) {
    let mut structs = modules
        .iter()
        .enumerate()
        .flat_map(|(idx, module)| {
            let compiled_module = module.module.as_ref().unwrap();
            assert_eq!(
                identifier_map.get_identifier(module.name),
                module.module_id.name().as_str(),
                "Mismatch in module name: env name {}, handle name {}",
                identifier_map.get_identifier(module.name),
                module.module_id.name().as_str(),
            );
            compiled_module
                .struct_defs
                .iter()
                .enumerate()
                .map(|(def_idx, struct_def)| {
                    let struct_handle =
                        &compiled_module.struct_handles[struct_def.struct_handle.0 as usize];
                    let abilities = struct_handle.abilities;
                    let type_parameters = struct_handle.type_parameters.clone();
                    let struct_name = module.module.as_ref().unwrap().identifiers
                        [struct_handle.name.0 as usize]
                        .to_string();
                    let name = identifier_map.get_identifier_idx(&struct_name);
                    Struct {
                        self_idx: 0,
                        package: module.package,
                        module: idx,
                        name,
                        def_idx: StructDefinitionIndex(def_idx as u16),
                        abilities,
                        type_parameters,
                        fields: vec![],
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let struct_map = structs
        .iter_mut()
        .enumerate()
        .map(|(idx, struct_)| {
            struct_.self_idx = idx;
            modules[struct_.module].structs.push(idx);
            let package_id = packages[struct_.package].id;
            let module = &modules[struct_.module];
            let compiled_module = module.module.as_ref().unwrap();
            let (mh_name, st_name) = module_struct_name_from_def(compiled_module, struct_.def_idx);
            let module_name = identifier_map.get_identifier(module.name);
            let struct_name = identifier_map.get_identifier(struct_.name);
            assert_eq!(
                module_name, mh_name,
                "Mismatch in module name: env name {}, module handle name {}",
                module_name, mh_name,
            );
            assert_eq!(
                struct_name, st_name,
                "Mismatch in struct name: env name {}, struct handle name {}",
                struct_name, st_name,
            );
            let key = format!("{}::{}::{}", package_id, module_name, struct_name);
            (key, idx)
        })
        .collect::<BTreeMap<_, _>>();

    (structs, struct_map)
}

fn load_fields(
    structs: &mut [Struct],
    identifier_map: &mut IdentifierMap,
    type_builder: &TypeBuilder,
    modules: &[Module],
) {
    structs.iter_mut().for_each(|struct_| {
        let module = &modules[struct_.module];
        let compiled_module = module.module.as_ref().unwrap();
        let struct_def = &compiled_module.struct_defs[struct_.def_idx.0 as usize];
        let fields = if let StructFieldInformation::Declared(fields) = &struct_def.field_information
        {
            fields
                .iter()
                .map(|field| {
                    let name = compiled_module.identifiers[field.name.0 as usize].to_string();
                    Field {
                        name: identifier_map.get_identifier_idx(&name),
                        type_: type_builder.make_type(module, &field.signature.0),
                    }
                })
                .collect::<Vec<_>>()
        } else {
            panic!(
                "Found native field in module {} in package {}",
                compiled_module.self_id(),
                module.package,
            )
        };
        struct_.fields = fields;
    });
}

fn load_functions(
    identifier_map: &mut IdentifierMap,
    modules: &mut [Module],
    type_builder: &TypeBuilder,
) -> (Vec<Function>, BTreeMap<String, FunctionIndex>) {
    let mut functions = modules
        .iter()
        .flat_map(|module| {
            let compiled_module = module.module.as_ref().unwrap();
            compiled_module
                .function_defs
                .iter()
                .enumerate()
                .map(|(def_idx, func_def)| {
                    let func_handle =
                        &compiled_module.function_handles[func_def.function.0 as usize];

                    let visibility = func_def.visibility;
                    let is_entry = func_def.is_entry;
                    let type_parameters = func_handle.type_parameters.clone();

                    let params = &compiled_module.signatures[func_handle.parameters.0 as usize];
                    let parameters = params
                        .0
                        .iter()
                        .map(|type_| type_builder.make_type(module, type_))
                        .collect::<Vec<_>>();

                    let rets = &compiled_module.signatures[func_handle.return_.0 as usize];
                    let returns = rets
                        .0
                        .iter()
                        .map(|type_| type_builder.make_type(module, type_))
                        .collect::<Vec<_>>();

                    let func_name =
                        compiled_module.identifiers[func_handle.name.0 as usize].to_string();
                    let name = identifier_map.get_identifier_idx(&func_name);
                    assert_eq!(
                        identifier_map.get_identifier(name),
                        func_name,
                        "Mismatch in function name: env name {}, module handle name {}",
                        identifier_map.get_identifier(name),
                        func_name,
                    );

                    Function {
                        self_idx: 0,
                        package: module.package,
                        module: module.self_idx,
                        name,
                        def_idx: FunctionDefinitionIndex(def_idx as u16),
                        type_parameters,
                        parameters,
                        returns,
                        visibility,
                        is_entry,
                        code: None,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let function_map = functions
        .iter_mut()
        .enumerate()
        .map(|(idx, function)| {
            let package_id = type_builder.packages[function.package].id;
            function.self_idx = idx;
            let module = &mut modules[function.module];
            module.functions.push(idx);
            let compiled_module = module.module.as_ref().unwrap();
            let (mh_name, func_name) =
                module_function_name_from_def(compiled_module, function.def_idx);
            let module_name = identifier_map.get_identifier(module.name);
            let function_name = identifier_map.get_identifier(function.name);
            assert_eq!(
                module_name, mh_name,
                "Mismatch in module name: env name {}, module handle name {}",
                module_name, mh_name,
            );
            assert_eq!(
                function_name, func_name,
                "Mismatch in function name: env name {}, function handle name {}",
                function_name, func_name,
            );
            let key = format!("{}::{}::{}", package_id, module_name, function_name);
            (key, idx)
        })
        .collect::<BTreeMap<_, _>>();
    (functions, function_map)
}

fn load_code(
    functions: &mut [Function],
    type_builder: &TypeBuilder,
    modules: &[Module],
    function_map: &BTreeMap<String, FunctionIndex>,
) {
    functions.iter_mut().for_each(|function| {
        let module = &modules[function.module];

        macro_rules! get_from_map {
            ($key:expr, $map:expr) => {
                get_value($key, $map).unwrap_or_else(|err| {
                    panic!(
                        "Unable to find key {}, err: {}\npackage {}, module {}, function {}",
                        $key,
                        err,
                        type_builder.packages[module.package].id,
                        module.module_id,
                        function.def_idx,
                    )
                })
            };
        }

        let compiled_module = module.module.as_ref().unwrap();
        let func_def = &compiled_module.function_defs[function.def_idx.0 as usize];
        if let Some(code_unit) = func_def.code.as_ref() {
            let locals = &compiled_module.signatures[code_unit.locals.0 as usize];
            let locals = locals
                .0
                .iter()
                .map(|type_| type_builder.make_type(module, type_))
                .collect::<Vec<_>>();
            let code: Vec<Bytecode> = code_unit
                .code
                .iter()
                .map(|bytecode| match bytecode {
                    MoveBytecode::Nop => Bytecode::Nop,
                    MoveBytecode::Pop => Bytecode::Pop,
                    MoveBytecode::Ret => Bytecode::Ret,
                    MoveBytecode::BrTrue(code_offset) => Bytecode::BrTrue(*code_offset),
                    MoveBytecode::BrFalse(code_offset) => Bytecode::BrFalse(*code_offset),
                    MoveBytecode::Branch(code_offset) => Bytecode::Branch(*code_offset),
                    MoveBytecode::LdConst(idx) => Bytecode::LdConst(*idx),
                    MoveBytecode::LdTrue => Bytecode::LdTrue,
                    MoveBytecode::LdFalse => Bytecode::LdFalse,
                    MoveBytecode::LdU8(v) => Bytecode::LdU8(*v),
                    MoveBytecode::LdU16(v) => Bytecode::LdU16(*v),
                    MoveBytecode::LdU32(v) => Bytecode::LdU32(*v),
                    MoveBytecode::LdU64(v) => Bytecode::LdU64(*v),
                    MoveBytecode::LdU128(v) => Bytecode::LdU128(*v),
                    MoveBytecode::LdU256(v) => Bytecode::LdU256(*v),
                    MoveBytecode::CastU8 => Bytecode::CastU8,
                    MoveBytecode::CastU16 => Bytecode::CastU16,
                    MoveBytecode::CastU32 => Bytecode::CastU32,
                    MoveBytecode::CastU64 => Bytecode::CastU64,
                    MoveBytecode::CastU128 => Bytecode::CastU128,
                    MoveBytecode::CastU256 => Bytecode::CastU256,
                    MoveBytecode::Add => Bytecode::Add,
                    MoveBytecode::Sub => Bytecode::Sub,
                    MoveBytecode::Mul => Bytecode::Mul,
                    MoveBytecode::Mod => Bytecode::Mod,
                    MoveBytecode::Div => Bytecode::Div,
                    MoveBytecode::BitOr => Bytecode::BitOr,
                    MoveBytecode::BitAnd => Bytecode::BitAnd,
                    MoveBytecode::Xor => Bytecode::Xor,
                    MoveBytecode::Or => Bytecode::Or,
                    MoveBytecode::And => Bytecode::And,
                    MoveBytecode::Not => Bytecode::Not,
                    MoveBytecode::Eq => Bytecode::Eq,
                    MoveBytecode::Neq => Bytecode::Neq,
                    MoveBytecode::Lt => Bytecode::Lt,
                    MoveBytecode::Gt => Bytecode::Gt,
                    MoveBytecode::Le => Bytecode::Le,
                    MoveBytecode::Ge => Bytecode::Ge,
                    MoveBytecode::Shl => Bytecode::Shl,
                    MoveBytecode::Shr => Bytecode::Shr,
                    MoveBytecode::Abort => Bytecode::Abort,
                    MoveBytecode::CopyLoc(idx) => Bytecode::CopyLoc(*idx),
                    MoveBytecode::MoveLoc(idx) => Bytecode::MoveLoc(*idx),
                    MoveBytecode::StLoc(idx) => Bytecode::StLoc(*idx),
                    MoveBytecode::Call(idx) => {
                        let func_key = get_function_key_from_handle(
                            *idx,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let func_idx = get_from_map!(&func_key, function_map);
                        Bytecode::Call(func_idx)
                    }
                    MoveBytecode::CallGeneric(idx) => {
                        let func_inst = &compiled_module.function_instantiations[idx.0 as usize];
                        let func_key = get_function_key_from_handle(
                            func_inst.handle,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let func_idx = get_from_map!(&func_key, function_map);
                        let sig_idx = func_inst.type_parameters;
                        let params = &compiled_module.signatures[sig_idx.0 as usize];
                        let type_params = params
                            .0
                            .iter()
                            .map(|type_| type_builder.make_type(module, type_))
                            .collect::<Vec<_>>();
                        Bytecode::CallGeneric(func_idx, type_params)
                    }
                    MoveBytecode::Pack(idx) => {
                        let struct_key = get_struct_key_from_def(
                            *idx,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        Bytecode::Pack(struct_idx)
                    }
                    MoveBytecode::PackGeneric(idx) => {
                        let struct_inst =
                            &compiled_module.struct_def_instantiations[idx.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            struct_inst.def,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        let sig_idx = struct_inst.type_parameters;
                        let params = &compiled_module.signatures[sig_idx.0 as usize];
                        let type_params = params
                            .0
                            .iter()
                            .map(|type_| type_builder.make_type(module, type_))
                            .collect::<Vec<_>>();
                        Bytecode::PackGeneric(struct_idx, type_params)
                    }
                    MoveBytecode::Unpack(idx) => {
                        let struct_key = get_struct_key_from_def(
                            *idx,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        Bytecode::Unpack(struct_idx)
                    }
                    MoveBytecode::UnpackGeneric(idx) => {
                        let struct_inst =
                            &compiled_module.struct_def_instantiations[idx.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            struct_inst.def,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        let sig_idx = struct_inst.type_parameters;
                        let params = &compiled_module.signatures[sig_idx.0 as usize];
                        let type_params = params
                            .0
                            .iter()
                            .map(|type_| type_builder.make_type(module, type_))
                            .collect::<Vec<_>>();
                        Bytecode::UnpackGeneric(struct_idx, type_params)
                    }
                    MoveBytecode::MutBorrowLoc(idx) => Bytecode::MutBorrowLoc(*idx),
                    MoveBytecode::ImmBorrowLoc(idx) => Bytecode::ImmBorrowLoc(*idx),
                    MoveBytecode::MutBorrowField(idx) => {
                        let field_handle = &compiled_module.field_handles[idx.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            field_handle.owner,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        Bytecode::MutBorrowField(FieldRef {
                            struct_idx,
                            field_idx: field_handle.field,
                        })
                    }
                    MoveBytecode::MutBorrowFieldGeneric(idx) => {
                        let field_inst = &compiled_module.field_instantiations[idx.0 as usize];
                        let field_handle =
                            &compiled_module.field_handles[field_inst.handle.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            field_handle.owner,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        let params =
                            &compiled_module.signatures[field_inst.type_parameters.0 as usize];
                        let type_params = params
                            .0
                            .iter()
                            .map(|type_| type_builder.make_type(module, type_))
                            .collect::<Vec<_>>();
                        Bytecode::MutBorrowFieldGeneric(
                            FieldRef {
                                struct_idx,
                                field_idx: field_handle.field,
                            },
                            type_params,
                        )
                    }
                    MoveBytecode::ImmBorrowField(idx) => {
                        let field_handle = &compiled_module.field_handles[idx.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            field_handle.owner,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        Bytecode::ImmBorrowField(FieldRef {
                            struct_idx,
                            field_idx: field_handle.field,
                        })
                    }
                    MoveBytecode::ImmBorrowFieldGeneric(idx) => {
                        let field_inst = &compiled_module.field_instantiations[idx.0 as usize];
                        let field_handle =
                            &compiled_module.field_handles[field_inst.handle.0 as usize];
                        let struct_key = get_struct_key_from_def(
                            field_handle.owner,
                            compiled_module,
                            &type_builder.packages[module.package],
                        );
                        let struct_idx = get_from_map!(&struct_key, &type_builder.struct_map);
                        let params =
                            &compiled_module.signatures[field_inst.type_parameters.0 as usize];
                        let type_params = params
                            .0
                            .iter()
                            .map(|type_| type_builder.make_type(module, type_))
                            .collect::<Vec<_>>();
                        Bytecode::ImmBorrowFieldGeneric(
                            FieldRef {
                                struct_idx,
                                field_idx: field_handle.field,
                            },
                            type_params,
                        )
                    }
                    MoveBytecode::ReadRef => Bytecode::ReadRef,
                    MoveBytecode::WriteRef => Bytecode::WriteRef,
                    MoveBytecode::FreezeRef => Bytecode::FreezeRef,
                    MoveBytecode::VecPack(type_, count) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecPack(t, *count)
                    }
                    MoveBytecode::VecLen(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecLen(t)
                    }
                    MoveBytecode::VecImmBorrow(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecImmBorrow(t)
                    }
                    MoveBytecode::VecMutBorrow(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecMutBorrow(t)
                    }
                    MoveBytecode::VecPushBack(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecPushBack(t)
                    }
                    MoveBytecode::VecPopBack(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecPopBack(t)
                    }
                    MoveBytecode::VecUnpack(type_, count) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecUnpack(t, *count)
                    }
                    MoveBytecode::VecSwap(type_) => {
                        let type_sig = &compiled_module.signatures[type_.0 as usize];
                        let t = type_builder.make_type(module, &type_sig.0[0]);
                        Bytecode::VecSwap(t)
                    }
                    _ => panic!("Invalid bytecode found: {:?}", bytecode),
                })
                .collect();
            function.code = Some(Code { locals, code });
        }
    });
}

fn load_constants(type_builder: &TypeBuilder, modules: &mut [Module]) {
    modules.iter_mut().for_each(|module| {
        let compiled_module = module.module.as_ref().unwrap();
        module.constants = compiled_module
            .constant_pool
            .iter()
            .enumerate()
            .map(|(const_idx, constant)| {
                let type_ = type_builder.make_type(module, &constant.type_);
                Constant {
                    type_,
                    constant: ConstantPoolIndex(const_idx as u16),
                }
            })
            .collect::<Vec<_>>();
    });
}

fn get_value<V: Copy>(key: &String, map: &BTreeMap<String, V>) -> Result<V, PackageAnalyzerError> {
    map.get(key)
        .ok_or(PackageAnalyzerError::MissingKey(key.clone()))
        .copied()
}

fn get_function_key_from_handle(
    func_handle_idx: FunctionHandleIndex,
    compiled_module: &CompiledModule,
    package: &Package,
) -> String {
    let (module_name, func_name) =
        module_function_name_from_handle(compiled_module, func_handle_idx);
    let module_id = get_package_from_function_handle(compiled_module, func_handle_idx);
    let package_id = if &module_id == compiled_module.self_id().address() {
        // do not relocate calls inside the package
        package.id
    } else {
        let package_id = ObjectID::from(module_id);
        match package
            .package
            .as_ref()
            .unwrap()
            .linkage_table()
            .get(&package_id)
        {
            None => package_id,
            Some(upgrade_info) => upgrade_info.upgraded_id,
        }
    };
    format!("{}::{}::{}", package_id, module_name, func_name)
}

fn get_struct_key_from_def(
    struct_def_idx: StructDefinitionIndex,
    compiled_module: &CompiledModule,
    package: &Package,
) -> String {
    let (module_name, struct_name) = module_struct_name_from_def(compiled_module, struct_def_idx);
    let key = (module_name.to_string(), struct_name.to_string());
    let package_id = get_package_from_struct_def(compiled_module, struct_def_idx);
    let package_id = match package.type_origin.get(&key) {
        None => ObjectID::from(package_id),
        Some(package_id) => *package_id,
    };
    format!("{}::{}::{}", package_id, module_name, struct_name)
}
