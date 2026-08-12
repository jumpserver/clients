use anyhow::{anyhow, Context, Result};
use keyring_core::{Entry, Error as KeyringError};
use reqwest::{redirect, Client, NoProxy, Proxy, Request, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tempfile::NamedTempFile;
use url::{Host, Url};

use crate::service::config::ConfigService;
use crate::service::system_proxy::{self, ProxyDirective, RoutePlan};

const PROXY_PASSWORD_SERVICE: &str = "com.jumpserver.client.proxy";
const LEGACY_PROXY_PASSWORD_ACCOUNT: &str = "manual-proxy";
const CREDENTIAL_ID_PREFIX: &str = "manual-proxy-v1-";
const PROXY_CONFIG_FILE: &str = "proxy.json";
const PROXY_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_BYPASS: [&str; 3] = ["localhost", "127.0.0.0/8", "::1"];
const MAX_HOST_LEN: usize = 255;
const MAX_USERNAME_LEN: usize = 256;
const MAX_PASSWORD_LEN: usize = 1024;
const MAX_BYPASS_ENTRIES: usize = 128;
const MAX_EFFECTIVE_BYPASS_ENTRIES: usize = MAX_BYPASS_ENTRIES + DEFAULT_BYPASS.len();
const MAX_BYPASS_ENTRY_LEN: usize = 255;
const MAX_TARGET_LEN: usize = 2048;
const MAX_CREDENTIAL_ID_LEN: usize = 128;
static CREDENTIAL_REVISION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    #[serde(alias = "environment")]
    #[default]
    Direct,
    System,
    Pac,
    Manual,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyPreferredMode {
    System,
    Pac,
    #[default]
    Manual,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    #[default]
    Http,
    Socks5,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProxySettings {
    pub mode: ProxyMode,
    pub preferred_mode: ProxyPreferredMode,
    pub pac_url: String,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub bypass: Vec<String>,
    pub has_password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Direct,
            preferred_mode: ProxyPreferredMode::Manual,
            pac_url: String::new(),
            proxy_type: ProxyType::Http,
            host: String::new(),
            port: None,
            username: String::new(),
            bypass: default_bypass(),
            has_password: false,
            warning: None,
        }
    }
}

/// Command input. It intentionally does not implement `Serialize` or `Debug`,
/// so a password cannot accidentally be returned or included in debug logs.
#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProxySettingsInput {
    pub mode: ProxyMode,
    pub preferred_mode: ProxyPreferredMode,
    pub pac_url: String,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub bypass: Vec<String>,
    pub password: Option<String>,
    pub clear_password: bool,
}

impl Default for ProxySettingsInput {
    fn default() -> Self {
        let settings = ProxySettings::default();
        Self {
            mode: settings.mode,
            preferred_mode: settings.preferred_mode,
            pac_url: settings.pac_url,
            proxy_type: settings.proxy_type,
            host: settings.host,
            port: settings.port,
            username: settings.username,
            bypass: settings.bypass,
            password: None,
            clear_password: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct StoredProxySettings {
    mode: ProxyMode,
    preferred_mode: ProxyPreferredMode,
    pac_url: String,
    proxy_type: ProxyType,
    host: String,
    port: Option<u16>,
    username: String,
    bypass: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential_id: Option<String>,
}

impl Default for StoredProxySettings {
    fn default() -> Self {
        let settings = ProxySettings::default();
        Self {
            mode: settings.mode,
            preferred_mode: settings.preferred_mode,
            pac_url: settings.pac_url,
            proxy_type: settings.proxy_type,
            host: settings.host,
            port: settings.port,
            username: settings.username,
            bypass: settings.bypass,
            credential_id: None,
        }
    }
}

impl StoredProxySettings {
    fn public(&self, has_password: bool, warning: Option<String>) -> ProxySettings {
        ProxySettings {
            mode: self.mode,
            preferred_mode: self.preferred_mode,
            pac_url: self.pac_url.clone(),
            proxy_type: self.proxy_type,
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            bypass: self.bypass.clone(),
            has_password,
            warning,
        }
    }
}

#[derive(Clone)]
pub struct ProxyManager {
    config_path: PathBuf,
    state: Arc<RwLock<ProxyRuntimeState>>,
    update_lock: Arc<tokio::sync::Mutex<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyRouteKind {
    Direct,
    Proxy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyResolverKind {
    Direct,
    System,
    Pac,
    Manual,
}

pub struct ProxyTestClient {
    pub plan: ProxyExecutionPlan,
    pub resolver: ProxyResolverKind,
}

pub(crate) struct ProxyExecutionPlan {
    attempts: Vec<ProxyRouteAttempt>,
    timeout: Option<Duration>,
}

struct ProxyRouteAttempt {
    client: Result<Client>,
    route: ProxyRouteKind,
}

pub(crate) struct ProxyExecutionResult {
    pub(crate) response: Response,
    pub(crate) route: ProxyRouteKind,
    pub(crate) fallback_attempts: usize,
}

#[derive(Debug)]
pub(crate) struct ProxyExecutionFailure {
    error: anyhow::Error,
    pub(crate) route: ProxyRouteKind,
    pub(crate) fallback_attempts: usize,
}

impl std::fmt::Display for ProxyExecutionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ProxyExecutionFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl ProxyExecutionPlan {
    pub(crate) async fn execute(
        self,
        request: Request,
    ) -> std::result::Result<ProxyExecutionResult, ProxyExecutionFailure> {
        let deadline = self
            .timeout
            .map(|timeout| std::time::Instant::now() + timeout);
        let total = self.attempts.len();
        let mut request = Some(request);

        for (index, attempt) in self.attempts.into_iter().enumerate() {
            let client = attempt.client.map_err(|error| ProxyExecutionFailure {
                error,
                route: attempt.route,
                fallback_attempts: index,
            })?;
            let current = request
                .take()
                .expect("proxy execution always carries a request");
            let retry = (index + 1 < total).then(|| current.try_clone()).flatten();

            let result = if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(ProxyExecutionFailure {
                        error: anyhow!("Proxy request timed out across fallback routes"),
                        route: attempt.route,
                        fallback_attempts: index,
                    });
                }
                match tokio::time::timeout(remaining, client.execute(current)).await {
                    Ok(result) => result,
                    Err(_) => {
                        return Err(ProxyExecutionFailure {
                            error: anyhow!("Proxy request timed out across fallback routes"),
                            route: attempt.route,
                            fallback_attempts: index,
                        });
                    }
                }
            } else {
                client.execute(current).await
            };

            match result {
                Ok(response) => {
                    return Ok(ProxyExecutionResult {
                        response,
                        route: attempt.route,
                        fallback_attempts: index,
                    });
                }
                Err(error) => {
                    if is_retryable_connect_failure(&error) {
                        if let Some(retry) = retry {
                            log::warn!(
                                "Proxy route {} of {} failed during connection setup; trying the next PAC route",
                                index + 1,
                                total
                            );
                            request = Some(retry);
                            continue;
                        }
                        if index + 1 < total {
                            log::warn!(
                                "Proxy route failed during connection setup, but the request body cannot be cloned; fallback was not attempted"
                            );
                        }
                    }
                    return Err(ProxyExecutionFailure {
                        error: error.without_url().into(),
                        route: attempt.route,
                        fallback_attempts: index,
                    });
                }
            }
        }

        Err(ProxyExecutionFailure {
            error: anyhow!("Proxy resolver returned no executable routes"),
            route: ProxyRouteKind::Direct,
            fallback_attempts: 0,
        })
    }
}

fn is_retryable_connect_failure(error: &reqwest::Error) -> bool {
    if !error.is_connect() {
        return false;
    }
    if error.is_timeout() {
        return true;
    }

    let mut source = std::error::Error::source(error);
    while let Some(current) = source {
        if current.downcast_ref::<std::io::Error>().is_some() {
            return true;
        }
        source = current.source();
    }
    false
}

#[derive(Clone)]
struct ProxyRuntimeState {
    settings: StoredProxySettings,
    password: PasswordState,
    load_warning: Option<String>,
}

#[derive(Clone)]
enum PasswordState {
    Available {
        password: Option<String>,
        account: Option<String>,
    },
    Deferred {
        account: Option<String>,
    },
    Unavailable {
        account: Option<String>,
        error: String,
    },
}

impl PasswordState {
    fn password(&self) -> Result<Option<String>> {
        match self {
            Self::Available { password, .. } => Ok(password.clone()),
            Self::Deferred { account } => account
                .as_deref()
                .map(load_password)
                .transpose()
                .map(Option::flatten),
            Self::Unavailable { error, .. } => {
                Err(anyhow!("Proxy password keyring is unavailable: {}", error))
            }
        }
    }

    async fn password_async(&self) -> Result<Option<String>> {
        let Self::Deferred {
            account: Some(account),
        } = self
        else {
            return self.password();
        };
        let account = account.clone();
        tokio::task::spawn_blocking(move || load_password(&account))
            .await
            .context("proxy keyring task failed")?
    }

    fn account(&self) -> Option<&str> {
        match self {
            Self::Available { account, .. }
            | Self::Deferred { account }
            | Self::Unavailable { account, .. } => account.as_deref(),
        }
    }

    fn has_password(&self) -> bool {
        match self {
            Self::Available { password, .. } => password.is_some(),
            Self::Deferred { account } => account.is_some(),
            Self::Unavailable { .. } => false,
        }
    }

    fn deferred(&self) -> Self {
        Self::Deferred {
            account: self.account().map(str::to_string),
        }
    }
}

impl ProxyManager {
    pub fn from_app(app: &AppHandle) -> Result<Self> {
        let config_path = ConfigService::get_user_config_dir(app)
            .map_err(|error| anyhow!(error))?
            .join(PROXY_CONFIG_FILE);
        let (settings, load_warning) = recover_loaded_settings(load_settings(&config_path));
        if let Some(warning) = load_warning.as_deref() {
            log::warn!("{}", warning);
        }
        let password = if settings.mode == ProxyMode::Pac {
            PasswordState::Deferred {
                account: credential_account(&settings),
            }
        } else {
            load_password_state(credential_account(&settings))
        };

        Ok(Self {
            config_path,
            state: Arc::new(RwLock::new(ProxyRuntimeState {
                settings,
                password,
                load_warning,
            })),
            update_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(config_path: PathBuf, input: ProxySettingsInput) -> Result<Self> {
        let (settings, password) = normalize_input(input, None)?;
        validate_authentication(&settings, password.as_deref())?;
        Ok(Self {
            config_path,
            state: Arc::new(RwLock::new(ProxyRuntimeState {
                settings,
                password: PasswordState::Available {
                    password,
                    account: None,
                },
                load_warning: None,
            })),
            update_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    pub fn settings(&self) -> ProxySettings {
        let state = self.state.read().expect("proxy state lock poisoned");
        state
            .settings
            .public(state.password.has_password(), state.load_warning.clone())
    }

    pub async fn update(&self, input: ProxySettingsInput) -> Result<ProxySettings> {
        let _guard = self.update_lock.lock().await;
        let current = self
            .state
            .read()
            .expect("proxy state lock poisoned")
            .clone();

        if input.mode != ProxyMode::Manual {
            let (mut next_settings, _) = normalize_input(input, None)?;
            next_settings.credential_id = current.settings.credential_id.clone();
            if current.password.account() == Some(LEGACY_PROXY_PASSWORD_ACCOUNT) {
                // Without a credentialId, the legacy username is the only durable
                // signal that the fixed legacy account must be loaded next startup.
                next_settings.username = current.settings.username.clone();
            }
            save_settings(&self.config_path, &next_settings)?;

            let password = if next_settings.mode == ProxyMode::Pac {
                current.password.deferred()
            } else {
                current.password
            };
            let next = ProxyRuntimeState {
                settings: next_settings.clone(),
                password,
                load_warning: None,
            };
            let result = next
                .settings
                .public(next.password.has_password(), next.load_warning.clone());
            *self.state.write().expect("proxy state lock poisoned") = next;
            return Ok(result);
        }

        let supplied_password = input
            .password
            .as_ref()
            .is_some_and(|password| !password.is_empty());
        let current_password = if input.clear_password || supplied_password {
            None
        } else {
            current.password.password_async().await?
        };
        let (mut next_settings, next_password) =
            normalize_input(input, current_password.as_deref())?;
        validate_authentication(&next_settings, next_password.as_deref())?;

        let next_password_state = if let Some(password) = next_password.clone() {
            let credential_id = new_credential_id();
            write_password_async(credential_id.clone(), password.clone()).await?;
            next_settings.credential_id = Some(credential_id.clone());
            PasswordState::Available {
                password: Some(password),
                account: Some(credential_id),
            }
        } else {
            next_settings.credential_id = None;
            PasswordState::Available {
                password: None,
                account: None,
            }
        };

        if let Err(error) = save_settings(&self.config_path, &next_settings) {
            if let Some(account) = next_password_state.account() {
                if let Err(cleanup_error) = delete_password_async(account.to_string()).await {
                    log::error!(
                        "Failed to remove unreferenced proxy credential after config write failure: {}",
                        cleanup_error
                    );
                }
            }
            return Err(error);
        }

        let old_account = current.password.account().map(str::to_string);
        let new_account = next_password_state.account().map(str::to_string);
        let next = ProxyRuntimeState {
            settings: next_settings.clone(),
            password: next_password_state,
            load_warning: None,
        };
        let result = next
            .settings
            .public(next.password.has_password(), next.load_warning.clone());
        *self.state.write().expect("proxy state lock poisoned") = next;

        if old_account != new_account {
            if let Some(old_account) = old_account {
                if let Err(error) = delete_password_async(old_account).await {
                    log::warn!("Failed to remove superseded proxy credential: {}", error);
                }
            }
        }

        Ok(result)
    }

    pub fn test_client(&self, input: ProxySettingsInput, target: &str) -> Result<ProxyTestClient> {
        let needs_saved_password = input.mode == ProxyMode::Manual
            && !input.username.trim().is_empty()
            && !input.clear_password
            && input
                .password
                .as_ref()
                .is_none_or(|password| password.is_empty());
        let current_password = if needs_saved_password {
            self.state
                .read()
                .expect("proxy state lock poisoned")
                .password
                .password()?
        } else {
            None
        };
        let (settings, password) = normalize_input(input, current_password.as_deref())?;
        validate_authentication(&settings, password.as_deref())?;
        if settings.mode == ProxyMode::Manual && should_bypass_proxy(target, &settings.bypass) {
            return Err(anyhow!(
                "Target matches proxy bypass rules; the manual proxy was not used"
            ));
        }
        let route_plan = resolve_route_plan(&settings, target)?;
        let resolver = resolver_metadata(&settings, route_plan.as_ref())?;
        let plan = build_execution_plan_with_route_plan(
            &settings,
            password.as_deref(),
            target,
            true,
            Some(Duration::from_secs(10)),
            route_plan.as_ref(),
        )?;
        Ok(ProxyTestClient { plan, resolver })
    }

    pub(crate) fn execution_plan_for_target(
        &self,
        target: &str,
        disable_redirects: bool,
        timeout: Option<Duration>,
    ) -> Result<ProxyExecutionPlan> {
        let state = self
            .state
            .read()
            .expect("proxy state lock poisoned")
            .clone();
        let settings = state.settings;
        let bypasses_manual_proxy =
            settings.mode == ProxyMode::Manual && should_bypass_proxy(target, &settings.bypass);
        let password = request_password(&settings, &state.password, target)?;
        if !bypasses_manual_proxy {
            validate_authentication(&settings, password.as_deref())?;
        }
        let route_plan = resolve_route_plan(&settings, target)?;
        build_execution_plan_with_route_plan(
            &settings,
            password.as_deref(),
            target,
            disable_redirects,
            timeout,
            route_plan.as_ref(),
        )
    }
}

fn build_execution_plan_with_route_plan(
    settings: &StoredProxySettings,
    password: Option<&str>,
    target: &str,
    disable_redirects: bool,
    timeout: Option<Duration>,
    route_plan: Option<&RoutePlan>,
) -> Result<ProxyExecutionPlan> {
    let attempts = if matches!(settings.mode, ProxyMode::System | ProxyMode::Pac) {
        let resolved_plan;
        let plan = match route_plan {
            Some(plan) => plan,
            None => {
                resolved_plan = resolve_route_plan(settings, target)?
                    .ok_or_else(|| anyhow!("Proxy route was not resolved"))?;
                &resolved_plan
            }
        };
        plan.routes()
            .iter()
            .cloned()
            .map(|route| {
                let route_kind = match route {
                    ProxyDirective::Direct => ProxyRouteKind::Direct,
                    ProxyDirective::Proxy { .. } | ProxyDirective::Unsupported { .. } => {
                        ProxyRouteKind::Proxy
                    }
                };
                let client = RoutePlan::new(plan.source(), vec![route]).and_then(|single_plan| {
                    build_client_with_route_plan(
                        settings,
                        password,
                        target,
                        disable_redirects,
                        timeout,
                        Some(&single_plan),
                    )
                });
                ProxyRouteAttempt {
                    client,
                    route: route_kind,
                }
            })
            .collect()
    } else {
        let route = if settings.mode == ProxyMode::Manual
            && !should_bypass_proxy(target, &settings.bypass)
        {
            ProxyRouteKind::Proxy
        } else {
            ProxyRouteKind::Direct
        };
        vec![ProxyRouteAttempt {
            client: build_client(settings, password, target, disable_redirects, timeout),
            route,
        }]
    };

    Ok(ProxyExecutionPlan { attempts, timeout })
}

fn build_client(
    settings: &StoredProxySettings,
    password: Option<&str>,
    target: &str,
    disable_redirects: bool,
    timeout: Option<Duration>,
) -> Result<Client> {
    build_client_with_route_plan(settings, password, target, disable_redirects, timeout, None)
}

fn build_client_with_route_plan(
    settings: &StoredProxySettings,
    password: Option<&str>,
    target: &str,
    disable_redirects: bool,
    timeout: Option<Duration>,
    route_plan: Option<&RoutePlan>,
) -> Result<Client> {
    let mut builder = Client::builder();
    if !matches!(settings.mode, ProxyMode::System | ProxyMode::Pac) {
        // Compatibility hold for existing modes. Native and custom PAC modes
        // use strict platform trust and never inherit this legacy policy.
        builder = builder.danger_accept_invalid_certs(true);
    }

    if disable_redirects || matches!(settings.mode, ProxyMode::System | ProxyMode::Pac) {
        builder = builder.redirect(redirect::Policy::none());
    }
    if let Some(connect_timeout) = client_connect_timeout(settings.mode, timeout) {
        builder = builder.connect_timeout(connect_timeout);
    }
    if let Some(timeout) = timeout {
        builder = builder.timeout(timeout);
    }

    builder = match settings.mode {
        ProxyMode::Direct => builder.no_proxy(),
        ProxyMode::System | ProxyMode::Pac => {
            let resolved_plan;
            let plan = match route_plan {
                Some(plan) => plan,
                None => {
                    resolved_plan = resolve_route_plan(settings, target)?
                        .ok_or_else(|| anyhow!("Proxy route was not resolved"))?;
                    &resolved_plan
                }
            };
            let selected = plan.primary()?;
            match selected.route {
                ProxyDirective::Direct => builder.no_proxy(),
                route @ ProxyDirective::Proxy { .. } => {
                    let proxy_url = route.proxy_url()?.expect("proxy directive has a URL");
                    let proxy = Proxy::all(proxy_url).context("invalid resolved proxy")?;
                    builder.no_proxy().proxy(proxy)
                }
                ProxyDirective::Unsupported { reason } => {
                    return Err(anyhow!(
                        "Resolved proxy route is unsupported and was not downgraded: {}",
                        reason
                    ));
                }
            }
        }
        ProxyMode::Manual => {
            if should_bypass_proxy(target, &settings.bypass) {
                builder.no_proxy()
            } else {
                let proxy = manual_proxy(settings, password)?;
                // Disable automatic proxies before adding the single manual proxy.
                builder.no_proxy().proxy(proxy)
            }
        }
    };

    builder.build().context("build HTTP client failed")
}

fn resolver_metadata(
    settings: &StoredProxySettings,
    route_plan: Option<&RoutePlan>,
) -> Result<ProxyResolverKind> {
    match settings.mode {
        ProxyMode::Direct => Ok(ProxyResolverKind::Direct),
        ProxyMode::Manual => Ok(ProxyResolverKind::Manual),
        ProxyMode::System | ProxyMode::Pac => {
            let plan = route_plan.ok_or_else(|| anyhow!("Proxy route was not resolved"))?;
            let resolver = match settings.mode {
                ProxyMode::Pac => ProxyResolverKind::Pac,
                ProxyMode::System => match plan.source() {
                    crate::service::system_proxy::RouteSource::System => ProxyResolverKind::System,
                    crate::service::system_proxy::RouteSource::Pac => ProxyResolverKind::Pac,
                },
                _ => unreachable!("only resolved modes reach this branch"),
            };
            Ok(resolver)
        }
    }
}

fn client_connect_timeout(mode: ProxyMode, request_timeout: Option<Duration>) -> Option<Duration> {
    (matches!(mode, ProxyMode::System | ProxyMode::Pac) || request_timeout.is_some())
        .then_some(PROXY_CONNECT_TIMEOUT)
}

fn resolve_route_plan(settings: &StoredProxySettings, target: &str) -> Result<Option<RoutePlan>> {
    match settings.mode {
        ProxyMode::System => system_proxy::resolve(target).map(Some),
        ProxyMode::Pac => system_proxy::resolve_pac(&settings.pac_url, target).map(Some),
        ProxyMode::Direct | ProxyMode::Manual => Ok(None),
    }
}

fn request_password(
    settings: &StoredProxySettings,
    password_state: &PasswordState,
    target: &str,
) -> Result<Option<String>> {
    if settings.mode == ProxyMode::Manual
        && !settings.username.is_empty()
        && !should_bypass_proxy(target, &settings.bypass)
    {
        password_state.password()
    } else {
        Ok(None)
    }
}

fn manual_proxy(settings: &StoredProxySettings, password: Option<&str>) -> Result<Proxy> {
    let proxy_url = manual_proxy_url(settings)?;
    let mut proxy = Proxy::all(proxy_url).context("invalid manual proxy")?;
    if let (username, Some(password)) = (settings.username.trim(), password) {
        if !username.is_empty() {
            proxy = proxy.basic_auth(username, password);
        }
    }

    let bypass = effective_bypass(&settings.bypass).join(",");
    Ok(proxy.no_proxy(NoProxy::from_string(&bypass)))
}

fn manual_proxy_url(settings: &StoredProxySettings) -> Result<String> {
    validate_settings(settings)?;
    let scheme = match settings.proxy_type {
        ProxyType::Http => "http",
        // Resolve destination DNS through the proxy to avoid local DNS leakage.
        ProxyType::Socks5 => "socks5h",
    };
    let host = settings.host.trim();
    let host = if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]", host)
    } else {
        host.to_string()
    };

    Ok(format!(
        "{}://{}:{}",
        scheme,
        host,
        settings.port.expect("validated proxy port")
    ))
}

fn validate_authentication(settings: &StoredProxySettings, password: Option<&str>) -> Result<()> {
    if settings.mode != ProxyMode::Manual {
        return Ok(());
    }

    let has_username = !settings.username.trim().is_empty();
    let has_password = password.is_some_and(|value| !value.is_empty());
    if has_username != has_password {
        return Err(anyhow!(
            "Proxy username and password must either both be set or both be empty"
        ));
    }
    Ok(())
}

fn normalize_input(
    input: ProxySettingsInput,
    current_password: Option<&str>,
) -> Result<(StoredProxySettings, Option<String>)> {
    let pac_url = input.pac_url.trim().to_string();
    if input.mode == ProxyMode::Manual
        && input
            .password
            .as_ref()
            .is_some_and(|password| password.len() > MAX_PASSWORD_LEN)
    {
        return Err(anyhow!("Proxy password is too long"));
    }
    if pac_url.len() > system_proxy::MAX_PAC_URL_LEN {
        return Err(anyhow!("PAC URL is too long"));
    }
    validate_bypass_limits(&input.bypass, MAX_BYPASS_ENTRIES)?;
    if input.mode == ProxyMode::Manual {
        validate_bypass(&input.bypass, MAX_BYPASS_ENTRIES)?;
    }
    let preferred_mode = match input.mode {
        ProxyMode::Direct
            if input.preferred_mode == ProxyPreferredMode::Pac
                && !system_proxy::custom_pac_supported() =>
        {
            ProxyPreferredMode::System
        }
        ProxyMode::Direct => input.preferred_mode,
        ProxyMode::System => ProxyPreferredMode::System,
        ProxyMode::Pac => ProxyPreferredMode::Pac,
        ProxyMode::Manual => ProxyPreferredMode::Manual,
    };
    let settings = StoredProxySettings {
        mode: input.mode,
        preferred_mode,
        pac_url,
        proxy_type: input.proxy_type,
        host: input.host.trim().to_string(),
        port: input.port,
        username: input.username.trim().to_string(),
        bypass: effective_bypass(&input.bypass),
        credential_id: None,
    };
    validate_settings(&settings)?;

    let password = if settings.mode != ProxyMode::Manual {
        current_password.map(str::to_string)
    } else if input.clear_password {
        None
    } else {
        input
            .password
            .filter(|value| !value.is_empty())
            .or_else(|| current_password.map(str::to_string))
    };

    Ok((settings, password))
}

fn validate_settings(settings: &StoredProxySettings) -> Result<()> {
    if let Some(credential_id) = settings.credential_id.as_deref() {
        validate_credential_id(credential_id)?;
    }
    let host = settings.host.trim();
    if host.len() > MAX_HOST_LEN {
        return Err(anyhow!("Proxy host is too long"));
    }
    if settings.username.len() > MAX_USERNAME_LEN {
        return Err(anyhow!("Proxy username is too long"));
    }
    if settings.pac_url.len() > system_proxy::MAX_PAC_URL_LEN {
        return Err(anyhow!("PAC URL is too long"));
    }
    if settings.mode == ProxyMode::Pac {
        system_proxy::validate_pac_url(&settings.pac_url)?;
        system_proxy::ensure_custom_pac_supported()?;
    }
    if settings.mode != ProxyMode::Manual {
        return Ok(());
    }
    validate_bypass(&settings.bypass, MAX_EFFECTIVE_BYPASS_ENTRIES)?;

    if host.is_empty() {
        return Err(anyhow!("Proxy host is required in manual mode"));
    }
    if host.contains(char::is_whitespace)
        || host.contains("://")
        || host.contains('/')
        || host.contains('?')
        || host.contains('#')
        || host.contains('@')
    {
        return Err(anyhow!("Proxy host must be a host name or IP address"));
    }
    if settings.port.is_none() || settings.port == Some(0) {
        return Err(anyhow!("Proxy port must be between 1 and 65535"));
    }

    let candidate = manual_proxy_url_without_validation(settings);
    Url::parse(&candidate).context("Proxy host or port is invalid")?;
    Ok(())
}

fn validate_credential_id(credential_id: &str) -> Result<()> {
    if credential_id.is_empty()
        || credential_id.len() > MAX_CREDENTIAL_ID_LEN
        || !credential_id.starts_with(CREDENTIAL_ID_PREFIX)
        || !credential_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(anyhow!("Invalid proxy credential reference"));
    }
    Ok(())
}

fn manual_proxy_url_without_validation(settings: &StoredProxySettings) -> String {
    let scheme = match settings.proxy_type {
        ProxyType::Http => "http",
        ProxyType::Socks5 => "socks5h",
    };
    let host = settings.host.trim();
    let host = if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]", host)
    } else {
        host.to_string()
    };
    format!("{}://{}:{}", scheme, host, settings.port.unwrap_or(0))
}

fn validate_bypass(entries: &[String], max_entries: usize) -> Result<()> {
    validate_bypass_limits(entries, max_entries)?;
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty()
            || entry.contains(char::is_whitespace)
            || entry.contains(',')
            || entry.contains("://")
            || entry.contains('@')
            || entry.contains('?')
            || entry.contains('#')
        {
            return Err(anyhow!("Invalid proxy bypass entry"));
        }
        if entry == "*" {
            continue;
        }
        if let Some((address, prefix)) = entry.split_once('/') {
            let ip: IpAddr = address
                .parse()
                .map_err(|_| anyhow!("Invalid CIDR proxy bypass entry"))?;
            let prefix: u8 = prefix
                .parse()
                .map_err(|_| anyhow!("Invalid CIDR proxy bypass entry"))?;
            let max = if ip.is_ipv4() { 32 } else { 128 };
            if prefix > max {
                return Err(anyhow!("Invalid CIDR proxy bypass entry"));
            }
            continue;
        }
        if entry.parse::<IpAddr>().is_ok() {
            continue;
        }

        let host = entry.strip_prefix('.').unwrap_or(entry);
        let valid_host = match Host::parse(host) {
            Ok(Host::Domain(domain)) => valid_domain_name(&domain),
            Ok(Host::Ipv4(_) | Host::Ipv6(_)) => true,
            Err(_) => false,
        };
        if host.is_empty() || host.starts_with('.') || !valid_host {
            return Err(anyhow!("Invalid host proxy bypass entry"));
        }
    }
    Ok(())
}

fn validate_bypass_limits(entries: &[String], max_entries: usize) -> Result<()> {
    if entries.len() > max_entries {
        return Err(anyhow!("Too many proxy bypass entries"));
    }
    if entries
        .iter()
        .any(|entry| entry.trim().len() > MAX_BYPASS_ENTRY_LEN)
    {
        return Err(anyhow!("Proxy bypass entry is too long"));
    }
    Ok(())
}

fn valid_domain_name(domain: &str) -> bool {
    let domain = domain.strip_suffix('.').unwrap_or(domain);
    !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

pub(crate) fn proxy_test_url(target: &str) -> Result<Url> {
    if target.len() > MAX_TARGET_LEN {
        return Err(anyhow!("Target URL is too long"));
    }
    let url = Url::parse(target).context("Target URL is invalid")?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(anyhow!("Target URL must use http or https"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(anyhow!("Target URL must not contain user information"));
    }
    if url.host_str().is_none() {
        return Err(anyhow!("Target URL must include a host"));
    }
    if !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
        return Err(anyhow!(
            "Target URL must be an origin without path, query, or fragment"
        ));
    }

    let mut probe = url;
    probe.set_path(crate::api::endpoint::oauth::WELL_KNOWN);
    Ok(probe)
}

pub(crate) fn should_bypass_proxy(target: &str, configured: &[String]) -> bool {
    let Ok(url) = Url::parse(target) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host
        .strip_prefix('[')
        .and_then(|host| host.strip_suffix(']'))
        .unwrap_or(host);
    let host = host.trim_end_matches('.');
    if host.is_empty() {
        return false;
    }

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if host
        .parse::<IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
    {
        return true;
    }

    effective_bypass(configured)
        .iter()
        .any(|entry| bypass_entry_matches(entry, host))
}

fn bypass_entry_matches(entry: &str, host: &str) -> bool {
    if entry == "*" {
        return true;
    }
    if let Some((network, prefix)) = entry.split_once('/') {
        let Ok(network) = network.parse::<IpAddr>() else {
            return false;
        };
        let Ok(prefix) = prefix.parse::<u8>() else {
            return false;
        };
        let Ok(address) = host.parse::<IpAddr>() else {
            return false;
        };
        return ip_in_network(address, network, prefix);
    }
    if let (Ok(entry_ip), Ok(host_ip)) = (entry.parse::<IpAddr>(), host.parse::<IpAddr>()) {
        return entry_ip == host_ip;
    }

    let domain = entry.trim_start_matches('.').trim_end_matches('.');
    if domain.is_empty() {
        return false;
    }
    host.eq_ignore_ascii_case(domain)
        || host
            .to_ascii_lowercase()
            .ends_with(&format!(".{}", domain.to_ascii_lowercase()))
}

fn ip_in_network(address: IpAddr, network: IpAddr, prefix: u8) -> bool {
    match (address, network) {
        (IpAddr::V4(address), IpAddr::V4(network)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            u32::from(address) & mask == u32::from(network) & mask
        }
        (IpAddr::V6(address), IpAddr::V6(network)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            u128::from(address) & mask == u128::from(network) & mask
        }
        _ => false,
    }
}

fn default_bypass() -> Vec<String> {
    DEFAULT_BYPASS
        .iter()
        .map(|entry| entry.to_string())
        .collect()
}

fn effective_bypass(configured: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    DEFAULT_BYPASS
        .iter()
        .map(|entry| (*entry).to_string())
        .chain(configured.iter().map(|entry| entry.trim().to_string()))
        .filter(|entry| !entry.is_empty())
        .filter(|entry| seen.insert(entry.to_ascii_lowercase()))
        .collect()
}

fn load_settings(path: &Path) -> Result<StoredProxySettings> {
    match fs::read(path) {
        Ok(contents) => {
            let mut settings: StoredProxySettings =
                serde_json::from_slice(&contents).context("parse proxy.json failed")?;
            settings.host = settings.host.trim().to_string();
            settings.username = settings.username.trim().to_string();
            settings.bypass = effective_bypass(&settings.bypass);
            validate_settings(&settings)?;
            Ok(settings)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(StoredProxySettings::default())
        }
        Err(error) => Err(error).context("read proxy.json failed"),
    }
}

fn recover_loaded_settings(
    result: Result<StoredProxySettings>,
) -> (StoredProxySettings, Option<String>) {
    match result {
        Ok(settings) => (settings, None),
        Err(error) => {
            let settings = StoredProxySettings {
                mode: ProxyMode::Direct,
                ..StoredProxySettings::default()
            };
            (
                settings,
                Some(format!(
                    "Proxy settings could not be loaded; direct mode is active: {}",
                    error
                )),
            )
        }
    }
}

fn save_settings(path: &Path, settings: &StoredProxySettings) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("proxy config path has no parent"))?;
    fs::create_dir_all(parent).context("create proxy config directory failed")?;
    let payload = serde_json::to_vec_pretty(settings).context("serialize proxy settings failed")?;
    let mut temporary =
        NamedTempFile::new_in(parent).context("create proxy config temp file failed")?;
    temporary
        .write_all(&payload)
        .context("write proxy config temp file failed")?;
    temporary
        .as_file_mut()
        .sync_all()
        .context("sync proxy config temp file failed")?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .context("atomically replace proxy.json failed")?;
    Ok(())
}

fn credential_account(settings: &StoredProxySettings) -> Option<String> {
    settings.credential_id.clone().or_else(|| {
        (!settings.username.is_empty()).then(|| LEGACY_PROXY_PASSWORD_ACCOUNT.to_string())
    })
}

fn load_password_state(account: Option<String>) -> PasswordState {
    let Some(account) = account else {
        return PasswordState::Available {
            password: None,
            account: None,
        };
    };

    match load_password(&account) {
        Ok(password) => PasswordState::Available {
            password,
            account: Some(account),
        },
        Err(error) => {
            log::warn!("Failed to load proxy password from keyring: {}", error);
            PasswordState::Unavailable {
                account: Some(account),
                error: error.to_string(),
            }
        }
    }
}

fn credential_entry(account: &str) -> Result<Entry> {
    keyring::use_native_store(false)?;
    Ok(Entry::new(PROXY_PASSWORD_SERVICE, account)?)
}

fn load_password(account: &str) -> Result<Option<String>> {
    match credential_entry(account)?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn new_credential_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let revision = CREDENTIAL_REVISION.fetch_add(1, Ordering::Relaxed);
    format!(
        "{}{:x}-{:x}-{:x}",
        CREDENTIAL_ID_PREFIX,
        std::process::id(),
        timestamp,
        revision
    )
}

async fn write_password_async(account: String, password: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        credential_entry(&account)?.set_password(&password)?;
        Ok(())
    })
    .await
    .context("proxy keyring task failed")??;
    Ok(())
}

async fn delete_password_async(account: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        match credential_entry(&account)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(error.into()),
        }
    })
    .await
    .context("proxy keyring task failed")??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn manual_input() -> ProxySettingsInput {
        ProxySettingsInput {
            mode: ProxyMode::Manual,
            preferred_mode: ProxyPreferredMode::Manual,
            pac_url: String::new(),
            proxy_type: ProxyType::Http,
            host: "proxy.example.com".to_string(),
            port: Some(8080),
            username: String::new(),
            bypass: vec!["internal.example.com".to_string()],
            password: None,
            clear_password: false,
        }
    }

    #[test]
    fn defaults_to_direct_without_credentials() {
        let settings = ProxySettings::default();
        assert_eq!(settings.mode, ProxyMode::Direct);
        assert!(settings.pac_url.is_empty());
        assert!(!settings.has_password);
        assert!(settings.bypass.contains(&"127.0.0.0/8".to_string()));

        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(serialized.contains(r#""mode":"direct""#));
        assert!(serialized.contains(r#""pacUrl":"""#));
        assert!(!serialized.contains("environment"));

        let legacy: ProxySettings = serde_json::from_str(r#"{"mode":"environment"}"#).unwrap();
        assert_eq!(legacy.mode, ProxyMode::Direct);
    }

    #[test]
    fn public_and_persisted_serialization_never_contains_password() {
        let (mut stored, password) = normalize_input(
            ProxySettingsInput {
                password: Some("top-secret".to_string()),
                username: "alice".to_string(),
                ..manual_input()
            },
            None,
        )
        .unwrap();
        stored.credential_id = Some("manual-proxy-v1-test".to_string());
        let public = stored.public(password.is_some(), None);

        let stored_json = serde_json::to_string(&stored).unwrap();
        assert!(stored_json.contains("credentialId"));
        assert!(!stored_json.contains("top-secret"));
        assert!(!serde_json::to_string(&public)
            .unwrap()
            .contains("top-secret"));
        assert!(!serde_json::to_string(&public)
            .unwrap()
            .contains("credentialId"));
        assert!(public.has_password);
    }

    #[test]
    fn corrupt_settings_recover_to_direct_with_warning() {
        let (settings, warning) = recover_loaded_settings(Err(anyhow!("parse proxy.json failed")));
        assert_eq!(settings.mode, ProxyMode::Direct);
        assert!(warning
            .as_deref()
            .is_some_and(|warning| warning.contains("direct mode is active")));

        let (settings, warning) = recover_loaded_settings(Ok(StoredProxySettings::default()));
        assert_eq!(settings.mode, ProxyMode::Direct);
        assert!(warning.is_none());
    }

    #[test]
    fn credential_account_supports_legacy_and_versioned_records() {
        let mut settings = StoredProxySettings {
            username: "alice".to_string(),
            ..StoredProxySettings::default()
        };
        assert_eq!(
            credential_account(&settings).as_deref(),
            Some(LEGACY_PROXY_PASSWORD_ACCOUNT)
        );

        settings.credential_id = Some("manual-proxy-v1-revision".to_string());
        assert_eq!(
            credential_account(&settings).as_deref(),
            Some("manual-proxy-v1-revision")
        );
    }

    #[test]
    fn manual_proxy_uses_remote_dns_for_socks5() {
        let (stored, _) = normalize_input(
            ProxySettingsInput {
                proxy_type: ProxyType::Socks5,
                ..manual_input()
            },
            None,
        )
        .unwrap();
        assert_eq!(
            manual_proxy_url(&stored).unwrap(),
            "socks5h://proxy.example.com:8080"
        );
    }

    #[test]
    fn validation_rejects_invalid_manual_settings_and_partial_auth() {
        let invalid = ProxySettingsInput {
            host: "https://proxy.example.com".to_string(),
            ..manual_input()
        };
        assert!(normalize_input(invalid, None).is_err());

        let (stored, password) = normalize_input(
            ProxySettingsInput {
                username: "alice".to_string(),
                ..manual_input()
            },
            None,
        )
        .unwrap();
        assert!(validate_authentication(&stored, password.as_deref()).is_err());
    }

    #[test]
    fn pac_mode_serializes_and_validates_an_independent_url() {
        let (stored, password) = normalize_input(
            ProxySettingsInput {
                mode: ProxyMode::Pac,
                preferred_mode: ProxyPreferredMode::Manual,
                pac_url: " https://proxy.example.com/config.pac ".to_string(),
                host: "unused manual host".to_string(),
                username: "unused manual user".to_string(),
                password: Some("unused manual password".to_string()),
                ..ProxySettingsInput::default()
            },
            Some("existing"),
        )
        .unwrap();

        assert_eq!(stored.mode, ProxyMode::Pac);
        assert_eq!(stored.preferred_mode, ProxyPreferredMode::Pac);
        assert_eq!(stored.pac_url, "https://proxy.example.com/config.pac");
        assert_eq!(password.as_deref(), Some("existing"));

        let serialized = serde_json::to_string(&stored.public(true, None)).unwrap();
        assert!(serialized.contains(r#""mode":"pac""#));
        assert!(serialized.contains(r#""preferredMode":"pac""#));
        assert!(serialized.contains(r#""pacUrl":"https://proxy.example.com/config.pac""#));
    }

    #[test]
    fn pac_url_validation_is_strict_only_when_pac_is_active() {
        for invalid in [
            "",
            "file:///tmp/proxy.pac",
            "http://alice:secret@proxy.example.com/config.pac",
            "https://",
        ] {
            let input = ProxySettingsInput {
                mode: ProxyMode::Pac,
                pac_url: invalid.to_string(),
                ..ProxySettingsInput::default()
            };
            assert!(normalize_input(input, None).is_err(), "accepted {invalid}");
        }

        let hidden_invalid = ProxySettingsInput {
            mode: ProxyMode::Direct,
            pac_url: "file:///tmp/proxy.pac".to_string(),
            ..ProxySettingsInput::default()
        };
        assert!(normalize_input(hidden_invalid, None).is_ok());

        let hidden_oversized = ProxySettingsInput {
            mode: ProxyMode::Direct,
            pac_url: "x".repeat(system_proxy::MAX_PAC_URL_LEN + 1),
            ..ProxySettingsInput::default()
        };
        assert!(normalize_input(hidden_oversized, None).is_err());

        let padded_at_limit = ProxySettingsInput {
            mode: ProxyMode::Direct,
            pac_url: format!("  {}  ", "x".repeat(system_proxy::MAX_PAC_URL_LEN)),
            ..ProxySettingsInput::default()
        };
        assert!(normalize_input(padded_at_limit, None).is_ok());
    }

    #[test]
    fn bypass_validation_rejects_malformed_hosts_and_counts_only_user_entries() {
        for invalid in ["bad:port", "!!", "example..com", "..example.com"] {
            let input = ProxySettingsInput {
                bypass: vec![invalid.to_string()],
                ..manual_input()
            };
            assert!(normalize_input(input, None).is_err(), "accepted {invalid}");
        }

        let input = ProxySettingsInput {
            bypass: (0..MAX_BYPASS_ENTRIES)
                .map(|index| format!("host{index}.example.com"))
                .collect(),
            ..manual_input()
        };
        let (settings, _) = normalize_input(input, None).unwrap();
        assert_eq!(
            settings.bypass.len(),
            MAX_BYPASS_ENTRIES + DEFAULT_BYPASS.len()
        );

        let oversized_hidden_bypass = ProxySettingsInput {
            mode: ProxyMode::System,
            bypass: vec!["x".repeat(MAX_BYPASS_ENTRY_LEN + 1)],
            ..ProxySettingsInput::default()
        };
        assert!(normalize_input(oversized_hidden_bypass, None).is_err());
    }

    #[test]
    fn empty_password_preserves_existing_and_clear_is_explicit() {
        let (_, preserved) = normalize_input(
            ProxySettingsInput {
                username: "alice".to_string(),
                password: Some(String::new()),
                ..manual_input()
            },
            Some("existing"),
        )
        .unwrap();
        assert_eq!(preserved.as_deref(), Some("existing"));

        let (_, cleared) = normalize_input(
            ProxySettingsInput {
                username: String::new(),
                clear_password: true,
                ..manual_input()
            },
            Some("existing"),
        )
        .unwrap();
        assert_eq!(cleared, None);
    }

    #[test]
    fn non_manual_modes_ignore_credential_mutations() {
        for mode in [ProxyMode::Direct, ProxyMode::System, ProxyMode::Pac] {
            let (_, password) = normalize_input(
                ProxySettingsInput {
                    mode,
                    pac_url: (mode == ProxyMode::Pac)
                        .then(|| "https://proxy.example.com/config.pac".to_string())
                        .unwrap_or_default(),
                    password: Some("replacement".to_string()),
                    clear_password: true,
                    ..ProxySettingsInput::default()
                },
                Some("existing"),
            )
            .unwrap();
            assert_eq!(password.as_deref(), Some("existing"));
        }
    }

    #[test]
    fn direct_mode_preserves_the_preferred_proxy_source() {
        for preferred_mode in [
            ProxyPreferredMode::System,
            ProxyPreferredMode::Pac,
            ProxyPreferredMode::Manual,
        ] {
            let (settings, _) = normalize_input(
                ProxySettingsInput {
                    mode: ProxyMode::Direct,
                    preferred_mode,
                    ..ProxySettingsInput::default()
                },
                None,
            )
            .unwrap();
            assert_eq!(settings.mode, ProxyMode::Direct);
            assert_eq!(settings.preferred_mode, preferred_mode);
        }
    }

    #[test]
    fn pac_routes_report_pac_and_never_read_manual_credentials() {
        let settings = StoredProxySettings {
            mode: ProxyMode::Pac,
            preferred_mode: ProxyPreferredMode::Pac,
            pac_url: "https://proxy.example.com/config.pac".to_string(),
            username: "alice".to_string(),
            credential_id: Some("manual-proxy-v1-test".to_string()),
            ..StoredProxySettings::default()
        };
        let unavailable_password = PasswordState::Unavailable {
            account: settings.credential_id.clone(),
            error: "keyring must not be accessed".to_string(),
        };
        assert_eq!(
            request_password(
                &settings,
                &unavailable_password,
                "https://target.example.com/api?tenant=one"
            )
            .unwrap(),
            None
        );

        let plan =
            RoutePlan::new(system_proxy::RouteSource::Pac, vec![ProxyDirective::Direct]).unwrap();
        assert_eq!(
            resolver_metadata(&settings, Some(&plan)).unwrap(),
            ProxyResolverKind::Pac
        );
    }

    #[tokio::test]
    async fn switching_to_pac_does_not_read_an_unavailable_manual_credential() {
        let directory = tempfile::tempdir().unwrap();
        let current_settings = StoredProxySettings {
            mode: ProxyMode::Manual,
            preferred_mode: ProxyPreferredMode::Manual,
            host: "proxy.example.com".to_string(),
            port: Some(8080),
            username: "alice".to_string(),
            credential_id: Some("manual-proxy-v1-test".to_string()),
            ..StoredProxySettings::default()
        };
        let manager = ProxyManager {
            config_path: directory.path().join(PROXY_CONFIG_FILE),
            state: Arc::new(RwLock::new(ProxyRuntimeState {
                settings: current_settings,
                password: PasswordState::Unavailable {
                    account: Some("manual-proxy-v1-test".to_string()),
                    error: "keyring must not be accessed".to_string(),
                },
                load_warning: None,
            })),
            update_lock: Arc::new(tokio::sync::Mutex::new(())),
        };

        let updated = manager
            .update(ProxySettingsInput {
                mode: ProxyMode::Pac,
                preferred_mode: ProxyPreferredMode::Pac,
                pac_url: "https://proxy.example.com/config.pac".to_string(),
                ..ProxySettingsInput::default()
            })
            .await
            .unwrap();
        assert_eq!(updated.mode, ProxyMode::Pac);
        assert!(matches!(
            manager
                .state
                .read()
                .expect("proxy state lock poisoned")
                .password,
            PasswordState::Deferred { .. }
        ));
    }

    #[test]
    fn loopback_domain_and_cidr_bypass_are_enforced() {
        let bypass = vec![".example.com".to_string(), "10.0.0.0/8".to_string()];
        assert!(should_bypass_proxy("http://127.42.0.1:8080", &[]));
        assert!(should_bypass_proxy("http://[::1]:8080", &[]));
        assert!(should_bypass_proxy("https://api.example.com", &bypass));
        assert!(should_bypass_proxy("https://localhost./", &[]));
        assert!(should_bypass_proxy("https://api.example.com./", &bypass));
        assert!(should_bypass_proxy("https://10.2.3.4", &bypass));
        assert!(!should_bypass_proxy("https://notexample.com", &bypass));
    }

    #[test]
    fn test_target_rejects_credentials_and_non_http_schemes() {
        assert!(proxy_test_url("https://user:pass@example.com").is_err());
        assert!(proxy_test_url("file:///tmp/example").is_err());
        assert!(proxy_test_url("https://example.com/health").is_err());
        assert_eq!(
            proxy_test_url("https://example.com").unwrap().path(),
            crate::api::endpoint::oauth::WELL_KNOWN
        );
    }

    #[test]
    fn client_policy_builds_for_all_modes() {
        let direct = StoredProxySettings {
            mode: ProxyMode::Direct,
            ..StoredProxySettings::default()
        };
        assert!(build_client(&direct, None, "https://example.com", false, None).is_ok());

        let (manual, _) = normalize_input(manual_input(), None).unwrap();
        assert!(build_client(&manual, None, "https://example.com", false, None).is_ok());

        let pac = StoredProxySettings {
            mode: ProxyMode::Pac,
            preferred_mode: ProxyPreferredMode::Pac,
            pac_url: "https://proxy.example.com/config.pac".to_string(),
            ..StoredProxySettings::default()
        };
        let pac_plan =
            RoutePlan::new(system_proxy::RouteSource::Pac, vec![ProxyDirective::Direct]).unwrap();
        assert!(build_client_with_route_plan(
            &pac,
            None,
            "https://example.com/path?query=value",
            false,
            None,
            Some(&pac_plan)
        )
        .is_ok());

        let bypassed_invalid_manual = StoredProxySettings {
            mode: ProxyMode::Manual,
            host: String::new(),
            port: None,
            ..StoredProxySettings::default()
        };
        assert!(build_client(
            &bypassed_invalid_manual,
            None,
            "https://localhost./",
            false,
            None
        )
        .is_ok());
        assert!(build_client(
            &bypassed_invalid_manual,
            None,
            "https://example.com/",
            false,
            None
        )
        .is_err());
    }

    fn unused_loopback_address() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        address
    }

    fn spawn_http_server(
        listener: TcpListener,
        response: &'static [u8],
    ) -> thread::JoinHandle<String> {
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
            }
            stream.write_all(response).unwrap();
            String::from_utf8(request).unwrap()
        })
    }

    fn pac_execution_settings() -> StoredProxySettings {
        StoredProxySettings {
            mode: ProxyMode::Pac,
            preferred_mode: ProxyPreferredMode::Pac,
            pac_url: "https://proxy.example.com/config.pac".to_string(),
            ..StoredProxySettings::default()
        }
    }

    #[test]
    fn system_and_pac_always_limit_connection_setup_time() {
        assert_eq!(
            client_connect_timeout(ProxyMode::System, None),
            Some(PROXY_CONNECT_TIMEOUT)
        );
        assert_eq!(
            client_connect_timeout(ProxyMode::Pac, None),
            Some(PROXY_CONNECT_TIMEOUT)
        );
        assert_eq!(client_connect_timeout(ProxyMode::Direct, None), None);
        assert_eq!(client_connect_timeout(ProxyMode::Manual, None), None);
        assert_eq!(
            client_connect_timeout(ProxyMode::Direct, Some(Duration::from_secs(10))),
            Some(PROXY_CONNECT_TIMEOUT)
        );
    }

    #[tokio::test]
    async fn pac_unusable_route_does_not_fall_back_to_direct() {
        let target_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        target_listener.set_nonblocking(true).unwrap();
        let target = target_listener.local_addr().unwrap();
        let target_url = format!("http://{target}/probe");
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                ProxyDirective::Unsupported {
                    reason: "unsupported test route".to_string(),
                },
                ProxyDirective::Direct,
            ],
        )
        .unwrap();
        assert_eq!(
            resolver_metadata(&pac_execution_settings(), Some(&route_plan)).unwrap(),
            ProxyResolverKind::Pac
        );
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            &target_url,
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .post(&target_url)
            .body(reqwest::Body::wrap(reqwest::Body::from("single-use")))
            .build()
            .unwrap();
        assert!(request.try_clone().is_none());

        let error = match plan.execute(request).await {
            Ok(_) => panic!("unusable PAC route unexpectedly fell back to direct"),
            Err(error) => error,
        };
        assert_eq!(error.route, ProxyRouteKind::Proxy);
        assert_eq!(error.fallback_attempts, 0);
        assert_eq!(
            target_listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[tokio::test]
    async fn pac_unusable_route_without_explicit_fallback_fails_closed() {
        let target_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        target_listener.set_nonblocking(true).unwrap();
        let target_url = format!("http://{}/probe", target_listener.local_addr().unwrap());
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![ProxyDirective::Unsupported {
                reason: "unsupported test route".to_string(),
            }],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            &target_url,
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(&target_url)
            .build()
            .unwrap();

        let error = match plan.execute(request).await {
            Ok(_) => panic!("unsupported-only PAC route unexpectedly connected"),
            Err(error) => error,
        };
        assert_eq!(error.route, ProxyRouteKind::Proxy);
        assert_eq!(error.fallback_attempts, 0);
        assert_eq!(
            target_listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[tokio::test]
    async fn pac_connection_failure_falls_back_to_the_next_proxy() {
        let failed_proxy = unused_loopback_address();
        let fallback_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let fallback_proxy = fallback_listener.local_addr().unwrap();
        let server = spawn_http_server(
            fallback_listener,
            b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    failed_proxy.ip().to_string(),
                    failed_proxy.port(),
                ),
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    fallback_proxy.ip().to_string(),
                    fallback_proxy.port(),
                ),
                ProxyDirective::Direct,
            ],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            "http://upstream.invalid/api",
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get("http://upstream.invalid/api")
            .build()
            .unwrap();

        let execution = plan.execute(request).await.unwrap();
        assert_eq!(execution.response.status(), reqwest::StatusCode::NO_CONTENT);
        assert_eq!(execution.route, ProxyRouteKind::Proxy);
        assert_eq!(execution.fallback_attempts, 1);
        let request = server.join().unwrap();
        assert!(request.starts_with("GET http://upstream.invalid/api HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn pac_connection_failure_can_fall_back_to_direct() {
        let failed_proxy = unused_loopback_address();
        let target_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let target = target_listener.local_addr().unwrap();
        let server = spawn_http_server(
            target_listener,
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
        );
        let target_url = format!("http://{target}/probe");
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    failed_proxy.ip().to_string(),
                    failed_proxy.port(),
                ),
                ProxyDirective::Direct,
            ],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            &target_url,
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(&target_url)
            .build()
            .unwrap();

        let execution = plan.execute(request).await.unwrap();
        assert_eq!(execution.response.status(), reqwest::StatusCode::OK);
        assert_eq!(execution.route, ProxyRouteKind::Direct);
        assert_eq!(execution.fallback_attempts, 1);
        assert!(server
            .join()
            .unwrap()
            .starts_with("GET /probe HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn pac_http_response_is_never_replayed_on_a_fallback_route() {
        let first_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let first_proxy = first_listener.local_addr().unwrap();
        let first_server = spawn_http_server(
            first_listener,
            b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        let fallback_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        fallback_listener.set_nonblocking(true).unwrap();
        let fallback_proxy = fallback_listener.local_addr().unwrap();
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    first_proxy.ip().to_string(),
                    first_proxy.port(),
                ),
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    fallback_proxy.ip().to_string(),
                    fallback_proxy.port(),
                ),
            ],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            "http://upstream.invalid/api",
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get("http://upstream.invalid/api")
            .build()
            .unwrap();

        let execution = plan.execute(request).await.unwrap();
        assert_eq!(
            execution.response.status(),
            reqwest::StatusCode::BAD_GATEWAY
        );
        assert_eq!(execution.fallback_attempts, 0);
        first_server.join().unwrap();
        assert_eq!(
            fallback_listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[tokio::test]
    async fn pac_connect_http_response_is_never_replayed_on_a_fallback_route() {
        let first_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let first_proxy = first_listener.local_addr().unwrap();
        let first_server = spawn_http_server(
            first_listener,
            b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        let fallback_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        fallback_listener.set_nonblocking(true).unwrap();
        let fallback_proxy = fallback_listener.local_addr().unwrap();
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    first_proxy.ip().to_string(),
                    first_proxy.port(),
                ),
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    fallback_proxy.ip().to_string(),
                    fallback_proxy.port(),
                ),
            ],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            "https://target.example.com/api",
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get("https://target.example.com/api")
            .build()
            .unwrap();

        let error = match plan.execute(request).await {
            Ok(_) => panic!("CONNECT HTTP response unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.route, ProxyRouteKind::Proxy);
        assert_eq!(error.fallback_attempts, 0);
        assert!(first_server
            .join()
            .unwrap()
            .starts_with("CONNECT target.example.com:443 HTTP/1.1\r\n"));
        assert_eq!(
            fallback_listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[tokio::test]
    async fn pac_does_not_replay_a_non_cloneable_request_body() {
        let failed_proxy = unused_loopback_address();
        let fallback_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        fallback_listener.set_nonblocking(true).unwrap();
        let fallback_proxy = fallback_listener.local_addr().unwrap();
        let route_plan = RoutePlan::new(
            system_proxy::RouteSource::Pac,
            vec![
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    failed_proxy.ip().to_string(),
                    failed_proxy.port(),
                ),
                system_proxy::proxy_directive(
                    system_proxy::ProxyScheme::Http,
                    fallback_proxy.ip().to_string(),
                    fallback_proxy.port(),
                ),
            ],
        )
        .unwrap();
        let plan = build_execution_plan_with_route_plan(
            &pac_execution_settings(),
            None,
            "http://upstream.invalid/token",
            true,
            Some(Duration::from_secs(5)),
            Some(&route_plan),
        )
        .unwrap();
        let streaming_body = reqwest::Body::wrap(reqwest::Body::from("single-use"));
        let request = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .post("http://upstream.invalid/token")
            .body(streaming_body)
            .build()
            .unwrap();
        assert!(request.try_clone().is_none());

        let error = match plan.execute(request).await {
            Ok(_) => panic!("non-cloneable request unexpectedly used a fallback route"),
            Err(error) => error,
        };
        assert_eq!(error.fallback_attempts, 0);
        assert_eq!(
            fallback_listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[tokio::test]
    async fn http_proxy_receives_absolute_uri_and_basic_authentication() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (request_tx, request_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
            }
            request_tx
                .send(String::from_utf8(request).unwrap())
                .unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
        });

        let (settings, _) = normalize_input(
            ProxySettingsInput {
                host: address.ip().to_string(),
                port: Some(address.port()),
                username: "alice".to_string(),
                password: Some("secret".to_string()),
                ..manual_input()
            },
            None,
        )
        .unwrap();
        let client = build_client(
            &settings,
            Some("secret"),
            "http://upstream.invalid",
            false,
            Some(Duration::from_secs(5)),
        )
        .unwrap();
        let response = client
            .get("http://upstream.invalid/probe")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);

        let request = request_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        server.join().unwrap();
        assert!(request.starts_with("GET http://upstream.invalid/probe HTTP/1.1\r\n"));
        assert!(request
            .to_ascii_lowercase()
            .contains("proxy-authorization: basic ywxpy2u6c2vjcmv0\r\n"));
    }

    #[tokio::test]
    async fn socks5h_sends_the_destination_domain_to_the_proxy() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (destination_tx, destination_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();

            let mut greeting = [0_u8; 3];
            stream.read_exact(&mut greeting).unwrap();
            assert_eq!(greeting, [5, 1, 0]);
            stream.write_all(&[5, 0]).unwrap();

            let mut request = [0_u8; 4];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(request, [5, 1, 0, 3]);
            let mut domain_len = [0_u8; 1];
            stream.read_exact(&mut domain_len).unwrap();
            let mut domain = vec![0_u8; usize::from(domain_len[0])];
            stream.read_exact(&mut domain).unwrap();
            let mut port = [0_u8; 2];
            stream.read_exact(&mut port).unwrap();
            destination_tx
                .send((String::from_utf8(domain).unwrap(), u16::from_be_bytes(port)))
                .unwrap();
            stream.write_all(&[5, 1, 0, 1, 0, 0, 0, 0, 0, 0]).unwrap();
        });

        let (settings, _) = normalize_input(
            ProxySettingsInput {
                proxy_type: ProxyType::Socks5,
                host: address.ip().to_string(),
                port: Some(address.port()),
                ..manual_input()
            },
            None,
        )
        .unwrap();
        let client = build_client(
            &settings,
            None,
            "http://dns-only.invalid:8080",
            false,
            Some(Duration::from_secs(5)),
        )
        .unwrap();
        assert!(client
            .get("http://dns-only.invalid:8080/probe")
            .send()
            .await
            .is_err());

        assert_eq!(
            destination_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            ("dns-only.invalid".to_string(), 8080)
        );
        server.join().unwrap();
    }
}
