use anyhow::{bail, Context, Result};
use oauth2::{AsyncHttpClient, HttpRequest, HttpResponse};
use reqwest::{
    header::{
        HeaderMap, AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, COOKIE, HOST,
        LOCATION, PROXY_AUTHORIZATION, REFERER, TRANSFER_ENCODING, WWW_AUTHENTICATE,
    },
    Client, Method, Request, Response, StatusCode, Version,
};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use url::Url;

use crate::service::proxy::{ProxyExecutionResult, ProxyManager};

const MAX_REDIRECTS: usize = 10;

pub(crate) fn request_builder_client() -> Result<Client> {
    Client::builder()
        .no_proxy()
        .build()
        .context("build proxy-neutral HTTP request client failed")
}

pub(crate) async fn execute_proxy_request(
    manager: &ProxyManager,
    request: Request,
    disable_redirects: bool,
    timeout: Option<Duration>,
) -> Result<ProxyExecutionResult> {
    let target = request.url().as_str().to_string();
    let manager = manager.clone();
    let plan = tokio::task::spawn_blocking(move || {
        manager.execution_plan_for_target(&target, disable_redirects, timeout)
    })
    .await
    .context("proxy resolver task failed")??;
    plan.execute(request).await.map_err(Into::into)
}

pub(crate) async fn execute_api_request(
    manager: &ProxyManager,
    mut request: Request,
) -> Result<Response> {
    let mut redirects = 0;

    loop {
        let previous_url = request.url().clone();
        let previous_method = request.method().clone();
        let previous_headers = request.headers().clone();
        let previous_timeout = request.timeout().copied();
        let previous_version = request.version();
        let replay = request.try_clone();

        let response = execute_proxy_request(manager, request, true, None)
            .await?
            .response;
        let Some(next_url) = redirect_target(&response, &previous_url)? else {
            return Ok(response);
        };

        if redirects >= MAX_REDIRECTS {
            bail!("HTTP redirect limit exceeded ({MAX_REDIRECTS})");
        }

        let Some(next_request) = prepare_redirect_request(
            replay,
            previous_method,
            previous_url,
            previous_headers,
            previous_timeout,
            previous_version,
            response.status(),
            next_url,
        ) else {
            return Ok(response);
        };

        request = next_request;
        redirects += 1;
    }
}

fn redirect_target(response: &Response, previous_url: &Url) -> Result<Option<Url>> {
    if !matches!(
        response.status(),
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    ) {
        return Ok(None);
    }

    let Some(location) = response.headers().get(LOCATION) else {
        return Ok(None);
    };
    let location = location
        .to_str()
        .context("redirect Location is not valid UTF-8")?;
    if redirect_location_has_userinfo(location) {
        bail!("redirect target must not contain credentials");
    }
    let next_url = previous_url
        .join(location)
        .context("redirect Location is not a valid URL")?;

    if !matches!(next_url.scheme(), "http" | "https") {
        bail!("redirect target must use HTTP or HTTPS");
    }
    if !next_url.username().is_empty() || next_url.password().is_some() {
        bail!("redirect target must not contain credentials");
    }

    Ok(Some(next_url))
}

#[allow(clippy::too_many_arguments)]
fn prepare_redirect_request(
    replay: Option<Request>,
    previous_method: Method,
    previous_url: Url,
    previous_headers: HeaderMap,
    previous_timeout: Option<Duration>,
    previous_version: Version,
    status: StatusCode,
    next_url: Url,
) -> Option<Request> {
    let next_url_for_headers = next_url.clone();
    let (next_method, drop_body) = match status {
        StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND if previous_method == Method::POST => {
            (Method::GET, true)
        }
        StatusCode::SEE_OTHER => (
            if previous_method == Method::HEAD {
                Method::HEAD
            } else {
                Method::GET
            },
            true,
        ),
        StatusCode::MOVED_PERMANENTLY
        | StatusCode::FOUND
        | StatusCode::TEMPORARY_REDIRECT
        | StatusCode::PERMANENT_REDIRECT => (previous_method, false),
        _ => return None,
    };

    let mut request = match replay {
        Some(request) => request,
        None if drop_body => {
            let mut request = Request::new(next_method.clone(), next_url.clone());
            *request.headers_mut() = previous_headers;
            *request.timeout_mut() = previous_timeout;
            *request.version_mut() = previous_version;
            request
        }
        None => return None,
    };

    *request.method_mut() = next_method;
    *request.url_mut() = next_url;
    if drop_body {
        *request.body_mut() = None;
        drop_payload_headers(request.headers_mut());
    }
    sanitize_redirect_headers(request.headers_mut(), &previous_url, &next_url_for_headers);
    Some(request)
}

fn drop_payload_headers(headers: &mut HeaderMap) {
    for name in [
        CONTENT_TYPE,
        CONTENT_LENGTH,
        CONTENT_ENCODING,
        TRANSFER_ENCODING,
    ] {
        headers.remove(name);
    }
}

fn sanitize_redirect_headers(headers: &mut HeaderMap, previous: &Url, next: &Url) {
    headers.remove(HOST);
    if same_origin(previous, next) {
        return;
    }

    for name in [
        AUTHORIZATION,
        COOKIE,
        PROXY_AUTHORIZATION,
        REFERER,
        WWW_AUTHENTICATE,
    ] {
        headers.remove(name);
    }
    headers.remove("cookie2");
    headers.remove("x-jms-org");
}

fn redirect_location_has_userinfo(location: &str) -> bool {
    let authority = location.strip_prefix("//").or_else(|| {
        location
            .find(':')
            .and_then(|scheme_end| location[scheme_end + 1..].strip_prefix("//"))
    });
    authority
        .and_then(|value| value.split(['/', '?', '#']).next())
        .is_some_and(|value| value.contains('@'))
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

#[derive(Clone)]
pub(crate) struct OAuthProxyClient {
    request_client: Client,
    proxy_manager: ProxyManager,
}

impl OAuthProxyClient {
    pub(crate) fn new(proxy_manager: &ProxyManager) -> Result<Self> {
        Ok(Self {
            request_client: request_builder_client()?,
            proxy_manager: proxy_manager.clone(),
        })
    }

    pub(crate) async fn get(&self, url: impl reqwest::IntoUrl) -> Result<Response> {
        let request = self.request_client.get(url).build()?;
        execute_api_request(&self.proxy_manager, request).await
    }
}

#[derive(Debug)]
pub(crate) struct OAuthProxyClientError(anyhow::Error);

impl std::fmt::Display for OAuthProxyClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for OAuthProxyClientError {}

impl From<anyhow::Error> for OAuthProxyClientError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl<'c> AsyncHttpClient<'c> for OAuthProxyClient {
    type Error = OAuthProxyClientError;
    type Future = Pin<
        Box<dyn Future<Output = std::result::Result<HttpResponse, Self::Error>> + Send + Sync + 'c>,
    >;

    fn call(&'c self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let request: Request = request
                .try_into()
                .map_err(|error: reqwest::Error| OAuthProxyClientError(error.into()))?;
            let response = execute_proxy_request(&self.proxy_manager, request, true, None)
                .await
                .map_err(OAuthProxyClientError)?
                .response;
            let mut builder = oauth2::http::Response::builder()
                .status(response.status())
                .version(response.version());
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            let body = response
                .bytes()
                .await
                .map_err(|error| OAuthProxyClientError(error.without_url().into()))?
                .to_vec();
            builder
                .body(body)
                .map_err(|error| OAuthProxyClientError(error.into()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::proxy::{ProxyMode, ProxyPreferredMode, ProxySettingsInput, ProxyType};
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::thread;

    fn manual_proxy_manager(address: SocketAddr) -> (tempfile::TempDir, ProxyManager) {
        let directory = tempfile::tempdir().unwrap();
        let manager = ProxyManager::for_test(
            directory.path().join("proxy.json"),
            ProxySettingsInput {
                mode: ProxyMode::Manual,
                preferred_mode: ProxyPreferredMode::Manual,
                proxy_type: ProxyType::Http,
                host: address.ip().to_string(),
                port: Some(address.port()),
                ..ProxySettingsInput::default()
            },
        )
        .unwrap();
        (directory, manager)
    }

    fn read_request_and_respond(mut stream: TcpStream, response: &[u8]) -> String {
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
    }

    #[test]
    fn oauth_form_post_remains_cloneable_after_reqwest_conversion() {
        let oauth_request = oauth2::http::Request::builder()
            .method(oauth2::http::Method::POST)
            .uri("https://example.com/api/v1/authentication/oauth2/token/")
            .header(
                oauth2::http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(b"grant_type=refresh_token&refresh_token=secret".to_vec())
            .unwrap();
        let request: Request = oauth_request.try_into().unwrap();
        let cloned = request
            .try_clone()
            .expect("OAuth form body must be replayable");

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            cloned.body().and_then(reqwest::Body::as_bytes),
            Some(b"grant_type=refresh_token&refresh_token=secret".as_slice())
        );
    }

    #[test]
    fn redirect_method_and_body_rules_match_reqwest() {
        let previous_url = Url::parse("http://first.invalid/submit").unwrap();
        let next_url = Url::parse("http://first.invalid/result").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(CONTENT_LENGTH, "2".parse().unwrap());

        let post_302 = prepare_redirect_request(
            None,
            Method::POST,
            previous_url.clone(),
            headers.clone(),
            None,
            Version::HTTP_11,
            StatusCode::FOUND,
            next_url.clone(),
        )
        .expect("302 POST must be replayable without its original body");
        assert_eq!(post_302.method(), Method::GET);
        assert!(post_302.body().is_none());
        assert!(!post_302.headers().contains_key(CONTENT_TYPE));
        assert!(!post_302.headers().contains_key(CONTENT_LENGTH));

        let head_303 = prepare_redirect_request(
            None,
            Method::HEAD,
            previous_url.clone(),
            headers,
            None,
            Version::HTTP_11,
            StatusCode::SEE_OTHER,
            next_url.clone(),
        )
        .expect("303 HEAD must be replayable without its original body");
        assert_eq!(head_303.method(), Method::HEAD);
        assert!(head_303.body().is_none());

        for status in [
            StatusCode::TEMPORARY_REDIRECT,
            StatusCode::PERMANENT_REDIRECT,
        ] {
            assert!(prepare_redirect_request(
                None,
                Method::POST,
                previous_url.clone(),
                HeaderMap::new(),
                None,
                Version::HTTP_11,
                status,
                next_url.clone(),
            )
            .is_none());
        }
    }

    #[test]
    fn redirect_userinfo_check_ignores_at_signs_outside_authority() {
        assert!(redirect_location_has_userinfo("https://@host.invalid/x"));
        assert!(redirect_location_has_userinfo("//@host.invalid/x"));
        assert!(redirect_location_has_userinfo(
            "https://user@host.invalid/x"
        ));
        assert!(!redirect_location_has_userinfo("/users/@me"));
        assert!(!redirect_location_has_userinfo("?email=user@example.com"));
    }

    #[tokio::test]
    async fn api_redirect_re_resolves_http_to_https_target() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (_directory, manager) = manual_proxy_manager(listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let first = read_request_and_respond(
                listener.accept().unwrap().0,
                b"HTTP/1.1 302 Found\r\nLocation: https://secure.invalid/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            let second = read_request_and_respond(
                listener.accept().unwrap().0,
                b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            (first, second)
        });
        let request = request_builder_client()
            .unwrap()
            .get("http://plain.invalid/start")
            .build()
            .unwrap();

        match execute_api_request(&manager, request).await {
            Ok(_) => panic!("HTTPS proxy CONNECT unexpectedly succeeded"),
            Err(_) => {}
        }
        let (first, second) = server.join().unwrap();
        assert!(first.starts_with("GET http://plain.invalid/start HTTP/1.1\r\n"));
        assert!(second.starts_with("CONNECT secure.invalid:443 HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn api_redirect_strips_sensitive_headers_across_hosts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (_directory, manager) = manual_proxy_manager(listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let first = read_request_and_respond(
                listener.accept().unwrap().0,
                b"HTTP/1.1 302 Found\r\nLocation: http://second.invalid/final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            let second = read_request_and_respond(
                listener.accept().unwrap().0,
                b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            (first, second)
        });
        let request = request_builder_client()
            .unwrap()
            .get("http://first.invalid/start")
            .header(AUTHORIZATION, "Bearer access-secret")
            .header(COOKIE, "session=session-secret")
            .header(PROXY_AUTHORIZATION, "Basic proxy-secret")
            .header(REFERER, "http://first.invalid/private/path")
            .header("X-JMS-ORG", "tenant-secret")
            .build()
            .unwrap();

        let response = execute_api_request(&manager, request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let (first, second) = server.join().unwrap();
        let first = first.to_ascii_lowercase();
        let second = second.to_ascii_lowercase();
        assert!(first.contains("\r\nauthorization: bearer access-secret\r\n"));
        assert!(first.contains("\r\ncookie: session=session-secret\r\n"));
        assert!(first.contains("\r\nproxy-authorization: basic proxy-secret\r\n"));
        assert!(first.contains("\r\nreferer: http://first.invalid/private/path\r\n"));
        assert!(first.contains("\r\nx-jms-org: tenant-secret\r\n"));
        assert!(second.starts_with("get http://second.invalid/final http/1.1\r\n"));
        assert!(!second.contains("\r\nauthorization:"));
        assert!(!second.contains("\r\ncookie:"));
        assert!(!second.contains("\r\nproxy-authorization:"));
        assert!(!second.contains("\r\nreferer:"));
        assert!(!second.contains("\r\nx-jms-org:"));
    }

    #[tokio::test]
    async fn api_redirect_stops_after_ten_relative_location_hops() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let (_directory, manager) = manual_proxy_manager(listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let mut requests = Vec::new();
            for _ in 0..=MAX_REDIRECTS {
                requests.push(read_request_and_respond(
                    listener.accept().unwrap().0,
                    b"HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                ));
            }
            requests
        });
        let request = request_builder_client()
            .unwrap()
            .get("http://loop.invalid/start")
            .build()
            .unwrap();

        let error = match execute_api_request(&manager, request).await {
            Ok(_) => panic!("redirect loop unexpectedly succeeded"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("redirect limit exceeded"));
        let requests = server.join().unwrap();
        assert_eq!(requests.len(), MAX_REDIRECTS + 1);
        assert!(requests[1].starts_with("GET http://loop.invalid/loop HTTP/1.1\r\n"));
    }
}
