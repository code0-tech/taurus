use std::{
    collections::HashMap,
    fs::{self, DirEntry, read_dir},
};

use serde::Deserialize;
use tucana::shared::Translation;

use super::code::{CountryCode, code_from_file_name};

#[derive(Debug)]
pub struct Locale {
    translations: HashMap<String, Vec<Translation>>,
    accepted_locales: Vec<CountryCode>,
    default_locale: CountryCode,
}

pub struct TranslationMissingError;

#[derive(Deserialize)]
pub struct Translations {
    #[serde(flatten)]
    pub entries: HashMap<String, String>,
}

impl Locale {
    pub fn default() -> Self {
        let path = "./translation";
        let mut dictionary: HashMap<String, Vec<Translation>> = HashMap::new();
        let mut accepted_locales: Vec<CountryCode> = vec![];

        let entries = match read_dir(path) {
            Ok(entries) => entries,
            Err(e) => panic!("Failed to read translation directory: {}", e),
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            if !is_translation_file(&entry) {
                continue;
            }

            if let Some((file_name, content)) = read_translation_file(path, &entry) {
                let code = code_from_file_name(file_name.clone(), CountryCode::UnitedStates);
                accepted_locales.push(code.clone());

                process_translation_file(&content, &file_name, &code, &mut dictionary);
            }
        }

        Locale {
            translations: dictionary,
            accepted_locales,
            default_locale: CountryCode::UnitedStates,
        }
    }

    pub fn new(
        path: &str,
        accepted_locales: Vec<CountryCode>,
        default_locale: CountryCode,
    ) -> Self {
        let mut dictionary = HashMap::new();

        let entries = match read_dir(path) {
            Ok(entries) => entries,
            Err(e) => panic!("Failed to read translation directory: {}", e),
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            if !is_translation_file(&entry) {
                continue;
            }

            if let Some((file_name, content)) = read_translation_file(path, &entry) {
                let code = code_from_file_name(file_name.clone(), default_locale.clone());
                if !accepted_locales.contains(&code) {
                    continue;
                }

                process_translation_file(&content, &file_name, &code, &mut dictionary);
            }
        }

        Locale {
            translations: dictionary,
            accepted_locales,
            default_locale,
        }
    }

    pub fn reduce_to_default(&mut self) {
        let code = self.default_locale.to_string();
        for (_, translations) in self.translations.iter_mut() {
            translations.retain(|translation| translation.code == code);
        }
    }

    pub fn reduce_to_accepted(&mut self) {
        let codes: Vec<String> = self
            .accepted_locales
            .iter()
            .map(|code| code.to_string())
            .collect();
        for (_, translations) in self.translations.iter_mut() {
            translations.retain(|translation| codes.contains(&translation.code));
        }
    }

    pub fn get_translations(&self, key: String) -> Option<Vec<Translation>> {
        self.translations.get(&key).cloned()
    }

    pub fn get_dictionary(&self) -> HashMap<String, Vec<Translation>> {
        self.translations.clone()
    }
}

// Check if entry is a file that we should process
fn is_translation_file(entry: &DirEntry) -> bool {
    match entry.metadata() {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    }
}

// Read and parse a translation file
fn read_translation_file(path: &str, entry: &DirEntry) -> Option<(String, String)> {
    let file_name = entry.file_name();

    let file_name_str = match file_name.to_str() {
        Some(name) => name,
        None => return None,
    };

    // Check if it's a .toml file
    if !file_name_str.ends_with(".toml") {
        return None;
    }

    let file_path = format!("{}/{}", path, file_name_str);
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", file_path, e);
            return None;
        }
    };

    let base_name = file_name_str.strip_suffix(".toml").unwrap().to_string();
    Some((base_name, content))
}

// Process the content of a translation file
fn process_translation_file(
    content: &str,
    file_name: &str,
    code: &CountryCode,
    dictionary: &mut HashMap<String, Vec<Translation>>,
) {
    match toml::from_str::<toml::Value>(content) {
        Ok(value) => {
            // Process nested TOML by flattening it
            let flattened = flatten_toml(&value, "");
            add_translations_to_dictionary(flattened, code, dictionary);
        }
        Err(err) => {
            eprintln!("Warning: Error parsing '{}': {}", file_name, err);
        }
    }
}

// Add translations to the dictionary
fn add_translations_to_dictionary(
    flattened: HashMap<String, String>,
    code: &CountryCode,
    dictionary: &mut HashMap<String, Vec<Translation>>,
) {
    for (key, entry) in flattened {
        let translation = Translation {
            code: code.to_string(),
            content: entry,
        };

        dictionary
            .entry(key)
            .or_insert_with(Vec::new)
            .push(translation);
    }
}

// Helper function to flatten nested TOML structures
fn flatten_toml(value: &toml::Value, prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    match value {
        toml::Value::Table(table) => {
            for (key, val) in table {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                match val {
                    toml::Value::Table(_) => {
                        let nested = flatten_toml(val, &new_prefix);
                        result.extend(nested);
                    }
                    _ => {
                        extract_value_to_string(val, &new_prefix, &mut result);
                    }
                }
            }
        }
        _ => {
            if !prefix.is_empty() {
                extract_value_to_string(value, prefix, &mut result);
            }
        }
    }

    result
}

// Extract a TOML value into a string
fn extract_value_to_string(val: &toml::Value, key: &str, result: &mut HashMap<String, String>) {
    if let Some(str_val) = val.as_str() {
        result.insert(key.to_string(), str_val.to_string());
    } else if let Some(string_val) = val
        .to_string()
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
    {
        result.insert(key.to_string(), string_val.to_string());
    } else {
        result.insert(key.to_string(), val.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_test_translation_file(
        dir: &Path,
        filename: &str,
        content: &str,
    ) -> std::io::Result<()> {
        let file_path = dir.join(filename);
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_flatten_toml() {
        let toml_str = r#"
            key1 = "value1"

            [section1]
            key2 = "value2"

            [section1.subsection]
            key3 = "value3"
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let flattened = flatten_toml(&value, "");

        assert_eq!(flattened.get("key1"), Some(&"value1".to_string()));
        assert_eq!(flattened.get("section1.key2"), Some(&"value2".to_string()));
        assert_eq!(
            flattened.get("section1.subsection.key3"),
            Some(&"value3".to_string())
        );
    }

    #[test]
    fn test_add_translations_to_dictionary() {
        let mut flattened = HashMap::new();
        flattened.insert("key1".to_string(), "value1".to_string());
        flattened.insert("key2".to_string(), "value2".to_string());

        let code = CountryCode::UnitedStates;
        let mut dictionary = HashMap::new();

        add_translations_to_dictionary(flattened, &code, &mut dictionary);

        assert_eq!(dictionary.len(), 2);
        assert_eq!(dictionary["key1"][0].content, "value1");
        assert_eq!(dictionary["key1"][0].code, code.to_string());
        assert_eq!(dictionary["key2"][0].content, "value2");
        assert_eq!(dictionary["key2"][0].code, code.to_string());
    }

    #[test]
    fn test_reduce_to_default_empty() {
        let mut translations = HashMap::new();

        let key = "greeting".to_string();
        let mut values = Vec::new();

        values.push(Translation {
            code: CountryCode::Germany.to_string(),
            content: "Hallo".to_string(),
        });

        values.push(Translation {
            code: CountryCode::France.to_string(),
            content: "Bonjour".to_string(),
        });

        translations.insert(key.clone(), values);

        let mut locale = Locale {
            translations,
            accepted_locales: vec![CountryCode::UnitedStates, CountryCode::France],
            default_locale: CountryCode::UnitedStates,
        };

        locale.reduce_to_default();

        let translations = locale.get_translations(key);
        assert!(translations.is_some());
        assert_eq!(translations.unwrap().len(), 0);
    }

    #[test]
    fn test_reduce_to_default() {
        let mut translations = HashMap::new();

        let key = "greeting".to_string();
        let mut values = Vec::new();

        values.push(Translation {
            code: CountryCode::Germany.to_string(),
            content: "Hallo".to_string(),
        });

        values.push(Translation {
            code: CountryCode::France.to_string(),
            content: "Bonjour".to_string(),
        });

        values.push(Translation {
            code: CountryCode::UnitedStates.to_string(),
            content: "Hello".to_string(),
        });

        translations.insert(key.clone(), values);

        let mut locale = Locale {
            translations,
            accepted_locales: vec![CountryCode::UnitedStates, CountryCode::France],
            default_locale: CountryCode::UnitedStates,
        };

        locale.reduce_to_default();

        let translations = locale.get_translations(key);
        assert!(translations.is_some());
        assert_eq!(translations.unwrap().len(), 1);
    }

    #[test]
    fn test_reduce_to_accepted() {
        let mut translations = HashMap::new();

        let key = "greeting".to_string();
        let mut values = Vec::new();

        values.push(Translation {
            code: CountryCode::UnitedStates.to_string(),
            content: "Hello".to_string(),
        });

        values.push(Translation {
            code: CountryCode::France.to_string(),
            content: "Bonjour".to_string(),
        });

        values.push(Translation {
            code: CountryCode::Germany.to_string(),
            content: "Hallo".to_string(),
        });

        translations.insert(key.clone(), values);

        let mut locale = Locale {
            translations,
            accepted_locales: vec![CountryCode::UnitedStates, CountryCode::France],
            default_locale: CountryCode::UnitedStates,
        };

        locale.reduce_to_accepted();

        let translations = locale.get_translations(key);
        assert!(translations.is_some());
        assert_eq!(translations.unwrap().len(), 2);
    }

    #[test]
    fn test_reduce_to_accepted_empty() {
        let mut translations = HashMap::new();

        let key = "greeting".to_string();
        let mut values = Vec::new();

        values.push(Translation {
            code: CountryCode::UnitedStates.to_string(),
            content: "Hello".to_string(),
        });

        values.push(Translation {
            code: CountryCode::France.to_string(),
            content: "Bonjour".to_string(),
        });

        translations.insert(key.clone(), values);

        let mut locale = Locale {
            translations,
            accepted_locales: vec![CountryCode::Germany],
            default_locale: CountryCode::UnitedStates,
        };

        locale.reduce_to_accepted();

        let translations = locale.get_translations(key);
        assert!(translations.is_some());
        assert_eq!(translations.unwrap().len(), 0);
    }

    #[test]
    fn test_locale_new_with_files() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create test translation files
        let en_file = create_test_translation_file(
            temp_path,
            "en_US.toml",
            r#"
            welcome = "Welcome"
            goodbye = "Goodbye"
            "#,
        );

        assert!(en_file.is_ok());

        let fr_file = create_test_translation_file(
            temp_path,
            "fr_FR.toml",
            r#"
            welcome = "Bienvenue"
            goodbye = "Au revoir"
            "#,
        );

        assert!(fr_file.is_ok());

        let locale = Locale::new(
            temp_path.to_str().unwrap(),
            vec![CountryCode::UnitedStates, CountryCode::France],
            CountryCode::UnitedStates,
        );

        assert_eq!(locale.accepted_locales.len(), 2);
        assert!(locale.accepted_locales.contains(&CountryCode::UnitedStates));
        assert!(locale.accepted_locales.contains(&CountryCode::France));
        assert_eq!(
            locale.default_locale,
            CountryCode::UnitedStates,
            "Not the same default locale"
        );

        // Test that translations were loaded correctly
        let welcome_translations = locale.get_translations("welcome".to_string()).unwrap();
        let goodbye_translations = locale.get_translations("goodbye".to_string()).unwrap();
        let empty_translations = locale.get_translations("empty".to_string());
        assert_eq!(welcome_translations.len(), 2);
        assert_eq!(goodbye_translations.len(), 2);
        assert!(empty_translations.is_none());
    }
}
