use super::{
    proxy_directive, unsupported, ProxyDirective, ProxyScheme, RoutePlan, RouteSource,
    MAX_ROUTE_COUNT,
};
use anyhow::{anyhow, Result};
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::error::CFError;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURLRef, CFURL};
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{kCFAllocatorDefault, CFRelease, CFRetain, CFTypeRef};
use core_foundation_sys::dictionary::{CFDictionaryGetValueIfPresent, CFDictionaryRef};
use core_foundation_sys::error::CFErrorRef;
use core_foundation_sys::runloop::{
    kCFRunLoopDefaultMode, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRemoveSource,
    CFRunLoopRunInMode, CFRunLoopSourceInvalidate, CFRunLoopSourceRef,
};
use core_foundation_sys::url::CFURLCreateWithString;
use std::ffi::c_void;
use std::ptr;
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};
use url::Url;

const PAC_TIMEOUT: Duration = Duration::from_secs(5);
const INLINE_PAC_QUEUE_CAPACITY: usize = 32;
static INLINE_PAC_WORKER: OnceLock<std::result::Result<mpsc::SyncSender<InlinePacJob>, String>> =
    OnceLock::new();

struct InlinePacJob {
    script: String,
    target: String,
    deadline: Instant,
    result: mpsc::SyncSender<std::result::Result<Vec<ProxyDirective>, String>>,
}

#[repr(C)]
struct CFStreamClientContext {
    version: isize,
    info: *mut c_void,
    retain: Option<extern "C" fn(*const c_void) -> *const c_void>,
    release: Option<extern "C" fn(*const c_void)>,
    copy_description: Option<extern "C" fn(*const c_void) -> CFStringRef>,
}

type ProxyAutoConfigurationCallback = extern "C" fn(*mut c_void, CFArrayRef, CFErrorRef);

#[link(name = "CFNetwork", kind = "framework")]
extern "C" {
    fn CFNetworkCopySystemProxySettings() -> CFDictionaryRef;
    fn CFNetworkCopyProxiesForURL(url: CFURLRef, proxy_settings: CFDictionaryRef) -> CFArrayRef;
    fn CFNetworkCopyProxiesForAutoConfigurationScript(
        script: CFStringRef,
        target_url: CFURLRef,
        error: *mut CFErrorRef,
    ) -> CFArrayRef;
    fn CFNetworkExecuteProxyAutoConfigurationURL(
        pac_url: CFURLRef,
        target_url: CFURLRef,
        callback: ProxyAutoConfigurationCallback,
        context: *mut CFStreamClientContext,
    ) -> CFRunLoopSourceRef;

    static kCFProxyTypeKey: CFStringRef;
    static kCFProxyHostNameKey: CFStringRef;
    static kCFProxyPortNumberKey: CFStringRef;
    static kCFProxyAutoConfigurationURLKey: CFStringRef;
    static kCFProxyAutoConfigurationJavaScriptKey: CFStringRef;
    static kCFProxyUsernameKey: CFStringRef;
    static kCFProxyPasswordKey: CFStringRef;

    static kCFProxyTypeNone: CFStringRef;
    static kCFProxyTypeHTTP: CFStringRef;
    static kCFProxyTypeHTTPS: CFStringRef;
    static kCFProxyTypeSOCKS: CFStringRef;
    static kCFProxyTypeFTP: CFStringRef;
    static kCFProxyTypeAutoConfigurationURL: CFStringRef;
    static kCFProxyTypeAutoConfigurationJavaScript: CFStringRef;
}

pub(super) fn resolve(url: &Url) -> Result<RoutePlan> {
    let target_url = cf_url(url.as_str())?;
    let settings_ref = unsafe { CFNetworkCopySystemProxySettings() };
    if settings_ref.is_null() {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork returned no system proxy settings"
        ));
    }
    let settings: CFDictionary = unsafe { CFDictionary::wrap_under_create_rule(settings_ref) };
    let proxies_ref = unsafe {
        CFNetworkCopyProxiesForURL(
            target_url.as_concrete_TypeRef(),
            settings.as_concrete_TypeRef(),
        )
    };
    if proxies_ref.is_null() {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork failed to resolve the target URL"
        ));
    }
    let proxies = unsafe { CFArray::<CFType>::wrap_under_create_rule(proxies_ref) };
    let (routes, source) = parse_proxy_array(&proxies, &target_url, true)?;
    RoutePlan::new(source, routes)
}

pub(super) fn resolve_pac(pac_url: &Url, target_url: &Url) -> Result<RoutePlan> {
    let pac_url = cf_url(pac_url.as_str())?;
    let target_url = cf_url(target_url.as_str())?;
    // CFNetwork owns both PAC retrieval and execution. It may normalize the
    // target URL before invoking FindProxyForURL, as covered by the inline PAC
    // compatibility test below.
    let proxies = execute_pac_url(&pac_url, &target_url)?;
    let (routes, _) = parse_proxy_array(&proxies, &target_url, false)?;
    RoutePlan::new(RouteSource::Pac, routes)
}

fn cf_url(value: &str) -> Result<CFURL> {
    let string = CFString::new(value);
    let url_ref = unsafe {
        CFURLCreateWithString(
            kCFAllocatorDefault,
            string.as_concrete_TypeRef(),
            ptr::null(),
        )
    };
    if url_ref.is_null() {
        return Err(anyhow!("System proxy target URL could not be converted"));
    }
    Ok(unsafe { CFURL::wrap_under_create_rule(url_ref) })
}

fn parse_proxy_array(
    proxies: &CFArray<CFType>,
    target_url: &CFURL,
    allow_pac: bool,
) -> Result<(Vec<ProxyDirective>, RouteSource)> {
    let mut routes = Vec::new();
    let mut source = RouteSource::System;
    let count = unsafe { CFArrayGetCount(proxies.as_concrete_TypeRef()) };
    if count < 0 || count as usize > MAX_ROUTE_COUNT {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork returned too many routes"
        ));
    }
    for index in 0..count {
        let value = unsafe { CFArrayGetValueAtIndex(proxies.as_concrete_TypeRef(), index) };
        if value.is_null() {
            routes.push(unsupported("CFNetwork returned an empty proxy entry"));
            continue;
        }
        let value = unsafe { CFType::wrap_under_get_rule(value as CFTypeRef) };
        let Some(dictionary) = value.downcast::<CFDictionary>() else {
            routes.push(unsupported(
                "CFNetwork returned a non-dictionary proxy entry",
            ));
            continue;
        };

        if dictionary_value(dictionary.as_concrete_TypeRef(), unsafe {
            kCFProxyUsernameKey
        })
        .is_some()
            || dictionary_value(dictionary.as_concrete_TypeRef(), unsafe {
                kCFProxyPasswordKey
            })
            .is_some()
        {
            routes.push(unsupported(
                "authenticated system proxy routes require a separate credential integration",
            ));
            continue;
        }

        let Some(proxy_type) =
            dictionary_string(dictionary.as_concrete_TypeRef(), unsafe { kCFProxyTypeKey })
        else {
            routes.push(unsupported("CFNetwork proxy entry has no type"));
            continue;
        };

        if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeNone }) {
            routes.push(ProxyDirective::Direct);
        } else if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeHTTP }) {
            routes.push(dictionary_proxy(&dictionary, ProxyScheme::Http));
        } else if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeHTTPS }) {
            // CFNetwork uses this type for the proxy selected for an HTTPS
            // destination, including a PAC `PROXY` directive. The transport to
            // conventional system proxies remains plain HTTP CONNECT.
            routes.push(dictionary_proxy(&dictionary, ProxyScheme::Http));
        } else if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeSOCKS }) {
            routes.push(dictionary_proxy(&dictionary, ProxyScheme::Socks5));
        } else if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeFTP }) {
            routes.push(unsupported("FTP proxy routes are not supported"));
        } else if cf_string_equals(&proxy_type, unsafe { kCFProxyTypeAutoConfigurationURL }) {
            if !allow_pac {
                routes.push(unsupported("CFNetwork returned a recursive PAC route"));
                continue;
            }
            let Some(pac_url) = dictionary_value(dictionary.as_concrete_TypeRef(), unsafe {
                kCFProxyAutoConfigurationURLKey
            })
            .and_then(|value| value.downcast::<CFURL>()) else {
                routes.push(unsupported("PAC route has no valid configuration URL"));
                continue;
            };
            let pac_result = execute_pac_url(&pac_url, target_url)?;
            let (pac_routes, _) = parse_proxy_array(&pac_result, target_url, false)?;
            routes.extend(pac_routes);
            source = RouteSource::Pac;
        } else if cf_string_equals(&proxy_type, unsafe {
            kCFProxyTypeAutoConfigurationJavaScript
        }) {
            if !allow_pac {
                routes.push(unsupported("CFNetwork returned a recursive PAC script"));
                continue;
            }
            let Some(script) = dictionary_string(dictionary.as_concrete_TypeRef(), unsafe {
                kCFProxyAutoConfigurationJavaScriptKey
            }) else {
                routes.push(unsupported("PAC route has no JavaScript"));
                continue;
            };
            let pac_routes = execute_pac_script(&script, target_url)?;
            routes.extend(pac_routes);
            source = RouteSource::Pac;
        } else {
            routes.push(unsupported(format!(
                "CFNetwork proxy type {} is not supported",
                proxy_type
            )));
        }
    }
    Ok((routes, source))
}

fn dictionary_proxy(dictionary: &CFDictionary, scheme: ProxyScheme) -> ProxyDirective {
    let Some(host) = dictionary_string(dictionary.as_concrete_TypeRef(), unsafe {
        kCFProxyHostNameKey
    }) else {
        return unsupported("CFNetwork proxy entry has no host");
    };
    let Some(port) = dictionary_value(dictionary.as_concrete_TypeRef(), unsafe {
        kCFProxyPortNumberKey
    })
    .and_then(|value| value.downcast::<CFNumber>())
    .and_then(|number| number.to_i64())
    .and_then(|number| u16::try_from(number).ok()) else {
        return unsupported("CFNetwork proxy entry has an invalid port");
    };
    proxy_directive(scheme, host.to_string(), port)
}

fn dictionary_string(dictionary: CFDictionaryRef, key: CFStringRef) -> Option<CFString> {
    dictionary_value(dictionary, key).and_then(|value| value.downcast::<CFString>())
}

fn dictionary_value(dictionary: CFDictionaryRef, key: CFStringRef) -> Option<CFType> {
    let mut value: *const c_void = ptr::null();
    let found =
        unsafe { CFDictionaryGetValueIfPresent(dictionary, key as *const c_void, &mut value) };
    (found != 0 && !value.is_null())
        .then(|| unsafe { CFType::wrap_under_get_rule(value as CFTypeRef) })
}

fn cf_string_equals(value: &CFString, expected: CFStringRef) -> bool {
    unsafe { core_foundation_sys::base::CFEqual(value.as_CFTypeRef(), expected as CFTypeRef) != 0 }
}

fn execute_pac_script(script: &CFString, target_url: &CFURL) -> Result<Vec<ProxyDirective>> {
    let deadline = Instant::now() + PAC_TIMEOUT;
    let (result_sender, result_receiver) = mpsc::sync_channel(1);
    let job = InlinePacJob {
        script: script.to_string(),
        target: target_url.get_string().to_string(),
        deadline,
        result: result_sender,
    };
    let sender = inline_pac_worker()?;
    sender.try_send(job).map_err(|error| match error {
        mpsc::TrySendError::Full(_) => {
            anyhow!("SystemProxyUnavailable: inline PAC evaluation queue is full")
        }
        mpsc::TrySendError::Disconnected(_) => {
            anyhow!("SystemProxyUnavailable: inline PAC worker stopped unexpectedly")
        }
    })?;

    let remaining = deadline.saturating_duration_since(Instant::now());
    match result_receiver.recv_timeout(remaining) {
        Ok(Ok(routes)) => Ok(routes),
        Ok(Err(error)) => Err(anyhow!(error)),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(anyhow!(
            "SystemProxyUnavailable: inline PAC evaluation timed out"
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow!(
            "SystemProxyUnavailable: inline PAC worker stopped unexpectedly"
        )),
    }
}

fn inline_pac_worker() -> Result<&'static mpsc::SyncSender<InlinePacJob>> {
    INLINE_PAC_WORKER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::sync_channel(INLINE_PAC_QUEUE_CAPACITY);
            std::thread::Builder::new()
                .name("system-proxy-inline-pac".to_string())
                .spawn(move || run_inline_pac_worker(receiver))
                .map(|_| sender)
                .map_err(|error| format!("start inline PAC worker failed: {error}"))
        })
        .as_ref()
        .map_err(|error| anyhow!("SystemProxyUnavailable: {error}"))
}

fn run_inline_pac_worker(receiver: mpsc::Receiver<InlinePacJob>) {
    while let Ok(job) = receiver.recv() {
        if Instant::now() >= job.deadline {
            let _ = job.result.send(Err(
                "SystemProxyUnavailable: inline PAC evaluation expired in queue".to_string(),
            ));
            continue;
        }
        let result =
            evaluate_pac_script(&job.script, &job.target).map_err(|error| error.to_string());
        let _ = job.result.send(result);
    }
}

fn evaluate_pac_script(script: &str, target: &str) -> Result<Vec<ProxyDirective>> {
    let target_url = cf_url(target)?;
    let script = CFString::new(script);
    let proxies = copy_proxies_for_pac_script(&script, &target_url)?;
    let (routes, _) = parse_proxy_array(&proxies, &target_url, false)?;
    Ok(routes)
}

fn copy_proxies_for_pac_script(script: &CFString, target_url: &CFURL) -> Result<CFArray<CFType>> {
    let mut error = ptr::null_mut();
    let proxies_ref = unsafe {
        CFNetworkCopyProxiesForAutoConfigurationScript(
            script.as_concrete_TypeRef(),
            target_url.as_concrete_TypeRef(),
            &mut error,
        )
    };
    if !error.is_null() {
        unsafe { CFRelease(error as CFTypeRef) };
    }
    if proxies_ref.is_null() {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork PAC script evaluation failed"
        ));
    }
    Ok(unsafe { CFArray::<CFType>::wrap_under_create_rule(proxies_ref) })
}

struct PacState {
    completed: bool,
    proxies: CFArrayRef,
    error: Option<String>,
}

extern "C" fn pac_callback(client: *mut c_void, proxies: CFArrayRef, error: CFErrorRef) {
    if client.is_null() {
        return;
    }
    let state = unsafe { &mut *(client as *mut PacState) };
    state.completed = true;
    state.error =
        (!error.is_null()).then(|| unsafe { CFError::wrap_under_get_rule(error).to_string() });
    if error.is_null() && !proxies.is_null() {
        unsafe {
            CFRetain(proxies as CFTypeRef);
        }
        state.proxies = proxies;
    }
}

fn execute_pac_url(pac_url: &CFURL, target_url: &CFURL) -> Result<CFArray<CFType>> {
    let mut state = PacState {
        completed: false,
        proxies: ptr::null(),
        error: None,
    };
    let mut context = CFStreamClientContext {
        version: 0,
        info: (&mut state as *mut PacState).cast(),
        retain: None,
        release: None,
        copy_description: None,
    };
    let source = unsafe {
        CFNetworkExecuteProxyAutoConfigurationURL(
            pac_url.as_concrete_TypeRef(),
            target_url.as_concrete_TypeRef(),
            pac_callback,
            &mut context,
        )
    };
    if source.is_null() {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork could not start PAC evaluation"
        ));
    }

    let run_loop = unsafe { CFRunLoopGetCurrent() };
    unsafe {
        CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode);
    }
    let deadline = Instant::now() + PAC_TIMEOUT;
    loop {
        if state.completed {
            break;
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline.saturating_duration_since(now).as_secs_f64();
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, remaining.min(0.1), 1);
        }
    }
    unsafe {
        CFRunLoopSourceInvalidate(source);
        CFRunLoopRemoveSource(run_loop, source, kCFRunLoopDefaultMode);
        CFRelease(source as CFTypeRef);
    }

    if !state.completed {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork PAC evaluation timed out"
        ));
    }
    if let Some(error) = state.error {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork PAC evaluation failed: {error}"
        ));
    }
    if state.proxies.is_null() {
        return Err(anyhow!(
            "SystemProxyUnavailable: CFNetwork PAC evaluation returned no routes"
        ));
    }
    Ok(unsafe { CFArray::<CFType>::wrap_under_create_rule(state.proxies) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pac_url_is_fetched_and_preserves_route_order() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let (stop_sender, stop_receiver) = mpsc::sync_channel(1);
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(6);
            let mut request_count = 0;
            loop {
                if stop_receiver.try_recv().is_ok() {
                    assert!(request_count > 0, "CFNetwork did not request the PAC URL");
                    return;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        request_count += 1;
                        stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .unwrap();
                        let mut request = [0_u8; 2048];
                        let _ = stream.read(&mut request);
                        let script = b"function FindProxyForURL(url, host) { return 'PROXY 127.0.0.1:8080; DIRECT'; }";
                        write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            script.len()
                        )
                        .unwrap();
                        stream.write_all(script).unwrap();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!("CFNetwork did not request the PAC URL");
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept PAC request failed: {error}"),
                }
            }
        });

        let pac_url = Url::parse(&format!("http://{address}/proxy.pac")).unwrap();
        let target_url = Url::parse("https://example.com/api/assets").unwrap();
        let plan = resolve_pac(&pac_url, &target_url);
        stop_sender.send(()).unwrap();
        server.join().unwrap();
        let plan = plan.unwrap();

        assert_eq!(plan.source(), RouteSource::Pac);
        let selected = plan.primary().unwrap();
        assert_eq!(
            selected.route,
            proxy_directive(ProxyScheme::Http, "127.0.0.1", 8080)
        );
        assert_eq!(selected.fallback_count, 1);
    }

    #[test]
    fn pac_proxy_for_https_target_uses_plain_http_connect() {
        let script = CFString::new(
            "function FindProxyForURL(url, host) { return 'PROXY 127.0.0.1:8080; DIRECT'; }",
        );
        let target = cf_url("https://example.com/api/assets").unwrap();
        assert_eq!(
            execute_pac_script(&script, &target).unwrap(),
            vec![
                proxy_directive(ProxyScheme::Http, "127.0.0.1", 8080),
                ProxyDirective::Direct,
            ]
        );
    }

    #[test]
    fn inline_pac_preserves_order_with_cfnetwork_url_sanitization() {
        let script = CFString::new(
            r#"function FindProxyForURL(url, host) {
                if (url == 'http://example.com/') {
                    return 'PROXY 127.0.0.1:8080; DIRECT';
                }
                return 'DIRECT';
            }"#,
        );
        let api_url = cf_url("http://example.com/api/assets?route=proxy").unwrap();
        assert_eq!(
            execute_pac_script(&script, &api_url).unwrap(),
            vec![
                proxy_directive(ProxyScheme::Http, "127.0.0.1", 8080),
                ProxyDirective::Direct,
            ]
        );
    }

    #[test]
    fn inline_pac_serializes_concurrent_evaluations() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(5));
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let script = CFString::new(
                        "function FindProxyForURL(url, host) { return 'PROXY 127.0.0.1:8080; DIRECT'; }",
                    );
                    let target = cf_url("https://example.com/api/assets").unwrap();
                    barrier.wait();
                    execute_pac_script(&script, &target).unwrap()
                })
            })
            .collect();
        barrier.wait();

        for worker in workers {
            assert_eq!(
                worker.join().unwrap(),
                vec![
                    proxy_directive(ProxyScheme::Http, "127.0.0.1", 8080),
                    ProxyDirective::Direct,
                ]
            );
        }
    }
}
