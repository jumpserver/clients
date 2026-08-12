use anyhow::{anyhow, Result};
use std::fmt;
use url::{Position, Url};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub(super) const MAX_ROUTE_COUNT: usize = 16;
pub(crate) const MAX_PAC_URL_LEN: usize = 2048;
const MAX_PROXY_HOST_LEN: usize = 255;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RouteSource {
    System,
    Pac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProxyScheme {
    Http,
    Https,
    Socks5,
}

impl fmt::Display for ProxyScheme {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Http => "http",
            Self::Https => "https",
            // Resolve destination DNS through the proxy.
            Self::Socks5 => "socks5h",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProxyDirective {
    Direct,
    Proxy {
        scheme: ProxyScheme,
        host: String,
        port: u16,
    },
    Unsupported {
        reason: String,
    },
}

impl ProxyDirective {
    pub(crate) fn proxy_url(&self) -> Result<Option<String>> {
        match self {
            Self::Direct => Ok(None),
            Self::Proxy { scheme, host, port } => {
                validate_proxy_host(host)?;
                if *port == 0 {
                    return Err(anyhow!("System proxy returned an invalid port"));
                }
                let host = if host.parse::<std::net::Ipv6Addr>().is_ok() {
                    format!("[{host}]")
                } else {
                    host.clone()
                };
                Ok(Some(format!("{scheme}://{host}:{port}")))
            }
            Self::Unsupported { reason } => Err(anyhow!(
                "System proxy route is unsupported and was not downgraded: {reason}"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RoutePlan {
    source: RouteSource,
    routes: Vec<ProxyDirective>,
}

impl RoutePlan {
    pub(crate) fn new(source: RouteSource, routes: Vec<ProxyDirective>) -> Result<Self> {
        if routes.is_empty() {
            return Err(anyhow!(
                "SystemProxyUnavailable: the system resolver returned no routes"
            ));
        }
        if routes.len() > MAX_ROUTE_COUNT {
            return Err(anyhow!(
                "SystemProxyUnavailable: the system resolver returned too many routes"
            ));
        }
        Ok(Self { source, routes })
    }

    pub(crate) fn source(&self) -> RouteSource {
        self.source
    }

    pub(crate) fn routes(&self) -> &[ProxyDirective] {
        &self.routes
    }

    /// Select the first route for metadata and single-route client construction.
    /// The request executor consumes `routes()` in order for safe connection fallback.
    pub(crate) fn primary(&self) -> Result<SelectedRoute> {
        let route = self
            .routes
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("SystemProxyUnavailable: empty route plan"))?;
        // Validate here so malformed or unknown first entries fail closed. A
        // malformed fallback remains visible in the full plan but is not used.
        route.proxy_url()?;
        Ok(SelectedRoute {
            route,
            fallback_count: self.routes.len().saturating_sub(1),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectedRoute {
    pub(crate) route: ProxyDirective,
    pub(crate) fallback_count: usize,
}

pub(crate) fn resolve(target: &str) -> Result<RoutePlan> {
    let url = validate_target(target)?;
    #[cfg(target_os = "macos")]
    return macos::resolve(&url);
    #[cfg(windows)]
    return windows::resolve(&url);
    #[cfg(target_os = "linux")]
    return linux::resolve(&url);
    #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
    Err(anyhow!(
        "SystemProxyUnavailable: system proxy is not supported on this platform"
    ))
}

pub(crate) fn resolve_pac(pac_url: &str, target: &str) -> Result<RoutePlan> {
    ensure_custom_pac_supported()?;
    let pac_url = validate_pac_url(pac_url)?;
    let target = validate_target(target)?;
    #[cfg(target_os = "macos")]
    return macos::resolve_pac(&pac_url, &target);
    #[cfg(windows)]
    return windows::resolve_pac(&pac_url, &target);
    #[cfg(target_os = "linux")]
    return linux::resolve_pac(&pac_url, &target);
    #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
    Err(anyhow!(
        "CustomPacUnsupported: explicit PAC URLs are not supported on this platform"
    ))
}

pub(crate) const fn custom_pac_supported() -> bool {
    cfg!(any(target_os = "macos", windows))
}

pub(crate) fn ensure_custom_pac_supported() -> Result<()> {
    if custom_pac_supported() {
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    return Err(anyhow!(
        "CustomPacUnsupported: explicit PAC URLs are not supported on Linux; use System proxy to inherit desktop PAC settings"
    ));
    #[cfg(not(target_os = "linux"))]
    Err(anyhow!(
        "CustomPacUnsupported: explicit PAC URLs are not supported on this platform"
    ))
}

pub(crate) fn validate_pac_url(value: &str) -> Result<Url> {
    if value.is_empty() {
        return Err(anyhow!("PAC URL is required in PAC mode"));
    }
    if value.len() > MAX_PAC_URL_LEN {
        return Err(anyhow!("PAC URL is too long"));
    }
    let url = Url::parse(value).map_err(|_| anyhow!("PAC URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(anyhow!("PAC URL must be an HTTP(S) URL"));
    }
    if raw_authority(value).is_none_or(str::is_empty) {
        return Err(anyhow!("PAC URL must contain a valid HTTP(S) authority"));
    }
    if has_url_credentials(value, &url) {
        return Err(anyhow!("PAC URL must not contain credentials"));
    }
    Ok(url)
}

fn validate_target(target: &str) -> Result<Url> {
    let url = Url::parse(target).map_err(|_| anyhow!("System proxy target URL is invalid"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(anyhow!("System proxy target must be an HTTP(S) URL"));
    }
    if has_url_credentials(target, &url) {
        return Err(anyhow!("System proxy target must not contain credentials"));
    }
    Ok(url)
}

fn has_url_credentials(raw_url: &str, url: &Url) -> bool {
    if !url[Position::BeforeUsername..Position::BeforeHost].is_empty() {
        return true;
    }

    // `url` normalizes an empty userinfo marker (`https://@host`) away. Check
    // only the original authority so an `@` in the path/query remains valid.
    raw_authority(raw_url).is_some_and(|authority| authority.contains('@'))
}

fn raw_authority(value: &str) -> Option<&str> {
    value
        .find(':')
        .and_then(|scheme_end| value[scheme_end + 1..].strip_prefix("//"))
        .and_then(|after_slashes| after_slashes.split(['/', '?', '#']).next())
}

pub(super) fn proxy_directive(
    scheme: ProxyScheme,
    host: impl Into<String>,
    port: u16,
) -> ProxyDirective {
    let host = host.into();
    if let Err(error) = validate_proxy_host(&host) {
        return unsupported(error.to_string());
    }
    if port == 0 {
        return unsupported("proxy port is zero");
    }
    ProxyDirective::Proxy { scheme, host, port }
}

#[cfg_attr(not(any(test, target_os = "linux")), allow(dead_code))]
pub(super) fn parse_proxy_uri(value: &str) -> ProxyDirective {
    let value = value.trim();
    if value.eq_ignore_ascii_case("direct") || value.eq_ignore_ascii_case("direct://") {
        return ProxyDirective::Direct;
    }
    let Ok(url) = Url::parse(value) else {
        return unsupported("resolver returned an invalid proxy URI");
    };
    if !url.username().is_empty() || url.password().is_some() {
        return unsupported("authenticated system proxy routes are not supported");
    }
    let scheme = match url.scheme().to_ascii_lowercase().as_str() {
        "http" => ProxyScheme::Http,
        "https" => ProxyScheme::Https,
        "socks" | "socks5" | "socks5h" => ProxyScheme::Socks5,
        other => return unsupported(format!("proxy scheme {other} is not supported")),
    };
    let Some(host) = url.host_str() else {
        return unsupported("proxy URI has no host");
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let port = url.port_or_known_default().unwrap_or(0);
    proxy_directive(scheme, host, port)
}

pub(super) fn unsupported(reason: impl Into<String>) -> ProxyDirective {
    ProxyDirective::Unsupported {
        reason: reason.into(),
    }
}

fn validate_proxy_host(host: &str) -> Result<()> {
    if host.is_empty()
        || host.len() > MAX_PROXY_HOST_LEN
        || host.contains(char::is_whitespace)
        || host.contains("://")
        || host.contains('/')
        || host.contains('?')
        || host.contains('#')
        || host.contains('@')
    {
        return Err(anyhow!("System proxy returned an invalid host"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_proxy_uris_without_credentials() {
        assert_eq!(
            parse_proxy_uri("http://proxy.example.com:3128"),
            ProxyDirective::Proxy {
                scheme: ProxyScheme::Http,
                host: "proxy.example.com".to_string(),
                port: 3128,
            }
        );
        assert_eq!(
            parse_proxy_uri("socks://[2001:db8::1]:1080"),
            ProxyDirective::Proxy {
                scheme: ProxyScheme::Socks5,
                host: "[2001:db8::1]".trim_matches(['[', ']']).to_string(),
                port: 1080,
            }
        );
        assert!(matches!(
            parse_proxy_uri("http://alice:secret@proxy.example.com:3128"),
            ProxyDirective::Unsupported { .. }
        ));
    }

    #[test]
    fn primary_route_keeps_fallback_count_and_fails_closed() {
        let plan = RoutePlan::new(
            RouteSource::Pac,
            vec![
                proxy_directive(ProxyScheme::Http, "proxy.example.com", 8080),
                ProxyDirective::Direct,
            ],
        )
        .unwrap();
        let selected = plan.primary().unwrap();
        assert_eq!(selected.fallback_count, 1);
        assert!(matches!(selected.route, ProxyDirective::Proxy { .. }));

        let unsupported = RoutePlan::new(
            RouteSource::System,
            vec![unsupported("unknown route"), ProxyDirective::Direct],
        )
        .unwrap();
        assert!(unsupported.primary().is_err());
    }

    #[test]
    fn invalid_targets_fail_before_native_resolution() {
        for target in [
            "file:///tmp/a",
            "https://user:pass@example.com",
            "not a url",
        ] {
            assert!(resolve(target).is_err());
        }
    }

    #[test]
    fn validates_bounded_http_pac_urls_without_credentials() {
        let url = validate_pac_url("https://proxy.example.com/config/proxy.pac?team=ops").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("proxy.example.com"));

        for value in [
            "",
            "file:///tmp/proxy.pac",
            "https://user:secret@proxy.example.com/proxy.pac",
            "https://@proxy.example.com/proxy.pac",
            "not a URL",
        ] {
            assert!(
                validate_pac_url(value).is_err(),
                "accepted invalid PAC URL: {value}"
            );
        }

        let oversized = format!("https://proxy.example.com/{}", "x".repeat(MAX_PAC_URL_LEN));
        assert!(validate_pac_url(&oversized).is_err());
    }
}
