//! Metadata that used to live in `definitions/*.json`, now declared inline
//! via `#[taurus_macros::runtime_function]` / `taurus_macros::data_type!` /
//! `taurus_macros::module!` next to the code it describes, and collected at
//! startup through `inventory` instead of being read from disk. See
//! [`crate::registry::build_modules`] for how this gets turned into
//! `tucana::shared::Module`s.

use tucana::shared::{DefinitionDataTypeRule, Translation};

pub struct ParameterMeta {
    pub runtime_name: &'static str,
    pub name: Vec<Translation>,
    pub description: Vec<Translation>,
}

pub struct RuntimeFunctionMeta {
    pub identifier: &'static str,
    /// Identifier of the [`ModuleMeta`] this function belongs to.
    pub module: &'static str,
    pub signature: &'static str,
    pub name: Vec<Translation>,
    pub description: Vec<Translation>,
    pub documentation: Vec<Translation>,
    pub display_message: Vec<Translation>,
    pub alias: Vec<Translation>,
    pub display_icon: Option<&'static str>,
    pub throws_error: bool,
    pub parameters: Vec<ParameterMeta>,
    pub linked_data_type_identifiers: Vec<&'static str>,
}

pub struct DataTypeMeta {
    pub identifier: &'static str,
    /// Identifier of the [`ModuleMeta`] this data type belongs to.
    pub module: &'static str,
    pub name: Vec<Translation>,
    pub display_message: Vec<Translation>,
    pub alias: Vec<Translation>,
    pub generic_keys: Vec<&'static str>,
    /// Hand-authored structural type string (e.g. `"boolean"`, or a
    /// TypeScript-like conditional type for generic data types) -- data
    /// types have no Rust type to derive this from.
    pub type_string: &'static str,
    pub linked_data_type_identifiers: Vec<&'static str>,
    pub rules: Vec<DefinitionDataTypeRule>,
}

pub struct ModuleMeta {
    pub identifier: &'static str,
    pub name: Vec<Translation>,
    pub description: Vec<Translation>,
    pub documentation: &'static str,
    pub author: &'static str,
    pub icon: &'static str,
    pub version: &'static str,
}

pub struct MetaRegistration(pub fn() -> RuntimeFunctionMeta);
inventory::collect!(MetaRegistration);

pub struct DataTypeRegistration(pub fn() -> DataTypeMeta);
inventory::collect!(DataTypeRegistration);

pub struct ModuleRegistration(pub fn() -> ModuleMeta);
inventory::collect!(ModuleRegistration);
