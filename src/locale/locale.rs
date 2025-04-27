use std::{
    collections::HashMap,
    fs::{self, read_dir, DirEntry},
};

use serde::Deserialize;
use tucana::shared::Translation;

use super::code::{code_from_file_name, CountryCode};

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
                let code = code_from_file_name(file_name.clone(), CountryCode::UnitedStates);
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
            *translations = translations
                .iter()
                .filter(|translation| translation.code == code)
                .cloned()
                .collect();
        }
    }

    pub fn reduce_to_accepted(&mut self) {
        let codes: Vec<String> = self
            .accepted_locales
            .iter()
            .map(|code| code.to_string())
            .collect();
        for (_, translations) in self.translations.iter_mut() {
            *translations = translations
                .iter()
                .filter(|translation| codes.contains(&translation.code.to_string()))
                .cloned()
                .collect();
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

    let file_path = format!("{}/{}", path, file_name_str);
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file {}: {}", file_path, e);
            return None;
        }
    };

    let base_name = match file_name_str.strip_suffix(".toml") {
        Some(name) => name.to_string(),
        None => return None,
    };

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
            code: code.clone().to_string(),
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
