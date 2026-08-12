use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: u16,
    pub data: String,
    pub success: bool,
}

impl ApiResponse {
    pub fn ok(status: u16, data: String) -> Self {
        Self {
            status,
            data,
            success: is_success_status(status),
        }
    }

    pub fn failed(data: String) -> Self {
        Self {
            status: 0,
            data,
            success: false,
        }
    }
}

/// 将响应信息转换成前端统一使用的 API 结构格式
pub async fn into_api_response(
    _url: &str,
    result: anyhow::Result<reqwest::Response>,
) -> ApiResponse {
    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let data = resp.text().await.unwrap_or_default();

            ApiResponse::ok(status, data)
        }
        Err(err) => {
            log::warn!("请求失败: {}", err);
            ApiResponse::failed(format!("请求失败: {}", err))
        }
    }
}

fn is_success_status(status: u16) -> bool {
    matches!(status, 200 | 201 | 204)
}
