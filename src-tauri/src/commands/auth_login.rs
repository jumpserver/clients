use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AuthenticationFlow, ClientId, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge, RedirectUrl,
    Scope,
};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

// 客户端生成授权链接，用系统浏览器打开
// 用户登录成功后，IdP 把 code 重定向回本地的回调地址
// 本地服务接收 code，校验 state/nonce，再用 code + pkce_verifier 去换 token 并验证 ID Token
#[tauri::command]
pub async fn auth_login(app: AppHandle, site: String) -> anyhow::Result<()> {
    let http_client = reqwest::Client::new();
    let issuer = IssuerUrl::new(site.trim().to_string())?;

    // 获取 OIDC 配置
    let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client).await?;

    // 客户端
    let callback_url = RedirectUrl::new(String::from("http://localhost:3000"))?;
    let oidc_client = CoreClient::from_provider_metadata(
        provider_metadata,
        // todo
        ClientId::new("CLIENT_ID".into()),
        None,
    )
    .set_redirect_uri(callback_url);

    let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, _csrf, _nonce) = oidc_client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new(String::from("openid")))
        .add_scope(Scope::new(String::from("profile")))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // 打开浏览器
    app.opener().open_url(auth_url, None::<&str>)?;

    Ok(())
}
