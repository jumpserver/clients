use oauth2::{
    basic::BasicClient, reqwest, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::sync::Mutex;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::oneshot;
use url::Url;

/// 记录一次登录发起时的上下文（PKCE/CSRF 和回调通道）。
pub struct PendingAuth {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf: CsrfToken,
    pub tx: oneshot::Sender<CallbackParams>,
}

/// 全局保存正在进行的登录流程，等待 deep link 回调。
#[derive(Default)]
pub struct AuthFlowState {
    pub pending: Mutex<Option<PendingAuth>>,
}

/// deep link 回调携带的参数。
pub struct CallbackParams {
    pub code: AuthorizationCode,
    pub state: Option<String>,
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf: CsrfToken,
}

#[tauri::command]
pub async fn auth_login(
    app: AppHandle,
    flow_state: State<'_, AuthFlowState>,
    site: String,
) -> Result<(), String> {
    let fut = async {
        let client = BasicClient::new(ClientId::new(String::from("client-id")))
            .set_client_secret(ClientSecret::new(String::from("sectret")))
            // 指定授权端点：用户会被重定向到这个 URL 登录/授权
            // 会自动拼上 response_type、client_id、redirect_uri、scope、state、code_challenge 等参数
            .set_auth_uri(AuthUrl::new(format!("{}/core/o/authorize/", site))?)
            // 指定令牌端点：将 code + pkce_verifier 或者 refresh_token
            // 向这个 URL 发 POST 来换取/刷新 access_token、id_token 等
            .set_token_uri(TokenUrl::new(String::from(""))?)
            .set_redirect_uri(RedirectUrl::new(String::from("jms://oauth2/callback"))?);

        // 生成 PKCE + 授权 URL
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(String::from("write")))
            .add_scope(Scope::new(String::from("read")))
            .set_pkce_challenge(pkce_challenge)
            .url();

        // 保存这次发起的登录上下文，并挂起等待 deep link 回调。
        let (tx, rx) = oneshot::channel();
        {
            let mut guard = flow_state.pending.lock().expect("lock poisoned");
            *guard = Some(PendingAuth {
                pkce_verifier,
                csrf: csrf_token.clone(),
                tx,
            });
        }

        log::info!("Browse to: {}", auth_url);
        let _ = app.opener().open_url(auth_url, None::<&str>);

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client no build");

        // 等待 deep link 回调传回 code/state
        let callback = rx
            .await
            .map_err(|_| anyhow::anyhow!("auth flow cancelled or timed out"))?;

        // 校验 state，防止 CSRF
        if let Some(state) = callback.state.as_ref() {
            if state != callback.csrf.secret() {
                anyhow::bail!("state mismatch");
            }
        }

        let token_result = client
            .exchange_code(callback.code)
            .set_pkce_verifier(callback.pkce_verifier)
            .request_async(&http_client)
            .await?;

        // 后续刷新，取 refresh_token
        if let Some(refresh) = token_result.refresh_token() {
            let _refreshed = client
                .exchange_refresh_token(refresh)
                .request_async(&http_client)
                .await?;
        }

        Ok::<(), anyhow::Error>(())
    };

    fut.await.map_err(|e| e.to_string())
}

/// deep link on_open_url 解析到 code/state 后调用，把数据喂回正在等待的 auth_login。
pub fn handle_oauth_callback(flow_state: &State<'_, AuthFlowState>, raw_url: &str) {
    if let Ok(url) = Url::parse(raw_url) {
        let mut code = None;
        let mut state = None;
        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "code" => code = Some(v.to_string()),
                "state" => state = Some(v.to_string()),
                _ => {}
            }
        }
        if let Some(code) = code {
            if let Ok(mut guard) = flow_state.pending.lock() {
                if let Some(pending) = guard.take() {
                    let _ = pending.tx.send(CallbackParams {
                        code: AuthorizationCode::new(code),
                        state,
                        pkce_verifier: pending.pkce_verifier,
                        csrf: pending.csrf,
                    });
                }
            }
        }
    }
}
