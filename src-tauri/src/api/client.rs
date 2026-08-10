use anyhow::Result;
use reqwest::Client;

use crate::service::proxy::ProxyManager;

/// Build a JumpServer API client from the latest proxy settings.
pub(crate) fn api_client_for_origin(manager: &ProxyManager, origin: &str) -> Result<Client> {
    manager.api_client_for_origin(origin)
}

/// Build an OAuth token client from the latest proxy settings while preserving
/// the existing no-redirect policy.
pub(crate) fn oauth_client_for_origin(manager: &ProxyManager, origin: &str) -> Result<Client> {
    manager.oauth_client_for_origin(origin)
}
