use super::{parse_proxy_uri, ProxyDirective, RoutePlan, RouteSource};
use anyhow::{anyhow, Result};
use gio::prelude::*;
use std::sync::mpsc;
use std::time::Duration;
use url::Url;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) fn resolve(url: &Url) -> Result<RoutePlan> {
    let target = url.as_str().to_string();
    let cancellable = gio::Cancellable::new();
    let worker_cancellable = cancellable.clone();
    let (sender, receiver) = mpsc::sync_channel(1);

    std::thread::Builder::new()
        .name("system-proxy-gio".to_string())
        .spawn(move || {
            let resolver = gio::ProxyResolver::default();
            let result = if !resolver.is_supported() {
                Err(
                    "GIO proxy resolver is unsupported; install glib-networking/libproxy"
                        .to_string(),
                )
            } else {
                resolver
                    .lookup(&target, Some(&worker_cancellable))
                    .map(|routes| routes.into_iter().map(|route| route.to_string()).collect())
                    .map_err(|error| error.to_string())
            };
            let _ = sender.send(result);
        })
        .map_err(|error| anyhow!("SystemProxyUnavailable: start GIO resolver failed: {error}"))?;

    let raw_routes: Vec<String> = match receiver.recv_timeout(RESOLVE_TIMEOUT) {
        Ok(Ok(routes)) => routes,
        Ok(Err(error)) => {
            return Err(anyhow!(
                "SystemProxyUnavailable: GIO proxy resolution failed: {error}"
            ));
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            cancellable.cancel();
            return Err(anyhow!(
                "SystemProxyUnavailable: GIO proxy resolution timed out"
            ));
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(anyhow!(
                "SystemProxyUnavailable: GIO proxy resolver stopped unexpectedly"
            ));
        }
    };

    let routes: Vec<ProxyDirective> = raw_routes
        .iter()
        .map(|route| parse_proxy_uri(route))
        .collect();
    RoutePlan::new(RouteSource::System, routes)
}

pub(super) fn resolve_pac(_pac_url: &Url, _target_url: &Url) -> Result<RoutePlan> {
    Err(anyhow!(
        "CustomPacUnsupported: explicit PAC URLs are not supported on Linux; GIO only resolves system proxy settings"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pac_is_not_misreported_as_system_resolution() {
        let pac_url = Url::parse("https://proxy.example.com/proxy.pac").unwrap();
        let target_url = Url::parse("https://target.example.com/api").unwrap();

        let error = resolve_pac(&pac_url, &target_url).unwrap_err().to_string();
        assert!(error.contains("CustomPacUnsupported"));
        assert!(error.contains("GIO only resolves system proxy settings"));
    }
}
