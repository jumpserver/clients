use serde::Serialize;
use std::time::Instant;
use tauri::State;

use crate::api::client::request_builder_client;
use crate::service::proxy::{
    proxy_test_url, ProxyManager, ProxyResolverKind, ProxyRouteKind, ProxySettings,
    ProxySettingsInput,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,
    elapsed_ms: u64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    route: Option<ProxyRouteKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolver: Option<ProxyResolverKind>,
    fallback_attempts: usize,
}

#[tauri::command]
pub fn get_proxy_settings(manager: State<'_, ProxyManager>) -> ProxySettings {
    manager.settings()
}

#[tauri::command]
pub async fn update_proxy_settings(
    manager: State<'_, ProxyManager>,
    settings: ProxySettingsInput,
) -> Result<ProxySettings, String> {
    manager
        .update(settings)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn test_proxy_settings(
    manager: State<'_, ProxyManager>,
    settings: ProxySettingsInput,
    target_url: String,
) -> Result<ProxyTestResult, String> {
    let started = Instant::now();
    let probe_url = match proxy_test_url(&target_url) {
        Ok(url) => url,
        Err(error) => {
            return Ok(ProxyTestResult {
                success: false,
                status: None,
                elapsed_ms: elapsed_millis(started),
                message: error.to_string(),
                route: None,
                resolver: None,
                fallback_attempts: 0,
            });
        }
    };
    let resolver_manager = manager.inner().clone();
    let resolver_target = probe_url.as_str().to_string();
    let test_client = match tokio::task::spawn_blocking(move || {
        resolver_manager.test_client(settings, &resolver_target)
    })
    .await
    {
        Ok(Ok(client)) => client,
        Ok(Err(error)) => {
            return Ok(ProxyTestResult {
                success: false,
                status: None,
                elapsed_ms: elapsed_millis(started),
                message: error.to_string(),
                route: None,
                resolver: None,
                fallback_attempts: 0,
            });
        }
        Err(error) => {
            return Ok(ProxyTestResult {
                success: false,
                status: None,
                elapsed_ms: elapsed_millis(started),
                message: format!("Proxy resolver task failed: {error}"),
                route: None,
                resolver: None,
                fallback_attempts: 0,
            });
        }
    };

    let request = request_builder_client()
        .and_then(|client| client.get(probe_url).build().map_err(Into::into))
        .map_err(|error| error.to_string())?;
    let result = match test_client.plan.execute(request).await {
        Ok(execution) => {
            let response = execution.response;
            let status = response.status();
            ProxyTestResult {
                success: status.is_success(),
                status: Some(status.as_u16()),
                elapsed_ms: elapsed_millis(started),
                message: format!("HTTP {}", status.as_u16()),
                route: Some(execution.route),
                resolver: Some(test_client.resolver),
                fallback_attempts: execution.fallback_attempts,
            }
        }
        Err(error) => ProxyTestResult {
            success: false,
            status: None,
            elapsed_ms: elapsed_millis(started),
            message: format!("Proxy test request failed: {}", error),
            route: Some(error.route),
            resolver: Some(test_client.resolver),
            fallback_attempts: error.fallback_attempts,
        },
    };
    Ok(result)
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}
