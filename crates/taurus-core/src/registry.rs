//! Assembles `tucana::shared::Module`s from the `inventory`-collected
//! metadata in [`crate::meta`] -- the Rust-native replacement for reading
//! `definitions/*.json` off disk. This is pure domain data assembly; sending
//! the result anywhere (Aquila, a file, ...) is a transport concern that
//! belongs to the caller.

use crate::meta::{
    DataTypeMeta, DataTypeRegistration, MetaRegistration, ModuleMeta, ModuleRegistration,
    RuntimeFunctionMeta,
};
use tucana::shared::{
    DefinitionDataType, FunctionDefinition, Module, ParameterDefinition, RuntimeFunctionDefinition,
    RuntimeParameterDefinition,
};

/// Builds every registered module, complete with its function and data-type
/// definitions. Panics if a function or data type declares a `module` that
/// has no matching `taurus_macros::module!` registration -- a broken link
/// between a handler and its owning module is a programming error, not a
/// runtime condition to recover from.
pub fn build_modules() -> Vec<Module> {
    let mut modules: Vec<Module> = inventory::iter::<ModuleRegistration>()
        .map(|reg| module_from_meta((reg.0)()))
        .collect();

    for reg in inventory::iter::<MetaRegistration>() {
        let meta = (reg.0)();
        let module = find_module(&mut modules, meta.module, meta.identifier);
        let version = module.version.clone();
        module
            .runtime_function_definitions
            .push(runtime_function_definition(&meta, version.clone()));
        module
            .function_definitions
            .push(function_definition(&meta, version));
    }

    for reg in inventory::iter::<DataTypeRegistration>() {
        let meta = (reg.0)();
        let module = find_module(&mut modules, meta.module, meta.identifier);
        let version = module.version.clone();
        module
            .definition_data_types
            .push(data_type_definition(&meta, version));
    }

    modules
}

fn find_module<'a>(modules: &'a mut [Module], module_id: &str, owner_id: &str) -> &'a mut Module {
    modules
        .iter_mut()
        .find(|m| m.identifier == module_id)
        .unwrap_or_else(|| {
            panic!(
                "`{owner_id}` declares module `{module_id}`, which has no `taurus_macros::module!` registration"
            )
        })
}

fn module_from_meta(meta: ModuleMeta) -> Module {
    Module {
        identifier: meta.identifier.to_string(),
        name: meta.name,
        description: meta.description,
        documentation: meta.documentation.to_string(),
        author: meta.author.to_string(),
        icon: meta.icon.to_string(),
        version: meta.version.to_string(),
        ..Default::default()
    }
}

fn runtime_function_definition(
    meta: &RuntimeFunctionMeta,
    version: String,
) -> RuntimeFunctionDefinition {
    RuntimeFunctionDefinition {
        runtime_name: meta.identifier.to_string(),
        runtime_parameter_definitions: meta
            .parameters
            .iter()
            .map(|p| RuntimeParameterDefinition {
                runtime_name: p.runtime_name.to_string(),
                default_value: None,
                optional: None,
                hidden: None,
                name: p.name.clone(),
                description: p.description.clone(),
                documentation: p.documentation.clone(),
            })
            .collect(),
        signature: meta.signature.to_string(),
        throws_error: meta.throws_error,
        name: meta.name.clone(),
        description: meta.description.clone(),
        documentation: meta.documentation.clone(),
        deprecation_message: Vec::new(),
        display_message: meta.display_message.clone(),
        alias: meta.alias.clone(),
        linked_data_type_identifiers: meta
            .linked_data_type_identifiers
            .iter()
            .map(|s| s.to_string())
            .collect(),
        version,
        display_icon: meta.display_icon.unwrap_or_default().to_string(),
        definition_source: String::new(),
        design: None,
    }
}

fn function_definition(meta: &RuntimeFunctionMeta, version: String) -> FunctionDefinition {
    FunctionDefinition {
        runtime_name: meta.identifier.to_string(),
        parameter_definitions: meta
            .parameters
            .iter()
            .map(|p| ParameterDefinition {
                runtime_name: p.runtime_name.to_string(),
                default_value: None,
                optional: None,
                hidden: None,
                name: p.name.clone(),
                description: p.description.clone(),
                documentation: p.documentation.clone(),
                runtime_definition_name: p.runtime_name.to_string(),
            })
            .collect(),
        signature: meta.signature.to_string(),
        throws_error: meta.throws_error,
        name: meta.name.clone(),
        description: meta.description.clone(),
        documentation: meta.documentation.clone(),
        deprecation_message: Vec::new(),
        display_message: meta.display_message.clone(),
        alias: meta.alias.clone(),
        linked_data_type_identifiers: meta
            .linked_data_type_identifiers
            .iter()
            .map(|s| s.to_string())
            .collect(),
        version,
        display_icon: meta.display_icon.unwrap_or_default().to_string(),
        definition_source: String::new(),
        runtime_definition_name: meta.identifier.to_string(),
        design: None,
    }
}

fn data_type_definition(meta: &DataTypeMeta, version: String) -> DefinitionDataType {
    DefinitionDataType {
        identifier: meta.identifier.to_string(),
        name: meta.name.clone(),
        display_message: meta.display_message.clone(),
        alias: meta.alias.clone(),
        rules: meta.rules.clone(),
        generic_keys: meta.generic_keys.iter().map(|s| s.to_string()).collect(),
        r#type: meta.type_string.to_string(),
        linked_data_type_identifiers: meta
            .linked_data_type_identifiers
            .iter()
            .map(|s| s.to_string())
            .collect(),
        version,
        definition_source: String::new(),
    }
}
