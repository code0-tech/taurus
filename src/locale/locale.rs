use std::{
    collections::HashMap,
    fs::{self, read_dir},
};

use tucana::shared::Translation;

use super::code::{code_from_file_name, CountryCode};

pub struct Locale {
    pub translations: HashMap<String, Vec<Translation>>,
    pub accepted_locales: Vec<CountryCode>,
    pub default_locale: CountryCode,
    pub path: String,
}

pub struct TranslationMissingError;

impl Locale {
    /*
        Todo.
        - find better way then json to save the language json!

        add the following functions:
         - get tranlsations for certain key
         - reduce the list of translation to only the accepted locales
         - add a new function
    */

    pub fn default() -> Self {
        let path = "./translation";
        let translations: HashMap<String, Vec<Translation>> = HashMap::new();
        let mut accepted_locales: Vec<CountryCode> = vec![];
        let entries = read_dir(path).expect("msg");

        for entry in entries {
            let entry = entry.expect("msg");
            let meta = entry.metadata().expect("msg");

            if meta.is_file() {
                let file_name = entry.file_name();

                let mut real_file_name = match file_name.to_str() {
                    Some(name) => name,
                    None => continue,
                };
                let file_path = format!("{}/{}", path, real_file_name);
                let file = fs::File::open(&file_path).expect("file should open read only");

                real_file_name = match real_file_name.strip_suffix(".json") {
                    Some(name) => name,
                    None => continue,
                };

                let code = code_from_file_name(
                    real_file_name.to_string().clone(),
                    CountryCode::UnitedStates,
                );
                accepted_locales.push(code);

                let json: serde_json::Value =
                    serde_json::from_reader(file).expect("file should be proper JSON");

                for entry in json.as_object().expect("msg").into_iter() {
                    let key = entry.0.to_string();
                    let value = entry.1.as_str().expect("msg");

                    let translation = Translation {
                        code: code.to_string(),
                        content: value.to_string(),
                    };

                    let vec = translations.get(&key.to_string());

                    match vec {
                        Some(trans) => trans.push(translation),
                        None => {
                            translations.insert(key.clone(), vec![translation]);
                        }
                    }
                }
            }
        }

        Locale {
            translations,
            accepted_locales,
            default_locale: CountryCode::UnitedStates,
            path: path.to_string(),
        }
    }
}
