use crate::commands::requests::{get_unified, get_with_response, ApiResponse};
use crate::utils::to_api_response;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    #[default]
    Linux,
    Windows,
    #[serde(rename = "windows_ad")]
    WindowsAd,
    Unix,
    Other,
    Database,
    Device,
    Web,
}

#[derive(Debug, Deserialize)]
struct AssetListResponse {
    count: usize,
    next: Option<String>,
    previous: Option<String>,
    results: Vec<Value>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AssetQuery {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Category>,

    #[serde(rename = "category", skip_serializing_if = "Option::is_none")]
    pub category: Option<Category>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,

    pub oid: String,
}

impl AssetQuery {
    #[allow(dead_code)]
    pub fn new(asset_type: Category, org: String) -> Self {
        let (r#type, category) = match asset_type {
            Category::Database | Category::Device => (None, Some(asset_type)),
            Category::Web => (None, Some(asset_type)),
            Category::Linux
            | Category::Windows
            | Category::WindowsAd
            | Category::Unix
            | Category::Other => (Some(asset_type), None),
        };

        Self {
            r#type,
            category,
            offset: None,
            limit: None,
            search: None,
            order: None,
            oid: org,
        }
    }

    pub fn get_category(&self) -> Category {
        self.category.or(self.r#type).unwrap_or_default()
    }
}

pub trait HasOrg {
    fn org(&self) -> &str;
}

impl HasOrg for AssetQuery {
    fn org(&self) -> &str {
        &self.oid
    }
}

pub struct AssetService {
    origin: String,
    bearer_token: String,
    query: AssetQuery,
}

impl AssetService {
    pub fn new(origin: String, bearer_token: String, query: AssetQuery) -> Self {
        Self {
            origin,
            bearer_token,
            query,
        }
    }

    pub async fn get_category_assets(&self, favorite: bool) -> ApiResponse {
        let url = if favorite {
            format!(
                "{}/api/v1/perms/users/self/nodes/favorite/assets/",
                self.origin
            )
        } else {
            format!("{}/api/v1/perms/users/self/assets/", self.origin)
        };

        info!(
            "获取类型为：{:?} 的资产信息，请求 url: {}, oid: {}",
            self.query.get_category(),
            url,
            self.query.oid
        );
        info!("Bearer: {}", self.bearer_token);
        info!("query: {:?}", self.query);

        if favorite {
            let query = AssetQuery {
                r#type: None,
                category: None,
                offset: Some(self.query.offset.unwrap_or(0)),
                limit: Some(self.query.limit.unwrap_or(20)),
                search: Some(self.query.search.clone().unwrap_or_default()),
                order: Some(self.query.order.clone().unwrap_or_default()),
                oid: self.query.oid.clone(),
            };

            return to_api_response(&url, get_unified(&url, &self.bearer_token, &query).await)
                .await;
        }

        let queries = Self::expand_queries(&self.query);

        if queries.len() == 1 {
            return to_api_response(
                &url,
                get_unified(&url, &self.bearer_token, &queries[0]).await,
            )
            .await;
        }

        self.get_combined_category_assets(&url, &queries).await
    }

    pub async fn get_favorite_assets(&self) -> ApiResponse {
        let url = format!("{}/api/v1/assets/favorite-assets/", &self.origin);

        get_with_response(&url, &self.bearer_token).await
    }

    fn expand_queries(query: &AssetQuery) -> Vec<AssetQuery> {
        let offset = Some(query.offset.unwrap_or(0));
        let limit = Some(query.limit.unwrap_or(20));
        let search = Some(query.search.clone().unwrap_or_default());
        let order = Some(query.order.clone().unwrap_or_default());
        let oid = query.oid.clone();

        let exact = |r#type: Option<Category>, category: Option<Category>| AssetQuery {
            r#type,
            category,
            offset,
            limit,
            search: search.clone(),
            order: order.clone(),
            oid: oid.clone(),
        };

        match query.get_category() {
            Category::Linux => vec![exact(Some(Category::Linux), None)],
            Category::Windows => vec![
                exact(Some(Category::Windows), None),
                exact(Some(Category::WindowsAd), None),
            ],
            Category::WindowsAd => vec![exact(Some(Category::WindowsAd), None)],
            Category::Unix => vec![exact(Some(Category::Unix), None)],
            Category::Other => vec![
                exact(Some(Category::Unix), None),
                exact(Some(Category::Other), None),
            ],
            Category::Database => vec![exact(None, Some(Category::Database))],
            Category::Device => vec![exact(None, Some(Category::Device))],
            Category::Web => vec![exact(None, Some(Category::Web))],
        }
    }

    async fn get_combined_category_assets(&self, url: &str, queries: &[AssetQuery]) -> ApiResponse {
        let mut combined: Vec<Value> = Vec::new();
        let mut seen_ids = HashSet::new();

        for sub_query in queries {
            let response = self.fetch_all_assets(url, sub_query).await;
            let list = match response {
                Ok(list) => list,
                Err(err) => return err,
            };

            for item in list {
                let asset_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();

                if asset_id.is_empty() || seen_ids.insert(asset_id) {
                    combined.push(item);
                }
            }
        }

        Self::sort_assets(&mut combined, self.query.order.as_deref());

        let total = combined.len();
        let offset = self.query.offset.unwrap_or(0) as usize;
        let limit = self.query.limit.unwrap_or(20) as usize;
        let results = combined
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        ApiResponse {
            status: 200,
            data: json!({
                "count": total,
                "next": Value::Null,
                "previous": Value::Null,
                "results": results,
            })
            .to_string(),
            success: true,
        }
    }

    async fn fetch_all_assets(
        &self,
        url: &str,
        query: &AssetQuery,
    ) -> Result<Vec<Value>, ApiResponse> {
        const BATCH_LIMIT: u32 = 200;

        let mut offset = 0;
        let mut all_results = Vec::new();

        loop {
            let page_query = AssetQuery {
                r#type: query.r#type,
                category: query.category,
                offset: Some(offset),
                limit: Some(BATCH_LIMIT),
                search: Some(query.search.clone().unwrap_or_default()),
                order: Some(query.order.clone().unwrap_or_default()),
                oid: query.oid.clone(),
            };

            let response =
                to_api_response(url, get_unified(url, &self.bearer_token, &page_query).await).await;
            if !response.success {
                return Err(response);
            }

            let payload: AssetListResponse = match serde_json::from_str(&response.data) {
                Ok(payload) => payload,
                Err(error) => {
                    return Err(ApiResponse {
                        status: 500,
                        data: format!("parse asset list response failed: {}", error),
                        success: false,
                    });
                }
            };

            let AssetListResponse {
                count,
                next: _next,
                previous: _previous,
                results,
            } = payload;

            let page_size = results.len();
            all_results.extend(results);

            if page_size == 0 || all_results.len() >= count {
                break;
            }

            offset += page_size as u32;
        }

        Ok(all_results)
    }

    fn sort_assets(results: &mut [Value], order: Option<&str>) {
        let normalized = order
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("name");

        let descending = normalized.starts_with('-');
        let field = normalized.trim_start_matches('-');

        results.sort_by(|left, right| {
            let ordering = Self::compare_asset_field(left, right, field);
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    fn compare_asset_field(left: &Value, right: &Value, field: &str) -> Ordering {
        let left_value = Self::extract_asset_field(left, field);
        let right_value = Self::extract_asset_field(right, field);

        left_value.cmp(&right_value).then_with(|| {
            Self::extract_asset_field(left, "id").cmp(&Self::extract_asset_field(right, "id"))
        })
    }

    fn extract_asset_field(item: &Value, field: &str) -> String {
        item.get(field)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_lowercase()
    }
}
