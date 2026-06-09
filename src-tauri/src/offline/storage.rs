/*
负责：
    - 离线录像缓存根目录
    - 为每个录像创建独立目录
    - 复制 / 删除缓存文件
    - 清空所有离线缓存
    - 校验 recording_id，避免路径穿越
    - 校验文件名，避免 ../xxx
*/
use crate::offline::recording::RecordingId;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 离线录像的本地缓存管理器
///
/// 这个结构只负责文件系统路径和缓存目录，不负责解析录像内容。
#[derive(Debug, Clone)]
pub struct OfflineStorage {
    root_dir: PathBuf,
}

impl OfflineStorage {
    pub fn new(app_data_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: app_data_dir.into().join("offline").join("recordings"),
        }
    }

    /// 返回离线录像缓存根目录
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// 确保缓存根目录存在
    pub fn ensure_ready(&self) -> Result<()> {
        fs::create_dir_all(&self.root_dir)
            .with_context(|| format!("create offline storage dir failed: {:?}", self.root_dir))?;

        Ok(())
    }

    /// 返回某个录像的缓存目录
    pub fn recording_dir(&self, id: &RecordingId) -> Result<PathBuf> {
        Self::validate_recording_id(id)?;

        Ok(self.root_dir.join(id))
    }

    /// 创建某个录像的缓存目录
    pub fn create_recording_dir(&self, id: &RecordingId) -> Result<PathBuf> {
        let dir = self.recording_dir(id)?;

        fs::create_dir_all(&dir)
            .with_context(|| format!("create recording dir failed: {:?}", dir))?;

        Ok(dir)
    }

    /// 返回某个录像目录下的安全文件路径
    pub fn recording_file_path(&self, id: &RecordingId, file_name: &str) -> Result<PathBuf> {
        Self::validate_file_name(file_name)?;

        Ok(self.recording_dir(id)?.join(file_name))
    }

    /// 将外部文件复制到某个录像缓存目录
    pub fn copy_into_recording(
        &self,
        source: impl AsRef<Path>,
        id: &RecordingId,
        file_name: &str,
    ) -> Result<PathBuf> {
        self.create_recording_dir(id)?;

        let target = self.recording_file_path(id, file_name)?;
        let source = source.as_ref();

        fs::copy(source, &target)
            .with_context(|| format!("copy recording file failed: {:?} -> {:?}", source, target))?;

        Ok(target)
    }

    /// 删除某个录像的整个缓存目录
    pub fn remove_recording(&self, id: &RecordingId) -> Result<()> {
        let dir = self.recording_dir(id)?;

        if dir.exists() {
            fs::remove_dir_all(&dir)
                .with_context(|| format!("remove recording dir failed: {:?}", dir))?;
        }

        Ok(())
    }

    /// 清空所有离线录像缓存
    pub fn clear_all(&self) -> Result<()> {
        if self.root_dir.exists() {
            fs::remove_dir_all(&self.root_dir)
                .with_context(|| format!("clear offline storage failed: {:?}", self.root_dir))?;
        }

        self.ensure_ready()
    }

    /// 校验录像 ID，避免路径穿越
    fn validate_recording_id(id: &str) -> Result<()> {
        if id.is_empty() {
            bail!("recording is empty");
        }

        if id.len() > 128 {
            bail!("recording id too long");
        }

        // 检查 id 中每一个字符是否是 ASCII 字符或 _ 或 -。只要有一个字符不满足，valid 就是 false
        let valid = id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');

        if !valid {
            bail!("recording id contains invalid characters");
        }

        Ok(())
    }

    /// 校验缓存文件名，避免 ../ 这类路径穿越
    fn validate_file_name(file_name: &str) -> Result<()> {
        if file_name.is_empty() {
            bail!("file name is empty");
        }

        if file_name == "." || file_name == ".." {
            bail!("invalid file name");
        }

        if file_name.contains("/") || file_name.contains("\\") {
            bail!("file name must not contain path separators");
        }

        Ok(())
    }
}

/// 生成离线录像 ID
///
/// 使用进程 ID + 当前时间纳秒，足够用于本地缓存目录
pub fn new_recording_id() -> RecordingId {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();

    format!("rec-{}-{}", std::process::id(), nanos)
}
