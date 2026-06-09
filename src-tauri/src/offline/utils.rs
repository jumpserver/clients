use crate::offline::recording::{RecordingEntry, RecordingKind, RecordingMetadata};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;
use std::{fs, io};

const LOCAL_RECORDINGS_BASE_URL: &str = "http://127.0.0.1:14876/recordings";

pub fn file_size(path: &Path) -> anyhow::Result<u64> {
    Ok(fs::metadata(path)
        .with_context(|| format!("read file metadata failed: {:?}", path))?
        .len())
}

/// 根据 gzip 文件名判断录像类型
pub fn detect_gz_kind(file_name: &str) -> anyhow::Result<RecordingKind> {
    if file_name.ends_with(".cast.gz") {
        return Ok(RecordingKind::Cast);
    }

    if file_name.ends_with(".replay.gz") {
        return Ok(RecordingKind::Gua);
    }

    if file_name.ends_with(".part.gz") {
        return Ok(RecordingKind::Part);
    }

    bail!("unsupported gzip recording file: {}", file_name);
}

/// 移除 .gz 后缀
pub fn strip_gz_suffix(file_name: &str) -> String {
    file_name
        .strip_suffix(".gz")
        .unwrap_or(file_name)
        .to_string()
}

/// 移除常见录像文件后缀，用作显示名称
pub fn strip_known_suffix(file_name: &str) -> &str {
    file_name
        .strip_suffix(".replay")
        .or_else(|| file_name.strip_suffix(".part"))
        .or_else(|| file_name.strip_suffix(".cast"))
        .or_else(|| file_name.strip_suffix(".mp4"))
        .unwrap_or(file_name)
}

/// 生成前端播放地址
pub fn playable_url(id: &str) -> String {
    format!("{}/{}/content", LOCAL_RECORDINGS_BASE_URL, id)
}

/// 当前时间字符串
pub fn now_string() -> String {
    Utc::now().to_rfc3339()
}

/// 解压 gzip 文件到目标路径
pub fn gunzip_to_file(source: &Path, target: &Path) -> anyhow::Result<()> {
    let source_file =
        fs::File::open(source).with_context(|| format!("open gzip file failed: {:?}", source))?;

    let mut decoder = GzDecoder::new(source_file);
    let mut target_file = fs::File::create(target)
        .with_context(|| format!("create gunzip target failed: {:?}", target))?;

    io::copy(&mut decoder, &mut target_file)
        .with_context(|| format!("gunzip recording failed: {:?} -> {:?}", source, target))?;

    Ok(())
}

/// 只取 archive entry 的最后一段文件名，避免路径穿越
pub fn safe_archive_file_name(path: &Path) -> Option<String> {
    path.file_name().and_then(OsStr::to_str).map(str::to_string)
}

/// 判断是否是录像元信息文件
pub fn is_metadata_file(file_name: &str) -> bool {
    file_name.ends_with(".json")
}

/// 根据 archive 内部文件名判断录像类型
pub fn detect_archive_recording_kind(file_name: &str) -> Option<RecordingKind> {
    if file_name.ends_with(".mp4") {
        return Some(RecordingKind::Mp4);
    }

    if file_name.ends_with(".cast") || file_name.ends_with(".cast.gz") {
        return Some(RecordingKind::Cast);
    }

    if file_name.ends_with(".replay") || file_name.ends_with(".replay.gz") {
        return Some(RecordingKind::Gua);
    }

    if file_name.ends_with(".part") || file_name.ends_with(".part.gz") {
        return Some(RecordingKind::Part);
    }

    None
}

/// 读取录像元信息 JSON
pub fn read_metadata<R>(reader: &mut R) -> Result<RecordingMetadata>
where
    R: Read,
{
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .context("read recording metadata failed")?;

    let value: serde_json::Value =
        serde_json::from_str(&text).context("parse recording metadata failed")?;

    Ok(RecordingMetadata {
        user: string_field(&value, "user"),
        asset: string_field(&value, "asset"),
        protocol: string_field(&value, "protocol"),
        date_start: string_field(&value, "date_start"),
        date_end: string_field(&value, "date_end"),
        duration: string_field(&value, "duration"),
        command_amount: value.get("command_amount").and_then(|value| value.as_u64()),
    })
}

/// 从 JSON 中读取字符串字段
pub fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

/// 把 reader 内容直接写入文件
pub fn copy_reader_to_file<R>(reader: &mut R, target: &Path) -> Result<()>
where
    R: Read,
{
    let mut target_file = fs::File::create(target)
        .with_context(|| format!("create target file failed: {:?}", target))?;

    io::copy(reader, &mut target_file)
        .with_context(|| format!("copy archive entry failed: {:?}", target))?;

    Ok(())
}

/// 把 reader 里的 gzip 内容解压到文件
pub fn gunzip_reader_to_file<R>(reader: &mut R, target: &Path) -> Result<()>
where
    R: Read,
{
    let mut decoder = GzDecoder::new(reader);

    let mut target_file = fs::File::create(target)
        .with_context(|| format!("create gunzip target failed: {:?}", target))?;

    io::copy(&mut decoder, &mut target_file)
        .with_context(|| format!("gunzip archive entry failed: {:?}", target))?;

    Ok(())
}

/// 确保压缩包里至少解析出一个可播放录像
pub fn ensure_entries_not_empty(entries: Vec<RecordingEntry>) -> Result<Vec<RecordingEntry>> {
    if entries.is_empty() {
        bail!("archive does not contain supported recording files");
    }

    Ok(entries)
}
