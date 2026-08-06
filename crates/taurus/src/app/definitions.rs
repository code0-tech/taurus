//! Builds and pushes Taurus's module definitions to Aquila.
//!
//! Definitions used to be read from a `definitions/*.json` tree on disk via
//! `code0_flow::flow_definition::Reader`. Every implemented runtime function,
//! data type, and module now self-registers its own metadata (see
//! `taurus_core::registry::build_modules`), so this module builds the
//! `tucana::shared::Module`s in-memory and sends them directly -- reusing
//! `code0_flow::flow_service`'s channel/auth helpers, but not its
//! JSON-reading entry point.

use std::time::Duration;

use tonic::{Extensions, Request};
use tucana::aquila::{ModuleUpdateRequest, module_service_client::ModuleServiceClient};
use tucana::shared::Module;

use crate::config::Config;
use crate::telemetry::errors;

/// Identifier of the service that defines these definitions, stamped onto
/// every function/data-type definition before it's sent -- mirrors what
/// `FlowUpdateService::with_definition_source` used to do.
const DEFINITION_SOURCE: &str = "taurus";

fn modules_with_definition_source() -> Vec<Module> {
    taurus_core::registry::build_modules()
        .into_iter()
        .map(apply_definition_source)
        .collect()
}

fn apply_definition_source(mut module: Module) -> Module {
    module.definition_source = DEFINITION_SOURCE.to_string();
    for function in &mut module.function_definitions {
        function.definition_source = DEFINITION_SOURCE.to_string();
    }
    for runtime_function in &mut module.runtime_function_definitions {
        runtime_function.definition_source = DEFINITION_SOURCE.to_string();
    }
    for data_type in &mut module.definition_data_types {
        data_type.definition_source = DEFINITION_SOURCE.to_string();
    }
    module
}

/// Identifiers of every registered module, for the runtime-status heartbeat.
pub fn module_identifiers() -> Vec<String> {
    taurus_core::registry::build_modules()
        .into_iter()
        .map(|module| module.identifier)
        .filter(|identifier| !identifier.is_empty())
        .collect()
}

pub async fn push_definitions_until_success(config: &Config) {
    let modules = modules_with_definition_source();
    if modules.is_empty() {
        log::info!("No modules are registered, aborting definitions update.");
        return;
    }

    let channel = code0_flow::flow_service::retry::create_channel_with_retry(
        "Aquila",
        config.aquila_url.clone(),
        Duration::from_secs(config.aquila_grpc_connect_timeout_secs),
        Duration::from_secs(config.aquila_grpc_request_timeout_secs),
    )
    .await;
    let mut client = ModuleServiceClient::new(channel);

    let mut retry_count = 1;
    loop {
        let request = Request::from_parts(
            code0_flow::flow_service::auth::get_authorization_metadata(&config.aquila_token),
            Extensions::new(),
            ModuleUpdateRequest {
                modules: modules.clone(),
            },
        );

        match client.update(request).await {
            Ok(response) if response.into_inner().success => {
                log::info!("Module definition update has been successful");
                break;
            }
            Ok(_) => log::warn!("Module definition update has been unsuccessful"),
            Err(err) => {
                log::error!("Module definition update failed. Reason: {:?}", err);
                errors::record(
                    "transport",
                    "definitions.update",
                    &err,
                    "service=aquila",
                );
            }
        }

        log::warn!(
            "Updating definitions failed, trying again in 3 seconds (retry #{})",
            retry_count
        );
        retry_count += 1;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
