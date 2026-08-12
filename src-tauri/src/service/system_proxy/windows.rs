use super::{
    proxy_directive, unsupported, ProxyDirective, ProxyScheme, RoutePlan, RouteSource,
    MAX_ROUTE_COUNT,
};
use anyhow::{anyhow, Result};
use std::ffi::c_void;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use url::Url;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{GlobalFree, ERROR_IO_PENDING, HGLOBAL};
use windows::Win32::Networking::WinHttp::*;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(1);

struct CurrentUserConfig {
    auto_detect: bool,
    pac_url: Option<String>,
    proxy: Option<String>,
    bypass: Option<String>,
}

pub(super) fn resolve(url: &Url) -> Result<RoutePlan> {
    let config = current_user_config()?;
    if !config.auto_detect && config.pac_url.is_none() {
        return resolve_static(url, &config);
    }
    resolve_auto(url, config.auto_detect, config.pac_url.as_deref())
}

pub(super) fn resolve_pac(pac_url: &Url, target_url: &Url) -> Result<RoutePlan> {
    resolve_auto(target_url, false, Some(pac_url.as_str()))
}

fn current_user_config() -> Result<CurrentUserConfig> {
    let mut config = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
    unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut config) }.map_err(|error| {
        anyhow!("SystemProxyUnavailable: read current-user proxy settings failed: {error}")
    })?;
    let result = CurrentUserConfig {
        auto_detect: config.fAutoDetect.as_bool(),
        pac_url: unsafe { take_winhttp_string(config.lpszAutoConfigUrl) },
        proxy: unsafe { take_winhttp_string(config.lpszProxy) },
        bypass: unsafe { take_winhttp_string(config.lpszProxyBypass) },
    };
    Ok(result)
}

unsafe fn take_winhttp_string(value: PWSTR) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let result = value.to_string().ok();
    let _ = GlobalFree(HGLOBAL(value.as_ptr().cast_mut().cast()));
    result.filter(|value| !value.trim().is_empty())
}

fn resolve_static(url: &Url, config: &CurrentUserConfig) -> Result<RoutePlan> {
    if config
        .bypass
        .as_deref()
        .is_some_and(|bypass| matches_bypass(url, bypass))
    {
        return RoutePlan::new(RouteSource::System, vec![ProxyDirective::Direct]);
    }
    let Some(proxy) = config.proxy.as_deref() else {
        return RoutePlan::new(RouteSource::System, vec![ProxyDirective::Direct]);
    };
    let routes = parse_static_proxy(proxy, url.scheme());
    RoutePlan::new(RouteSource::System, routes)
}

fn parse_static_proxy(value: &str, target_scheme: &str) -> Vec<ProxyDirective> {
    let mut selected: Option<(&str, ProxyScheme)> = None;
    if value.contains('=') {
        for entry in value
            .split(';')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            let Some((scheme, proxies)) = entry.split_once('=') else {
                continue;
            };
            let scheme = scheme.trim().to_ascii_lowercase();
            if scheme == target_scheme {
                // IE's http=/https= keys describe the destination protocol;
                // both values are conventional HTTP CONNECT proxies.
                selected = Some((proxies, ProxyScheme::Http));
                break;
            }
            if scheme == "socks" && selected.is_none() {
                selected = Some((proxies, ProxyScheme::Socks5));
            }
        }
    } else {
        selected = Some((value, ProxyScheme::Http));
    }

    let Some((proxies, scheme)) = selected else {
        return vec![ProxyDirective::Direct];
    };
    proxies
        .split(|character: char| character.is_whitespace() || character == ';')
        .filter(|proxy| !proxy.is_empty())
        .map(|proxy| parse_host_port(proxy, scheme))
        .collect()
}

fn parse_host_port(value: &str, scheme: ProxyScheme) -> ProxyDirective {
    let candidate = if value.contains("://") {
        value.to_string()
    } else {
        format!("http://{value}")
    };
    let Ok(url) = Url::parse(&candidate) else {
        return unsupported("Windows returned an invalid static proxy");
    };
    if !url.username().is_empty() || url.password().is_some() {
        return unsupported("authenticated system proxy routes are not supported");
    }
    let Some(host) = url.host_str() else {
        return unsupported("Windows static proxy has no host");
    };
    let port = url.port().unwrap_or(match scheme {
        ProxyScheme::Http => 80,
        ProxyScheme::Https => 443,
        ProxyScheme::Socks5 => 1080,
    });
    proxy_directive(scheme, host, port)
}

fn matches_bypass(url: &Url, bypass: &str) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    bypass
        .split(|character: char| character == ';' || character.is_whitespace())
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .any(|pattern| {
            if pattern.eq_ignore_ascii_case("<local>") {
                return !host.contains('.');
            }
            wildcard_match(&host, &pattern.trim_end_matches('.').to_ascii_lowercase())
        })
}

fn wildcard_match(value: &str, pattern: &str) -> bool {
    let (mut value_index, mut pattern_index) = (0, 0);
    let (mut star, mut retry) = (None, 0);
    let value = value.as_bytes();
    let pattern = pattern.as_bytes();
    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            value_index += 1;
            pattern_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star = Some(pattern_index);
            pattern_index += 1;
            retry = value_index;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            retry += 1;
            value_index = retry;
        } else {
            return false;
        }
    }
    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

struct CallbackState {
    result: Mutex<Option<std::result::Result<Vec<ProxyDirective>, String>>>,
    result_ready: Condvar,
    closed: Mutex<bool>,
    handle_closed: Condvar,
}

impl CallbackState {
    fn new() -> Self {
        Self {
            result: Mutex::new(None),
            result_ready: Condvar::new(),
            closed: Mutex::new(false),
            handle_closed: Condvar::new(),
        }
    }
}

unsafe extern "system" fn resolver_callback(
    resolver: *mut c_void,
    context: usize,
    status: u32,
    information: *mut c_void,
    _information_length: u32,
) {
    let raw_state = context as *const CallbackState;
    if raw_state.is_null() {
        return;
    }
    if status == WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING {
        let state = Arc::from_raw(raw_state);
        *state.closed.lock().expect("WinHTTP close lock poisoned") = true;
        state.handle_closed.notify_all();
        return;
    }

    Arc::increment_strong_count(raw_state);
    let state = Arc::from_raw(raw_state);
    let result = if status == WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE {
        read_proxy_result(resolver)
    } else if status == WINHTTP_CALLBACK_STATUS_REQUEST_ERROR {
        let error = (information as *const WINHTTP_ASYNC_RESULT).as_ref();
        Err(format!(
            "WinHTTP asynchronous proxy resolution failed with error {}",
            error.map_or(0, |error| error.dwError)
        ))
    } else {
        return;
    };
    let mut slot = state.result.lock().expect("WinHTTP result lock poisoned");
    if slot.is_none() {
        *slot = Some(result);
        state.result_ready.notify_all();
    }
}

unsafe fn read_proxy_result(
    resolver: *mut c_void,
) -> std::result::Result<Vec<ProxyDirective>, String> {
    let mut result = WINHTTP_PROXY_RESULT::default();
    let code = WinHttpGetProxyResult(resolver, &mut result);
    if code != 0 {
        return Err(format!("WinHttpGetProxyResult failed with error {code}"));
    }
    if result.cEntries as usize > MAX_ROUTE_COUNT {
        WinHttpFreeProxyResult(&mut result);
        return Err("WinHTTP returned too many proxy routes".to_string());
    }
    let entries = if result.pEntries.is_null() {
        &[][..]
    } else {
        std::slice::from_raw_parts(result.pEntries, result.cEntries as usize)
    };
    let routes = entries
        .iter()
        .map(|entry| {
            if !entry.fProxy.as_bool() {
                return Ok(ProxyDirective::Direct);
            }
            let host = entry
                .pwszProxy
                .to_string()
                .map_err(|_| "WinHTTP returned a non-Unicode proxy host")?;
            let scheme = if entry.ProxyScheme == WINHTTP_INTERNET_SCHEME_HTTP {
                ProxyScheme::Http
            } else if entry.ProxyScheme == WINHTTP_INTERNET_SCHEME_HTTPS {
                ProxyScheme::Https
            } else if entry.ProxyScheme == WINHTTP_INTERNET_SCHEME_SOCKS {
                ProxyScheme::Socks5
            } else {
                return Ok(unsupported(format!(
                    "WinHTTP proxy scheme {} is not supported",
                    entry.ProxyScheme.0
                )));
            };
            Ok(proxy_directive(scheme, host, entry.ProxyPort))
        })
        .collect::<std::result::Result<Vec<_>, &'static str>>()
        .map_err(str::to_string);
    WinHttpFreeProxyResult(&mut result);
    routes
}

fn auto_proxy_options(auto_detect: bool, pac_url: Option<&HSTRING>) -> WINHTTP_AUTOPROXY_OPTIONS {
    WINHTTP_AUTOPROXY_OPTIONS {
        dwFlags: (auto_detect as u32) * WINHTTP_AUTOPROXY_AUTO_DETECT
            | (pac_url.is_some() as u32) * WINHTTP_AUTOPROXY_CONFIG_URL,
        dwAutoDetectFlags: if auto_detect {
            WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A
        } else {
            0
        },
        lpszAutoConfigUrl: pac_url
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or(PCWSTR::null()),
        lpvReserved: std::ptr::null_mut(),
        dwReserved: 0,
        // PAC/WPAD discovery is an untrusted bootstrap step. Never emit the
        // current user's domain credentials in response to its challenge.
        fAutoLogonIfChallenged: false.into(),
    }
}

fn resolve_auto(url: &Url, auto_detect: bool, pac_url: Option<&str>) -> Result<RoutePlan> {
    let deadline = Instant::now() + RESOLVE_TIMEOUT;
    let agent = HSTRING::from("JumpServer Client system proxy resolver");
    let session = unsafe {
        WinHttpOpen(
            &agent,
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            WINHTTP_FLAG_ASYNC,
        )
    };
    if session.is_null() {
        return Err(anyhow!("SystemProxyUnavailable: WinHttpOpen failed"));
    }
    let mut resolver = std::ptr::null_mut();
    let create_code = unsafe { WinHttpCreateProxyResolver(session, &mut resolver) };
    if create_code != 0 || resolver.is_null() {
        let _ = unsafe { WinHttpCloseHandle(session) };
        return Err(anyhow!(
            "SystemProxyUnavailable: WinHttpCreateProxyResolver failed with error {create_code}"
        ));
    }

    let state = Arc::new(CallbackState::new());
    let raw_state = Arc::into_raw(state.clone());
    let previous = unsafe {
        WinHttpSetStatusCallback(
            resolver,
            Some(resolver_callback),
            WINHTTP_CALLBACK_FLAG_GETPROXYFORURL_COMPLETE
                | WINHTTP_CALLBACK_FLAG_REQUEST_ERROR
                | WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING,
            0,
        )
    };
    if previous.map(|callback| callback as usize) == Some(usize::MAX) {
        unsafe { drop(Arc::from_raw(raw_state)) };
        let _ = unsafe { WinHttpCloseHandle(resolver) };
        let _ = unsafe { WinHttpCloseHandle(session) };
        return Err(anyhow!(
            "SystemProxyUnavailable: install WinHTTP resolver callback failed"
        ));
    }

    let pac_url = pac_url.map(HSTRING::from);
    let options = auto_proxy_options(auto_detect, pac_url.as_ref());
    let target = HSTRING::from(url.as_str());
    let start_code =
        unsafe { WinHttpGetProxyForUrlEx(resolver, &target, &options, raw_state as usize) };
    if start_code != ERROR_IO_PENDING.0 && start_code != 0 {
        // Detach the callback before releasing its context. Closing an async
        // handle can otherwise race a HANDLE_CLOSING callback with freed data.
        let _ = unsafe { WinHttpSetStatusCallback(resolver, None, 0, 0) };
        unsafe { drop(Arc::from_raw(raw_state)) };
        let _ = unsafe { WinHttpCloseHandle(resolver) };
        let _ = unsafe { WinHttpCloseHandle(session) };
        return Err(anyhow!(
            "SystemProxyUnavailable: WinHttpGetProxyForUrlEx failed with error {start_code}"
        ));
    }

    let mut slot = state.result.lock().expect("WinHTTP result lock poisoned");
    let remaining = deadline.saturating_duration_since(Instant::now());
    let (guard, wait) = state
        .result_ready
        .wait_timeout_while(slot, remaining, |result| result.is_none())
        .expect("WinHTTP result lock poisoned");
    slot = guard;
    let result = if wait.timed_out() && slot.is_none() {
        Err(anyhow!(
            "SystemProxyUnavailable: WinHTTP proxy resolution timed out"
        ))
    } else {
        match slot.take() {
            Some(Ok(routes)) => RoutePlan::new(RouteSource::Pac, routes),
            Some(Err(error)) => Err(anyhow!(
                "SystemProxyUnavailable: WinHTTP proxy resolution failed: {error}"
            )),
            None => Err(anyhow!(
                "SystemProxyUnavailable: WinHTTP returned no proxy result"
            )),
        }
    };
    drop(slot);

    let _ = unsafe { WinHttpCloseHandle(resolver) };
    wait_for_close(
        &state,
        CLOSE_TIMEOUT.min(deadline.saturating_duration_since(Instant::now())),
    );
    let _ = unsafe { WinHttpCloseHandle(session) };
    result
}

fn wait_for_close(state: &CallbackState, timeout: Duration) {
    let closed = state.closed.lock().expect("WinHTTP close lock poisoned");
    let _ = state
        .handle_closed
        .wait_timeout_while(closed, timeout, |closed| !*closed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pac_options_disable_auto_detection() {
        let pac_url = HSTRING::from("https://proxy.example.com/proxy.pac");
        let options = auto_proxy_options(false, Some(&pac_url));

        assert_eq!(options.dwFlags, WINHTTP_AUTOPROXY_CONFIG_URL);
        assert_eq!(options.dwAutoDetectFlags, 0);
        assert_eq!(options.lpszAutoConfigUrl.as_ptr(), pac_url.as_ptr());
        assert!(!options.fAutoLogonIfChallenged.as_bool());
    }

    #[test]
    fn auto_detect_options_set_detection_flags_without_a_pac_pointer() {
        let options = auto_proxy_options(true, None);

        assert_eq!(options.dwFlags, WINHTTP_AUTOPROXY_AUTO_DETECT);
        assert_eq!(
            options.dwAutoDetectFlags,
            WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A
        );
        assert!(options.lpszAutoConfigUrl.is_null());
    }
}
