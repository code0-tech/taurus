use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;
use tucana::shared::value::Kind;

use tucana::shared::{
    DataType, RuntimeFunctionDefinition, RuntimeParameterDefinition, Translation, Value,
};

macro_rules! print_on_build {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

/// Given an actual `Value`, emit the tokens to *reconstruct* it in code.
pub fn value_to_tokens(value: Value) -> TokenStream {
    todo!("correct struct imports & strings");
    match &value.kind {
        Some(Kind::BoolValue(b)) => {
            quote! {
                Value {
                    kind: Some(Kind::BoolValue(#b))
                }
            }
        }
        Some(Kind::NumberValue(n)) => {
            quote! {
                Value {
                    kind: Some(Kind::NumberValue(#n))
                }
            }
        }
        Some(Kind::StringValue(s)) => {
            quote! {
                Value { kind: Some(Kind::StringValue(#s.to_string())) }
            }
        }
        Some(Kind::StructValue(str)) => {
            let list_item_tokens: Vec<TokenStream> = str
                .fields
                .iter()
                .map(|(k, v)| {
                    let v_ts = value_to_tokens(v.clone());
                    quote! {
                        (#k.to_string(), #v_ts)
                    }
                })
                .collect();

            quote! {
                Value {
                    kind: Some(ValueKind::ListValue(Struct {
                        fields: {
                            ::std::collections::HashMap::from_iter(vec![ #(#list_item_tokens),* ])
                        }
                    }))
                }
            }
        }
        Some(Kind::ListValue(lv)) => {
            // embed each item of the real list
            let item_tokens: Vec<_> = lv
                .values
                .iter()
                .map(|f| value_to_tokens(f.clone()))
                .collect();
            quote! {
                Value {
                    kind: Some(ValueKind::ListValue(vec![
                        #(#item_tokens),*
                    ]))
                }
            }
        }
        Some(Kind::NullValue(_)) | None => {
            quote! {
                Value { kind: Some(ValueKind::NullValue(NullValue::NullValue as i32)) }
            }
        }
    }
}

fn translation_to_token(translations: Vec<Translation>) -> Vec<TokenStream> {
    let mut result: Vec<TokenStream> = Vec::new();

    for trans in translations {
        let code = trans.code;
        let content = trans.content;

        let token = quote! {
            tucana::shared::Translation {
                code: String::from(#code),
                content: String::from(#content),
            }
        };

        result.push(token);
    }
    result
}

fn runtime_function_parameter_to_token(
    paramter: Vec<RuntimeParameterDefinition>,
) -> Vec<TokenStream> {
    let mut result: Vec<TokenStream> = Vec::new();

    for param in paramter {
        let runtime_name = param.runtime_name;
        let data_type_identifier = param.data_type_identifier;
        let default_value = match param.default_value {
            Some(value) => value_to_tokens(value),
            None => quote! { None },
        };
        let name = translation_to_token(param.name);
        let description = translation_to_token(param.description);
        let documentation = translation_to_token(param.documentation);

        let quote = quote! {
            /*
            tucana::shared::RuntimeParameterDefinition {
                runtime_name: String::from(#runtime_name),
                data_type_identifier: String::from(#data_type_identifier),
                default_value: #default_value,
                name: vec![#(#name),*],
                description: vec![#(#description),*],
                documentation: vec![#(#documentation),*]
            }
            */
        };

        result.push(quote);
    }
    result
}

fn runtime_function_definition_to_token(definition: RuntimeFunctionDefinition) -> TokenStream {
    let runtime_name = definition.runtime_name;
    let runtime_parameter_definitions =
        runtime_function_parameter_to_token(definition.runtime_parameter_definitions);

    // let return_type_identifier = definition.return_type_identifier.into_token_stream();

    let error_type_identifiers = definition
        .error_type_identifiers
        .into_iter()
        .map(|f| {
            quote! {
                #f
            }
        })
        .into_iter()
        .collect::<Vec<TokenStream>>();

    let name = translation_to_token(definition.name);
    let description = translation_to_token(definition.description);
    let documentation = translation_to_token(definition.documentation);
    let deprecation_message = translation_to_token(definition.deprecation_message);

    quote! {
        tucana::shared::RuntimeFunctionDefinition {
            runtime_name: String::from(#runtime_name),
            runtime_parameter_definitions: vec![#(#runtime_parameter_definitions),*],
        //    return_type_identifier: Option::Some(String::from(#return_type_identifier)),
            error_type_identifiers: vec![#(#error_type_identifiers),*],
            name: vec![#(#name),*],
            description: vec![#(#description),*],
            documentation: vec![#(#documentation),*],
            deprecation_message: vec![#(#deprecation_message),*],
        }
    }
}
fn main() {
    // let mut file = File::create("./out/output.rs").expect("msg");

    let path = "./definitions/runtime_functions/array/array.md";
    let file_content = read_to_string(path).unwrap();
    let mut lines = file_content.split("\n");
    let mut inside_code_block = false;
    let mut code_blocks: Vec<String> = Vec::new();
    let mut current_code_block: String = String::from("");

    while let Some(line) = lines.next() {
        if line.contains("```") {
            if inside_code_block {
                code_blocks.push(current_code_block.clone());
            } else {
                current_code_block = String::from("")
            }

            inside_code_block = !inside_code_block;
        }

        if inside_code_block {
            if line.starts_with("```") {
                continue;
            }

            current_code_block.push_str(line);
        }
    }

    for code in code_blocks {
        match serde_json::from_str::<DataType>(&code) {
            Ok(def) => {
                //   let quote = runtime_function_definition_to_token(def);
                //   write!(file, "{},", quote).expect("Cannot write to file");
            }
            Err(err) => {
                print_on_build!("Error parsing JSON: {:?}", err);
                print_on_build!("JSON: {:?}", code);
            }
        }
    }

    /*
    write!(
        file,
        "pub mod output {{ fn getDefinitions() -> Vec<tucana::shared::RuntimeFunctionDefinition> {{ vec!["
    )
    .expect("Cannot write to file");

    for res in results {
        match res {
            Ok(def) => {
                let quote = runtime_function_definition_to_token(def);
                write!(file, "{},", quote).expect("Cannot write to file");
            }
            Err(err) => {
                print_on_build!("Error parsing JSON: {}", err);
            }
        }
    }

    write!(file, "] }} }}").expect("Cannot write to file");

    Command::new("rustfmt")
        .arg("./out/output.rs")
        .arg("--edition")
        .arg("2024");
    */
}
