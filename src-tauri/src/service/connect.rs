use crate::api::endpoint;
use crate::api::request::{ApiRequestClient, ApiResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenRequestBody {
    pub asset: String,
    pub account: String,
    pub protocol: String,
    pub input_username: String,
    pub input_secret: String,
    pub connect_method: String,
    pub connect_options: Option<Value>,
}

pub struct ConnectService {
    api: ApiRequestClient,
}

impl ConnectService {
    pub fn new(api: ApiRequestClient) -> Self {
        Self { api }
    }

    /// 获取当前组织可用的连接方式
    pub async fn get_connect_methods(&self) -> ApiResponse {
        let url = self.api.endpoint(endpoint::terminal::CONNECT_METHODS);

        self.api.get_with_response(&url).await
    }

    /// 创建资产连接 Token
    pub async fn get_connect_token(&self, body: &TokenRequestBody) -> ApiResponse {
        // 在同一个 API 会话中复核权限，防止旧配置或界面打开后的规则变更绕过限制。
        let methods = self.get_connect_methods().await;
        if !methods.success || methods.status != 200 {
            return connect_method_error(
                "connect_methods_unavailable",
                "Unable to check connection method permissions",
            );
        }
        if !is_connect_method_allowed(&methods.data, &body.protocol, &body.connect_method) {
            return connect_method_error(
                "connect_method_not_allowed",
                "The selected connection method is not allowed",
            );
        }

        let url = self
            .api
            .endpoint(endpoint::authentication::CONNECTION_TOKEN);

        self.api.post_json_with_response(&url, &body).await
    }

    /// 根据连接 Token 获取本地客户端启动 URL
    pub async fn get_local_client_url(
        &self,
        token_id: &str,
        extra_params: Option<&HashMap<String, String>>,
    ) -> ApiResponse {
        let path = endpoint::authentication::client_url(&token_id);
        let mut url = self.api.endpoint(&path);

        if let Some(params) = extra_params {
            if let Ok(mut parsed) = Url::parse(&url) {
                {
                    let mut query = parsed.query_pairs_mut();

                    for (key, value) in params {
                        query.append_pair(key, value);
                    }
                }

                url = parsed.to_string();
            }
        }

        self.api.get_with_response(&url).await
    }
}

fn connect_method_error(code: &str, detail: &str) -> ApiResponse {
    ApiResponse {
        status: 403,
        data: json!({ "code": code, "detail": detail }).to_string(),
        success: false,
    }
}

fn is_connect_method_allowed(data: &str, protocol: &str, method: &str) -> bool {
    if protocol.is_empty() || method.is_empty() {
        return false;
    }
    let Ok(methods) = serde_json::from_str::<Value>(data) else {
        return false;
    };
    methods
        .get(protocol)
        .and_then(Value::as_array)
        .is_some_and(|methods| {
            methods.iter().any(|entry| {
                entry.get("value").and_then(Value::as_str) == Some(method)
                    && match entry.get("disabled") {
                        None => true,
                        Some(disabled) => disabled.as_bool() == Some(false),
                    }
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{HeaderMap, StatusCode},
        routing::{get, post},
        Json, Router,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    struct TestServer {
        service: ConnectService,
        response: Arc<Mutex<(StatusCode, String)>>,
        token_requests: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<()>,
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn test_server(data: &str) -> TestServer {
        let response = Arc::new(Mutex::new((StatusCode::OK, data.to_string())));
        let token_requests = Arc::new(AtomicUsize::new(0));
        let methods_response = response.clone();
        let token_counter = token_requests.clone();
        let app = Router::new()
            .route(
                endpoint::terminal::CONNECT_METHODS,
                get(move |headers: HeaderMap| async move {
                    assert_eq!(headers["authorization"], "Bearer test-user");
                    assert_eq!(headers["x-jms-org"], "test-org");
                    methods_response.lock().unwrap().clone()
                }),
            )
            .route(
                endpoint::authentication::CONNECTION_TOKEN,
                post(
                    move |headers: HeaderMap, Json(body): Json<TokenRequestBody>| async move {
                        assert_eq!(headers["authorization"], "Bearer test-user");
                        assert_eq!(headers["x-jms-org"], "test-org");
                        assert_eq!(body.connect_method, "db_client");
                        token_counter.fetch_add(1, Ordering::SeqCst);
                        (StatusCode::CREATED, Json(json!({ "id": "test-token" })))
                    },
                ),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let api =
            ApiRequestClient::with_origin(origin, "test-user".into(), "test-org".into()).unwrap();
        TestServer {
            service: ConnectService::new(api),
            response,
            token_requests,
            task,
        }
    }

    fn database_request(method: &str) -> TokenRequestBody {
        TokenRequestBody {
            asset: "test-asset".into(),
            account: "test-account".into(),
            protocol: "mariadb".into(),
            input_username: String::new(),
            input_secret: String::new(),
            connect_method: method.into(),
            connect_options: None,
        }
    }

    #[tokio::test]
    async fn denied_or_unavailable_methods_never_create_tokens() {
        let server = test_server("{}").await;
        for (status, data, code) in [
            (
                StatusCode::OK,
                r#"{"mariadb":[]}"#,
                "connect_method_not_allowed",
            ),
            (StatusCode::OK, "invalid JSON", "connect_method_not_allowed"),
            (
                StatusCode::FORBIDDEN,
                "denied",
                "connect_methods_unavailable",
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "connect_methods_unavailable",
            ),
        ] {
            *server.response.lock().unwrap() = (status, data.into());
            for method in ["db_client", "db_guide"] {
                let response = server
                    .service
                    .get_connect_token(&database_request(method))
                    .await;
                assert!(!response.success);
                assert_eq!(response.status, 403);
                assert_eq!(
                    serde_json::from_str::<Value>(&response.data).unwrap()["code"],
                    code
                );
                assert_eq!(server.token_requests.load(Ordering::SeqCst), 0);
            }
        }
    }

    #[tokio::test]
    async fn rechecks_permissions_before_every_token_request() {
        let server = test_server(r#"{"mariadb":[{"value":"db_client"}]}"#).await;
        let request = database_request("db_client");
        let response = server.service.get_connect_token(&request).await;
        assert!(response.success);
        assert_eq!(response.status, 201);
        assert_eq!(server.token_requests.load(Ordering::SeqCst), 1);

        // 规则变更后，即使沿用相同的会话和连接参数也必须拒绝。
        *server.response.lock().unwrap() = (
            StatusCode::OK,
            r#"{"mariadb":[{"value":"db_guide"}]}"#.into(),
        );
        let response = server.service.get_connect_token(&request).await;
        assert!(!response.success);
        assert_eq!(response.status, 403);
        assert_eq!(server.token_requests.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn honors_database_method_restrictions_independently() {
        let client_only = r#"{"mariadb":[{"value":"db_client"}]}"#;
        let guide_only = r#"{"mariadb":[{"value":"db_guide"}]}"#;
        assert!(is_connect_method_allowed(
            client_only,
            "mariadb",
            "db_client"
        ));
        assert!(!is_connect_method_allowed(
            client_only,
            "mariadb",
            "db_guide"
        ));
        assert!(is_connect_method_allowed(guide_only, "mariadb", "db_guide"));
        assert!(!is_connect_method_allowed(
            guide_only,
            "mariadb",
            "db_client"
        ));
        for method in ["db_client", "db_guide"] {
            assert!(!is_connect_method_allowed(
                r#"{"mariadb":[]}"#,
                "mariadb",
                method
            ));
        }
    }

    #[test]
    fn rejects_disabled_missing_and_malformed_methods() {
        for data in [
            r#"{"mariadb":[{"value":"db_client","disabled":true}]}"#,
            r#"{"mariadb":[{"value":"db_client","disabled":"false"}]}"#,
            r#"{"mysql":[{"value":"db_client"}]}"#,
            r#"{"mariadb":null}"#,
            r#"{"mariadb":{}}"#,
            "{}",
            "[]",
            "null",
            "invalid JSON",
        ] {
            assert!(!is_connect_method_allowed(data, "mariadb", "db_client"));
        }
    }

    #[test]
    fn accepts_enabled_methods_for_other_protocols() {
        let data =
            r#"{"ssh":[{"value":"ssh_client","disabled":false}],"http":[{"value":"chrome"}]}"#;
        assert!(is_connect_method_allowed(data, "ssh", "ssh_client"));
        assert!(is_connect_method_allowed(data, "http", "chrome"));
        assert!(!is_connect_method_allowed(data, "", "chrome"));
        assert!(!is_connect_method_allowed(data, "http", ""));
    }
}
