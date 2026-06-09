use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 用于隐藏真实文件路径
pub type RecordingId = String;

/// 离线录像的播放类型
///
/// 这个类型决定前端后续应该使用哪一种播放器：
/// - mp4 走普通 video 播放器
/// - cast 走 asciinema 播放器
/// - gua / part 走 Guacamole 录像播放器
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingKind {
    Mp4,
    Cast,
    Gua,
    Part,
}

/// 离线录像的业务元信息
///
/// 这些字段来自录像包里的 json 文件，例如 replay.json。
/// 不是所有录像格式都会带完整信息，所以全部使用 Option。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub user: Option<String>,
    pub asset: Option<String>,
    pub protocol: Option<String>,
    pub date_start: Option<String>,
    pub date_end: Option<String>,
    pub duration: Option<String>,
    pub command_amount: Option<u64>,
}

/// 返回给前端的离线录像清单
///
/// 前端不应该拿到本地真实路径，只通过 playable_url 请求 Rust 本地 HTTP 服务。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingManifest {
    pub id: RecordingId,
    pub name: String,
    pub kind: RecordingKind,
    pub metadata: RecordingMetadata,
    pub playable_url: String,
    pub raw_url: Option<String>,
    pub file_size: u64,
    pub created_at: String,
}

/// Rust 后端内部保存的录像登记信息
///
/// 这个结构可以包含真实本地路径，但不要直接返回给前端。
#[derive(Debug, Clone)]
pub struct RecordingEntry {
    pub manifest: RecordingManifest,
    pub content_path: PathBuf,
    pub working_dir: PathBuf,
}

impl RecordingManifest {
    pub fn new(
        id: RecordingId,
        name: String,
        kind: RecordingKind,
        metadata: RecordingMetadata,
        playable_url: String,
        file_size: u64,
        created_at: String,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            metadata,
            playable_url,
            raw_url: None,
            file_size,
            created_at,
        }
    }

    /// 设置原始文件访问地址
    pub fn with_raw_url(mut self, raw_url: String) -> Self {
        self.raw_url = Some(raw_url);
        self
    }
}
