use crate::offline::recording::{
    RecordingEntry, RecordingKind, RecordingManifest, RecordingMetadata,
};
use crate::offline::storage::{new_recording_id, OfflineStorage};
use crate::offline::utils::{
    copy_reader_to_file, detect_archive_recording_kind, detect_gz_kind, ensure_entries_not_empty,
    file_size, gunzip_reader_to_file, gunzip_to_file, is_metadata_file, now_string, playable_url,
    read_metadata, safe_archive_file_name, strip_gz_suffix, strip_known_suffix,
};
use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

// package.rs 的整体解析流程：
// 1. parse_file 是唯一入口：
//      - 先确认用户传入的是一个存在的本地文件，再根据文件名后缀分流。
//      - 判断顺序很重要： .tar.gz 同时也是 .gz，所以必须先判断 .tar.gz / .tgz，再判断普通 .gz。
// 
// 2. 普通 .mp4 不需要解析包结构，只复制到 OfflineStorage 创建的缓存目录，并生成一个 RecordingEntry。
//
// 3. 普通 .gz 先根据文件名判断类型：
//      - .cast.gz   解压后给 asciinema 播放
//      - .replay.gz 解压后给 Guacamole 播放
//      - .part.gz   解压后按 Guacamole 分片播放
//
//    GzDecoder 的作用就是把 gzip 压缩流包装成一个可读取的 reader，后续 io::copy 会边读边解压写入目标文件。
//
// 4. .tar / .tar.gz / .tgz 走 parse_tar_reader：
//      - tar::Archive 会把一个 reader 解释成 tar 包，然后 entries() 逐个读包内文件。
//      - 代码不会直接解压整个目录，而是逐个 entry 判断：json 是元信息，录像文件才会落盘成 RecordingEntry。
//
// 5. .zip 走 ZipArchive：
//      - ZipArchive::new 会读取 zip 中央目录，archive.len() 是包内 entry 数量，by_index(index) 取出某个 entry。
//      - enclosed_name() 用来拿安全路径，避免 zip 里出现 ../ 这类路径穿越。
// 
// 6. 压缩包里的 json 文件会被 read_metadata 解析成 RecordingMetadata。
//    当前实现是读到 json 后，后面的录像 entry 复用这份 metadata，所以如果包内有多个分片，它们会共享最近一次读到的元信息。
//
// 7. 压缩包里的录像文件统一交给 create_entry_from_reader：
//    它负责生成 id、创建缓存目录、写入真实播放文件、计算文件大小、生成 playable_url，最后返回 RecordingEntry。
//
// 8. RecordingEntry 是后端内部结构，里面包含真实 content_path；前端后续只应该拿 manifest.playable_url，不应该看到真实本地路径。

/// 离线录像包解析器
///
/// 只负责把用户选择的本地文件转换成后端可登记、可播放的 RecordingEntry。
#[derive(Debug, Clone)]
pub struct OfflinePackageParser {
    storage: OfflineStorage,
}

impl OfflinePackageParser {
    pub fn new(storage: OfflineStorage) -> Self {
        Self { storage }
    }

    /// 解析单个离线录像文件
    ///
    /// AsRef 表示 source 不限定必须是 PathBuf 或 &Path，只要它可以被看作 Path 就行
    /// Path 表示一个路径，不一定是文件
    pub fn parse_file(&self, source: impl AsRef<Path>) -> Result<Vec<RecordingEntry>> {
        self.storage.ensure_ready()?;

        let source = source.as_ref();

        if !source.exists() {
            bail!("recording file does not exist: {:?}", source);
        }

        if !source.is_file() {
            bail!("recording path is not a file: {:?}", source);
        }

        // .file_name() 的意思是：取路径最后一段，返回 Option<&OsStr> 这是因为：在 Unix / macOS / Linux 上，文件路径不一定是合法 UTF-8
        // OsStr 操作系统字符串，to_str() 尝试把 OsStr 转成 Rust 普通字符串 &str
        let file_name = source
            .file_name()
            .and_then(OsStr::to_str)
            .context("recording file name is invalid utf-8")?;

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            return self.parse_tar_gz(source);
        }

        if file_name.ends_with(".tar") {
            return self.parse_tar(source);
        }

        if file_name.ends_with(".zip") {
            return self.parse_zip(source);
        }

        if file_name.ends_with(".mp4") {
            return Ok(vec![self.parse_mp4(source, file_name)?]);
        }

        if file_name.ends_with(".gz") {
            return Ok(vec![self.parse_gz(source, file_name)?]);
        }

        bail!("unsupported offline recording file: {}", file_name);
    }

    /// 解析 MP4
    fn parse_mp4(&self, source: &Path, file_name: &str) -> Result<RecordingEntry> {
        let id = new_recording_id();
        let playable_url = playable_url(&id);
        // MP4 本身就是可播放文件，所以只需要复制进目录中
        let target_path = self.storage.copy_into_recording(source, &id, file_name)?;
        let file_size = file_size(&target_path)?;
        let working_dir = self.storage.recording_dir(&id)?;

        let manifest = RecordingManifest::new(
            id,
            strip_known_suffix(file_name).to_string(),
            RecordingKind::Mp4,
            RecordingMetadata::default(),
            playable_url,
            file_size,
            now_string(),
        );

        Ok(RecordingEntry {
            manifest,
            content_path: target_path,
            working_dir,
        })
    }

    /// 解析 gz
    fn parse_gz(&self, source: &Path, file_name: &str) -> Result<RecordingEntry> {
        let kind = detect_gz_kind(file_name)?;
        let id = new_recording_id();
        let playable_url = playable_url(&id);
        let target_file_name = strip_gz_suffix(file_name);
        let target_path = self.storage.recording_file_path(&id, &target_file_name)?;

        // 创建目标目录
        self.storage.create_recording_dir(&id)?;
        // 把 .gz 解压成真实可播放文件
        gunzip_to_file(source, &target_path)?;

        let file_size = file_size(&target_path)?;
        let working_dir = self.storage.recording_dir(&id)?;

        let manifest = RecordingManifest::new(
            id,
            strip_known_suffix(&target_file_name).to_string(),
            kind,
            RecordingMetadata::default(),
            playable_url,
            file_size,
            now_string(),
        );

        Ok(RecordingEntry {
            manifest,
            content_path: target_path,
            working_dir,
        })
    }

    /// 解析 tar 包
    fn parse_tar(&self, source: &Path) -> Result<Vec<RecordingEntry>> {
        let file = fs::File::open(source)
            .with_context(|| format!("open tar file failed: {:?}", source))?;

        self.parse_tar_reader(file)
    }

    /// 解析 tar.gz / tgz 包
    fn parse_tar_gz(&self, source: &Path) -> Result<Vec<RecordingEntry>> {
        let file = fs::File::open(source)
            .with_context(|| format!("open tar.gz file failed: {:?}", source))?;

        let decoder = GzDecoder::new(file);
        self.parse_tar_reader(decoder)
    }

    /// 解析 zip 包
    fn parse_zip(&self, source: &Path) -> Result<Vec<RecordingEntry>> {
        let file = fs::File::open(source)
            .with_context(|| format!("open zip file failed: {:?}", source))?;

        // ZipArchive::new(file) 返回 Result<ZipArchive<File>, ZipError>
        // 也就是：尝试把一个已经打开的文件解析成 zip 包对象，它不是解压后的目录，也不是文件内容本身。它只是一个 zip 读取器
        let mut archive = ZipArchive::new(file)
            .with_context(|| format!("read zip archive failed: {:?}", source))?;

        let mut metadata = RecordingMetadata::default();
        let mut entries = Vec::new();

        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .with_context(|| format!("read zip entry failed: index={}", index))?;

            if file.is_dir() {
                continue;
            }

            // enclosed_name 可以确保文件路径可以安全地用作 Path
            let Some(path) = file.enclosed_name() else {
                continue;
            };

            let Some(file_name) = safe_archive_file_name(&path) else {
                continue;
            };

            if is_metadata_file(&file_name) {
                metadata = read_metadata(&mut file).unwrap_or(metadata);
                continue;
            }

            let Some(kind) = detect_archive_recording_kind(&file_name) else {
                continue;
            };

            entries.push(self.create_entry_from_reader(
                &file_name,
                kind,
                metadata.clone(),
                &mut file,
            )?);
        }

        ensure_entries_not_empty(entries)
    }

    /// 从 tar reader 中解析录像条目
    fn parse_tar_reader<R>(&self, reader: R) -> Result<Vec<RecordingEntry>>
    where
        R: Read,
    {
        let mut archive = Archive::new(reader);
        let mut metadata = RecordingMetadata::default();
        let mut entries = Vec::new();

        for item in archive.entries().context("read tar entries failed")? {
            let mut item = item.context("read tar entry failed")?;

            if item.header().entry_type().is_dir() {
                continue;
            }

            let path = item
                .path()
                .context("read tar entry path failed")?
                .to_path_buf();

            let Some(file_name) = safe_archive_file_name(&path) else {
                continue;
            };

            if is_metadata_file(&file_name) {
                metadata = read_metadata(&mut item).unwrap_or(metadata);
                continue;
            }

            let Some(kind) = detect_archive_recording_kind(&file_name) else {
                continue;
            };

            entries.push(self.create_entry_from_reader(
                &file_name,
                kind,
                metadata.clone(),
                &mut item,
            )?);
        }

        ensure_entries_not_empty(entries)
    }

    /// 从 archive entry 创建录像登记
    fn create_entry_from_reader<R>(
        &self,
        file_name: &str,
        kind: RecordingKind,
        metadata: RecordingMetadata,
        reader: &mut R,
    ) -> Result<RecordingEntry>
    where
        R: Read,
    {
        let id = new_recording_id();
        let playable_url = playable_url(&id);
        let target_file_name = if file_name.ends_with(".gz") {
            strip_gz_suffix(file_name)
        } else {
            file_name.to_string()
        };

        self.storage.create_recording_dir(&id)?;
        let target_path = self.storage.recording_file_path(&id, &target_file_name)?;

        if file_name.ends_with(".gz") {
            gunzip_reader_to_file(reader, &target_path)?;
        } else {
            copy_reader_to_file(reader, &target_path)?;
        }

        let file_size = file_size(&target_path)?;
        let working_dir = self.storage.recording_dir(&id)?;

        let manifest = RecordingManifest::new(
            id,
            strip_known_suffix(&target_file_name).to_string(),
            kind,
            metadata,
            playable_url,
            file_size,
            now_string(),
        );

        Ok(RecordingEntry {
            manifest,
            content_path: target_path,
            working_dir,
        })
    }
}
