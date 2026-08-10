use crate::service::oauth::revoke_and_clear_tokens;
use crate::service::proxy::ProxyManager;
use log::warn;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn logout(
    _app: AppHandle,
    proxy_manager: State<'_, ProxyManager>,
    _name: String,
    site: String,
) -> Result<(), String> {
    if let Err(e) = revoke_and_clear_tokens(&site, &proxy_manager).await {
        warn!("revoke token failed: {}", e);
    }

    Ok(())
}
