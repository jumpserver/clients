use crate::api::{request::ApiRequestClient, session::ApiSessionStore};
use crate::commands::api_session::fresh_api_context;
use crate::service::connect::ConnectService;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_connect_methods(
    app: AppHandle,
    session: State<'_, ApiSessionStore>,
) -> Result<String, String> {
    let context = fresh_api_context(&app, &session)
        .await
        .map_err(|error| error.to_string())?;
    let api = ApiRequestClient::from_session(&context).map_err(|error| error.to_string())?;
    let response = ConnectService::new(api).get_connect_methods().await;

    if !response.success || response.status != 200 {
        return Err(format!(
            "Failed to fetch connect methods: {}",
            response.status
        ));
    }

    // 直接返回给发起请求的调用方，避免并发请求收到其他会话的全局事件。
    Ok(response.data)
}
