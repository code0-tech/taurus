use std::ffi::OsString;

pub enum CountryCode {
    Germany,
    UnitedStates,
}

impl ToString for CountryCode {
    fn to_string(&self) -> String {
        match self {
            CountryCode::Germany => "de-DE".to_string(),
            CountryCode::UnitedStates => "en-US".to_string(),
        }
    }
}

pub fn code_from_file_name(file_name: String, default: CountryCode) -> CountryCode {
    match file_name.as_str() {
        "de-DE" => CountryCode::Germany,
        "en-US" => CountryCode::UnitedStates,
        _ => default,
    }
}
