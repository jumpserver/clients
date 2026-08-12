use crate::{
    api::{
        client::{execute_api_request, request_builder_client},
        context::{apply_org_header, ApiContext},
        response::into_api_response,
    },
    utils::tz_offset_string,
};
use anyhow::Result;
use log::info;
use reqwest::{header::AUTHORIZATION, Client, Method, RequestBuilder, Response};
use serde::Serialize;
use url::Url;

pub(crate) use crate::api::response::ApiResponse;
use crate::api::session::ApiSessionContext;
use crate::service::proxy::ProxyManager;

pub struct ApiRequestClient {
    client: Client,
    proxy_manager: ProxyManager,
    origin: String,
    bearer_token: String,
    org_id: String,
}

impl ApiRequestClient {
    /// 创建绑定站点 origin 的 API 客户端，后续 endpoint 可以直接拼成完整 URL
    pub fn with_origin(
        origin: String,
        bearer_token: String,
        org_id: String,
        proxy_manager: &ProxyManager,
    ) -> Result<Self> {
        Ok(Self {
            client: request_builder_client()?,
            proxy_manager: proxy_manager.clone(),
            origin,
            bearer_token,
            org_id,
        })
    }

    /// 根据当前 API 会话上下文创建请求客户端
    pub fn from_session(context: &ApiSessionContext, proxy_manager: &ProxyManager) -> Result<Self> {
        Self::with_origin(
            context.origin.clone(),
            context.bearer_token.clone(),
            context.org_id.clone(),
            proxy_manager,
        )
    }

    /// 将集中定义的 API path 拼接为当前站点下的完整 URL
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.origin.trim_end_matches('/'), path)
    }

    /// 发送 GET 请求并转换为统一 ApiResponse
    pub async fn get_with_response(&self, url: &str) -> ApiResponse {
        log_request("GET", url);

        self.send_with_response(Method::GET, url, |request| request)
            .await
    }

    /// 发送带 query 参数的 GET 请求并转换为统一 ApiResponse
    pub async fn get_with_query_response<T>(&self, url: &str, query: &T) -> ApiResponse
    where
        T: Serialize + ?Sized,
    {
        log_request("GET WITH QUERY", url);
        self.send_with_response(Method::GET, url, |request| request.query(query))
            .await
    }

    /// 发送 JSON POST 请求并转换为统一 ApiResponse
    pub async fn post_json_with_response<T>(&self, url: &str, body: &T) -> ApiResponse
    where
        T: Serialize + ?Sized,
    {
        log_request("POST WITH BODY", url);

        self.send_with_response(Method::POST, url, |request| request.json(body))
            .await
    }

    /// 发送 DELETE 请求并转换为统一 ApiResponse
    pub async fn delete_with_response(&self, url: &str) -> ApiResponse {
        log_request("DELETE", url);
        self.send_with_response(Method::DELETE, url, |request| request)
            .await
    }

    /// 构建并执行底层 reqwest 请求
    async fn send<F>(&self, method: Method, url: &str, apply: F) -> anyhow::Result<Response>
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        let request = apply(self.base_request(&self.client, method, url)).build()?;
        execute_api_request(&self.proxy_manager, request).await
    }

    /// 把客户端内部保存的 Token 和组织信息转换为请求上下文
    fn context(&self) -> ApiContext<'_> {
        ApiContext {
            bearer_token: &self.bearer_token,
            org_id: &self.org_id,
        }
    }

    /// 创建带有公共 header 的基础请求
    fn base_request(&self, client: &Client, method: Method, url: &str) -> RequestBuilder {
        let context = self.context();
        let mut request = client
            .request(method, url)
            .header("X-TZ", tz_offset_string());

        if !context.bearer_token.is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {}", context.bearer_token));
        }

        if let Some(referer) = referer_from(url) {
            request = request.header("Referer", referer);
        }

        if context.org_id.is_empty() {
            request
        } else {
            apply_org_header(request, &context)
        }
    }

    /// 执行请求并转换为统一响应结构
    async fn send_with_response<F>(&self, method: Method, url: &str, apply: F) -> ApiResponse
    where
        F: FnOnce(RequestBuilder) -> RequestBuilder,
    {
        into_api_response(url, self.send(method, url, apply).await).await
    }
}

/// 从 URL 中提取 Referer 头部值，确保仅包含协议、主机和端口
fn referer_from(url: &str) -> Option<String> {
    Url::parse(url).ok().and_then(|url| match url.scheme() {
        "http" | "https" => {
            let host = url.host_str()?;
            let mut origin = format!("{}://{}", url.scheme(), host);

            if let Some(port) = url.port() {
                origin.push(':');
                origin.push_str(&port.to_string());
            }

            Some(origin)
        }
        _ => None,
    })
}

fn log_request(operation: &str, url: &str) {
    match referer_from(url) {
        Some(origin) => info!("{} {}", operation, origin),
        None => info!("{} request", operation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::proxy::{ProxyPreferredMode, ProxySettingsInput, ProxyType};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;

    #[test]
    fn request_url_contains_query_before_per_request_proxy_resolution() {
        let client = Client::builder().no_proxy().build().unwrap();
        let request = client
            .get("http://example.com/api/assets")
            .query(&[("offset", "10"), ("search", "db server")])
            .build()
            .unwrap();
        assert_eq!(
            request.url().as_str(),
            "http://example.com/api/assets?offset=10&search=db+server"
        );
    }

    #[tokio::test]
    async fn existing_api_client_uses_proxy_settings_updated_after_construction() {
        let directory = tempfile::tempdir().unwrap();
        let manager = ProxyManager::for_test(
            directory.path().join("proxy.json"),
            ProxySettingsInput::default(),
        )
        .unwrap();
        let client = ApiRequestClient::with_origin(
            "http://upstream.invalid".to_string(),
            String::new(),
            String::new(),
            &manager,
        )
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
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
            stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
            String::from_utf8(request).unwrap()
        });

        manager
            .update(ProxySettingsInput {
                mode: crate::service::proxy::ProxyMode::Manual,
                preferred_mode: ProxyPreferredMode::Manual,
                proxy_type: ProxyType::Http,
                host: proxy.ip().to_string(),
                port: Some(proxy.port()),
                ..ProxySettingsInput::default()
            })
            .await
            .unwrap();

        let response = client
            .send(Method::GET, "http://upstream.invalid/api", |request| {
                request
            })
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
        assert!(server
            .join()
            .unwrap()
            .starts_with("GET http://upstream.invalid/api HTTP/1.1\r\n"));
    }
}
