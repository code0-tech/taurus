//! Renders [`build_modules`](crate::registry::build_modules)'s output as a
//! directory tree of JSON files -- one directory per module, containing
//! `module.json` plus one file per data type / function / runtime function --
//! using the same field names and camelCase wire format Aquila accepts,
//! since `tucana::shared`'s `Serialize` impls already implement protobuf's
//! canonical JSON mapping. Mirrors the layout the old `definitions/*.json`
//! tree used, so this is primarily a debugging/parity-check tool: run it and
//! diff the result against a known-good definitions dump.

use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;
use tucana::shared::{Module, Translation};

/// Writes one subdirectory per module under `root`, named by the module's
/// identifier.
pub fn write_all(modules: &[Module], root: &Path) -> io::Result<()> {
    for module in modules {
        write(module, &root.join(&module.identifier))?;
    }
    Ok(())
}

fn write(module: &Module, dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    write_json(&dir.join("module.json"), &Meta::from(module))?;

    write_each(
        &dir.join("data_types"),
        &module.definition_data_types,
        |dt| &dt.identifier,
    )?;
    write_each(&dir.join("functions"), &module.function_definitions, |f| {
        &f.runtime_name
    })?;
    write_each(
        &dir.join("runtime_functions"),
        &module.runtime_function_definitions,
        |f| &f.runtime_name,
    )?;

    Ok(())
}

#[derive(Serialize)]
struct Meta<'a> {
    identifier: &'a str,
    name: &'a [Translation],
    description: &'a [Translation],
    documentation: &'a str,
    author: &'a str,
    icon: &'a str,
    version: &'a str,
}

impl<'a> From<&'a Module> for Meta<'a> {
    fn from(module: &'a Module) -> Self {
        Self {
            identifier: &module.identifier,
            name: &module.name,
            description: &module.description,
            documentation: &module.documentation,
            author: &module.author,
            icon: &module.icon,
            version: &module.version,
        }
    }
}

fn write_each<T: Serialize>(
    dir: &Path,
    items: &[T],
    identifier: impl Fn(&T) -> &str,
) -> io::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    for item in items {
        let file_name = identifier(item).replace("::", "_");
        write_json(&dir.join(format!("{file_name}.json")), item)?;
    }
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use tucana::shared::{DefinitionDataType, Module, Translation};

    use super::write_all;

    fn sample_modules() -> Vec<Module> {
        vec![Module {
            identifier: "example-module".into(),
            version: "0.1.0".into(),
            author: "code0-tech".into(),
            icon: "tabler:bolt".into(),
            documentation: "An example".into(),
            name: vec![Translation {
                code: "en-US".into(),
                content: "Example".into(),
            }],
            definition_data_types: vec![DefinitionDataType {
                identifier: "EMAIL".into(),
                r#type: "string".into(),
                version: "0.1.0".into(),
                ..Default::default()
            }],
            ..Default::default()
        }]
    }

    #[test]
    fn writes_one_directory_per_module_with_one_file_per_definition() {
        let dir = std::env::temp_dir().join(format!("taurus-export-test-{}", std::process::id()));
        write_all(&sample_modules(), &dir).expect("export succeeds");

        let meta: serde_json::Value = serde_json::from_slice(
            &std::fs::read(dir.join("example-module").join("module.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(meta["identifier"], "example-module");

        let email: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                dir.join("example-module")
                    .join("data_types")
                    .join("EMAIL.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(email["identifier"], "EMAIL");

        // No functions were registered, so that directory shouldn't exist.
        assert!(!dir.join("example-module").join("functions").exists());

        std::fs::remove_dir_all(&dir).ok();
    }
}
