use serde::Serialize;
use std::time::Instant;
use tauri::State;

use crate::service::proxy::{proxy_test_url, ProxyManager, ProxySettings, ProxySettingsInput};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,
    elapsed_ms: u64,
    message: String,
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
            });
        }
    };
    let client = match manager.test_client(settings, probe_url.as_str()) {
        Ok(client) => client,
        Err(error) => {
            return Ok(ProxyTestResult {
                success: false,
                status: None,
                elapsed_ms: elapsed_millis(started),
                message: error.to_string(),
            });
        }
    };

    let result = match client.get(probe_url).send().await {
        Ok(response) => {
            let status = response.status();
            ProxyTestResult {
                success: status.as_u16() != 407,
                status: Some(status.as_u16()),
                elapsed_ms: elapsed_millis(started),
                message: format!("HTTP {}", status.as_u16()),
            }
        }
        Err(error) => ProxyTestResult {
            success: false,
            status: error.status().map(|status| status.as_u16()),
            elapsed_ms: elapsed_millis(started),
            message: format!("Proxy test request failed: {}", error),
        },
    };
    Ok(result)
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}
