// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::model::global_env::GlobalEnv;
use move_binary_format::file_format::{
    AbilitySet, CodeOffset, CompiledModule, ConstantPoolIndex, FunctionDefinitionIndex, LocalIndex,
    MemberCount, StructDefinitionIndex, StructTypeParameter, TypeParameterIndex, Visibility,
};
use move_core_types::{language_storage::ModuleId, u256::U256};
use std::collections::BTreeMap;
use sui_types::base_types::SequenceNumber;
use sui_types::{base_types::ObjectID, move_package::MovePackage};

// An index in one of the pools
pub type PackageIndex = usize;
pub type ModuleIndex = usize;
pub type StructIndex = usize;
pub type FunctionIndex = usize;
pub type IdentifierIndex = usize;

/// A package as known in the GlobalEnv.
/// Wraps a MovePackage and directly exposes some of its fields.
#[derive(Debug)]
pub struct Package {
    pub self_idx: PackageIndex,
    pub id: ObjectID, // The package is as known in Sui (DB, blockchain)
    pub package: Option<MovePackage>,
    pub version: Option<SequenceNumber>,
    pub type_origin: BTreeMap<(String, String), ObjectID>,
    // List of modules in this package as indices in the GlobalEnv.modules pool
    pub modules: Vec<ModuleIndex>,
}

impl Package {
    pub fn struct_count(&self, env: &GlobalEnv) -> usize {
        self.modules
            .iter()
            .map(|idx| env.modules[*idx].structs.len())
            .sum()
    }

    pub fn function_count(&self, env: &GlobalEnv) -> usize {
        self.modules
            .iter()
            .map(|idx| env.modules[*idx].functions.len())
            .sum()
    }
}

#[derive(Debug)]
pub struct Module {
    pub self_idx: ModuleIndex,
    pub package: PackageIndex,
    pub module: Option<CompiledModule>,
    pub name: IdentifierIndex,
    pub module_id: ModuleId,
    pub dependencies: Vec<ModuleIndex>,
    pub structs: Vec<StructIndex>,
    pub functions: Vec<FunctionIndex>,
    pub constants: Vec<Constant>,
}

#[derive(Debug)]
pub struct Struct {
    pub self_idx: StructIndex,
    pub package: PackageIndex,
    pub module: ModuleIndex,
    pub name: IdentifierIndex,
    pub def_idx: StructDefinitionIndex,
    pub abilities: AbilitySet,
    pub type_parameters: Vec<StructTypeParameter>,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Constant {
    pub type_: Type,
    pub constant: ConstantPoolIndex,
}

#[derive(Debug)]
pub struct Field {
    pub name: IdentifierIndex,
    pub type_: Type,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FieldRef {
    pub struct_idx: StructIndex,
    pub field_idx: MemberCount,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Type {
    /// Boolean, `true` or `false`.
    Bool,
    /// Unsigned integers, 8 bits length.
    U8,
    /// Unsigned integers, 16 bits length.
    U16,
    /// Unsigned integers, 32 bits length.
    U32,
    /// Unsigned integers, 64 bits length.
    U64,
    /// Unsigned integers, 128 bits length.
    U128,
    /// Unsigned integers, 256 bits length.
    U256,
    /// Address, a 16 bytes immutable type.
    Address,
    /// Vector
    Vector(Box<Type>),
    /// User defined type
    Struct(StructIndex),
    StructInstantiation(StructIndex, Vec<Type>),
    /// Reference to a type.
    Reference(Box<Type>),
    /// Mutable reference to a type.
    MutableReference(Box<Type>),
    /// Type parameter.
    TypeParameter(TypeParameterIndex),
}

#[derive(Debug)]
pub struct Function {
    pub self_idx: FunctionIndex,
    pub package: PackageIndex,
    pub module: ModuleIndex,
    pub name: IdentifierIndex,
    pub def_idx: FunctionDefinitionIndex,
    pub type_parameters: Vec<AbilitySet>,
    pub parameters: Vec<Type>,
    pub returns: Vec<Type>,
    pub visibility: Visibility,
    pub is_entry: bool,
    pub code: Option<Code>,
}

#[derive(Debug)]
pub struct Code {
    pub locals: Vec<Type>,
    pub code: Vec<Bytecode>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Bytecode {
    Nop,
    Pop,
    Ret,
    BrTrue(CodeOffset),
    BrFalse(CodeOffset),
    Branch(CodeOffset),
    LdConst(ConstantPoolIndex),
    LdTrue,
    LdFalse,
    LdU8(u8),
    LdU16(u16),
    LdU32(u32),
    LdU64(u64),
    LdU128(u128),
    LdU256(U256),
    CastU8,
    CastU16,
    CastU32,
    CastU64,
    CastU128,
    CastU256,
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    BitOr,
    BitAnd,
    Xor,
    Or,
    And,
    Not,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Shl,
    Shr,
    Abort,
    CopyLoc(LocalIndex),
    MoveLoc(LocalIndex),
    StLoc(LocalIndex),
    Call(FunctionIndex),
    CallGeneric(FunctionIndex, Vec<Type>),
    Pack(StructIndex),
    PackGeneric(StructIndex, Vec<Type>),
    Unpack(StructIndex),
    UnpackGeneric(StructIndex, Vec<Type>),
    MutBorrowLoc(LocalIndex),
    ImmBorrowLoc(LocalIndex),
    MutBorrowField(FieldRef),
    MutBorrowFieldGeneric(FieldRef, Vec<Type>),
    ImmBorrowField(FieldRef),
    ImmBorrowFieldGeneric(FieldRef, Vec<Type>),
    ReadRef,
    WriteRef,
    FreezeRef,
    VecPack(Type, u64),
    VecLen(Type),
    VecImmBorrow(Type),
    VecMutBorrow(Type),
    VecPushBack(Type),
    VecPopBack(Type),
    VecUnpack(Type, u64),
    VecSwap(Type),
}
