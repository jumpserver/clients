use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;

pub struct ConfigService;

impl ConfigService {
    /// 获取用户配置目录中的 config.json 路径
    fn get_user_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        // 使用系统配置目录 + 自定义应用名 "jumpserver-client"
        let config_dir = app
            .path()
            .config_dir()
            .map_err(|e| format!("Failed to get config directory: {}", e))?
            .join("jumpserver-client");

        // 确保配置目录存在
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
            log::info!("Created config directory: {:?}", config_dir);
        }

        Ok(config_dir.join("config.json"))
    }

    /// 获取资源目录中的 config.json 路径（作为默认模板）
    fn resolve_resource_path(app: &tauri::AppHandle) -> Option<PathBuf> {
        app.path()
            .resolve(
                "resources/bin/config.json",
                tauri::path::BaseDirectory::Resource,
            )
            .ok()
            .filter(|p| p.is_file())
    }

    /// 开发环境下的配置路径
    fn resolve_dev_path() -> Option<PathBuf> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        log::info!("Current working directory: {:?}", cwd);

        let candidates = [
            cwd.join("resources/bin/config.json"),
            cwd.join("../config.json"),
            cwd.join("../../config.json"),
            cwd.join("../../../config.json"),
        ];
        let result = candidates.into_iter().find(|p| p.is_file());
        log::info!("Selected dev config path: {:?}", result);
        result
    }

    /// 获取配置版本号
    fn get_config_version(config: &Value) -> i64 {
        config.get("version").and_then(|v| v.as_i64()).unwrap_or(1)
    }

    /// 合并用户配置项中的自定义设置（如 match_first）
    fn merge_app_items(user_items: &Value, default_items: &Value) -> Value {
        if !user_items.is_array() || !default_items.is_array() {
            return default_items.clone();
        }

        let user_arr = user_items.as_array().unwrap();
        let default_arr = default_items.as_array().unwrap();

        let mut result = default_arr.clone();

        // 遍历用户配置，保留用户的 match_first 等自定义字段
        for user_item in user_arr {
            if let Some(user_name) = user_item.get("name").and_then(|v| v.as_str()) {
                // 在默认配置中查找对应的项
                for result_item in result.iter_mut() {
                    if let Some(result_name) = result_item.get("name").and_then(|v| v.as_str()) {
                        if result_name == user_name {
                            // 保留用户的 match_first 设置
                            if let Some(match_first) = user_item.get("match_first") {
                                result_item
                                    .as_object_mut()
                                    .unwrap()
                                    .insert("match_first".to_string(), match_first.clone());
                            }
                            // 保留用户的 path 设置（如果用户自定义了路径）
                            if let Some(user_path) = user_item.get("path") {
                                if let Some(user_path_str) = user_path.as_str() {
                                    if !user_path_str.is_empty() {
                                        // 检查默认配置的 path 是否不同
                                        let default_path = result_item
                                            .get("path")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        // 如果用户设置了不同的路径，则保留
                                        if user_path_str != default_path {
                                            result_item
                                                .as_object_mut()
                                                .unwrap()
                                                .insert("path".to_string(), user_path.clone());
                                        }
                                    }
                                }
                            }
                            // 保留用户的 is_set 状态
                            if let Some(is_set) = user_item.get("is_set") {
                                result_item
                                    .as_object_mut()
                                    .unwrap()
                                    .insert("is_set".to_string(), is_set.clone());
                            }
                            break;
                        }
                    }
                }
            }
        }

        Value::Array(result)
    }

    /// 合并操作系统级别的配置
    fn merge_os_config(user_os: &Value, default_os: &Value) -> Value {
        if !user_os.is_object() || !default_os.is_object() {
            return default_os.clone();
        }

        let mut result = default_os.clone();
        let result_obj = result.as_object_mut().unwrap();

        // 合并每个类别（terminal, remotedesktop, filetransfer, databases）
        for category in ["terminal", "remotedesktop", "filetransfer", "databases"] {
            if let (Some(user_items), Some(default_items)) =
                (user_os.get(category), default_os.get(category))
            {
                let merged = Self::merge_app_items(user_items, default_items);
                result_obj.insert(category.to_string(), merged);
            }
        }

        result
    }

    /// 合并配置文件，保留用户的自定义设置
    fn merge_configs(user_config: Value, default_config: Value) -> Value {
        let default_version = Self::get_config_version(&default_config);

        let mut merged = default_config.clone();
        let merged_obj = merged.as_object_mut().unwrap();

        // 更新版本号为默认配置的版本
        merged_obj.insert("version".to_string(), json!(default_version));

        // 保留核心结构字段（从默认配置）
        for key in ["filename", "windowBounds", "defaultSetting"] {
            if let Some(value) = default_config.get(key) {
                merged_obj.insert(key.to_string(), value.clone());
            }
        }

        // 合并每个操作系统的配置
        for os_key in ["windows", "macos", "linux"] {
            if let (Some(user_os), Some(default_os)) =
                (user_config.get(os_key), default_config.get(os_key))
            {
                let merged_os = Self::merge_os_config(user_os, default_os);
                merged_obj.insert(os_key.to_string(), merged_os);
            } else if let Some(default_os) = default_config.get(os_key) {
                // 如果用户配置中没有该操作系统，使用默认配置
                merged_obj.insert(os_key.to_string(), default_os.clone());
            }
        }

        merged
    }

    /// 更新用户配置（如果默认配置版本更高）
    fn update_user_config_if_needed(
        user_config_path: &PathBuf,
        default_config_path: &PathBuf,
    ) -> Result<(), String> {
        // 读取默认配置
        let default_content = std::fs::read_to_string(default_config_path)
            .map_err(|e| format!("Failed to read default config: {}", e))?;
        let default_config: Value = serde_json::from_str(&default_content)
            .map_err(|e| format!("Failed to parse default config: {}", e))?;

        // 读取用户配置
        let user_content = std::fs::read_to_string(user_config_path)
            .map_err(|e| format!("Failed to read user config: {}", e))?;
        let user_config: Value = serde_json::from_str(&user_content)
            .map_err(|e| format!("Failed to parse user config: {}", e))?;

        let default_version = Self::get_config_version(&default_config);
        let user_version = Self::get_config_version(&user_config);

        log::info!(
            "Config versions - User: {}, Default: {}",
            user_version,
            default_version
        );

        // 如果默认配置版本更高，则合并配置
        if default_version > user_version {
            log::info!(
                "Upgrading config from version {} to {}",
                user_version,
                default_version
            );

            let merged_config = Self::merge_configs(user_config, default_config);

            // 写入合并后的配置（使用原子写入）
            let pretty = serde_json::to_string_pretty(&merged_config)
                .map_err(|e| format!("Failed to serialize merged config: {}", e))?;
            Self::atomic_write_config(user_config_path, &pretty)?;

            log::info!(
                "Config upgraded successfully to version {}",
                default_version
            );
        } else {
            log::info!("User config is up to date, no upgrade needed");
        }

        Ok(())
    }

    /// 检查文件是否存在且可读（比 exists() 更可靠）
    fn is_valid_config_file(path: &PathBuf) -> bool {
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    log::warn!("Config path exists but is not a file: {:?}", path);
                    return false;
                }
                // 尝试读取文件头部验证可访问性
                match std::fs::File::open(path) {
                    Ok(mut file) => {
                        use std::io::Read;
                        let mut buf = [0u8; 1];
                        match file.read(&mut buf) {
                            Ok(_) => true,
                            Err(e) => {
                                log::warn!("Config file exists but cannot be read: {:?}, error: {}", path, e);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Config file exists but cannot be opened: {:?}, error: {}", path, e);
                        false
                    }
                }
            }
            Err(e) => {
                log::info!("Config file does not exist or cannot be accessed: {:?}, error: {}", path, e);
                false
            }
        }
    }

    /// 验证配置文件内容是否有效
    fn is_valid_config_content(path: &PathBuf) -> bool {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        // 检查必要的字段是否存在
                        if json.get("version").is_none() {
                            log::warn!("Config file missing 'version' field: {:?}", path);
                            return false;
                        }
                        true
                    }
                    Err(e) => {
                        log::warn!("Config file contains invalid JSON: {:?}, error: {}", path, e);
                        false
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read config file: {:?}, error: {}", path, e);
                false
            }
        }
    }

    /// 备份用户配置文件
    fn backup_user_config(path: &PathBuf) -> Result<PathBuf, String> {
        let backup_path = path.with_extension("json.bak");
        std::fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to backup config: {}", e))?;
        log::info!("Backed up user config to {:?}", backup_path);
        Ok(backup_path)
    }

    /// 原子写入配置文件（先写入临时文件，再重命名）
    fn atomic_write_config(path: &PathBuf, content: &str) -> Result<(), String> {
        let temp_path = path.with_extension("json.tmp");

        // 写入临时文件
        {
            let mut file = std::fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp config file: {}", e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write temp config file: {}", e))?;
            file.sync_all()
                .map_err(|e| format!("Failed to sync temp config file: {}", e))?;
        }

        // 验证临时文件内容有效
        if !Self::is_valid_config_content(&temp_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err("Temp config file validation failed".to_string());
        }

        // 备份当前配置（如果存在）
        if path.exists() {
            if let Err(e) = Self::backup_user_config(path) {
                log::warn!("Failed to backup before atomic write: {}", e);
            }
        }

        // 重命名临时文件为目标文件
        std::fs::rename(&temp_path, path)
            .map_err(|e| format!("Failed to rename temp config to target: {}", e))?;

        log::info!("Config written atomically to {:?}", path);
        Ok(())
    }

    /// 尝试从备份文件恢复配置
    fn try_restore_from_backup(config_path: &PathBuf) -> Option<PathBuf> {
        let backup_path = config_path.with_extension("json.bak");

        if !backup_path.exists() {
            log::info!("No backup file found at {:?}", backup_path);
            return None;
        }

        // 验证备份文件有效性
        if !Self::is_valid_config_file(&backup_path) || !Self::is_valid_config_content(&backup_path) {
            log::warn!("Backup file exists but is invalid: {:?}", backup_path);
            return None;
        }

        // 复制备份文件到配置文件
        match std::fs::copy(&backup_path, config_path) {
            Ok(_) => {
                log::info!("Successfully restored config from backup: {:?}", backup_path);
                Some(backup_path)
            }
            Err(e) => {
                log::warn!("Failed to restore from backup: {}", e);
                None
            }
        }
    }

    /// 确保用户配置文件存在，如果不存在则从模板复制
    fn ensure_user_config(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let user_config_path = Self::get_user_config_path(app)?;
        let template_path = Self::resolve_resource_path(app)
            .or_else(Self::resolve_dev_path)
            .ok_or_else(|| "config.json template not found (resource/dev)".to_string())?;

        // 使用更可靠的文件检查方法
        let config_exists = Self::is_valid_config_file(&user_config_path);
        let config_valid = config_exists && Self::is_valid_config_content(&user_config_path);

        if !config_exists {
            // 配置文件不存在，首先尝试从备份恢复
            log::info!("Config file does not exist at {:?}, trying to restore from backup", user_config_path);

            if Self::try_restore_from_backup(&user_config_path).is_some() {
                // 从备份恢复成功，验证恢复后的文件
                if Self::is_valid_config_content(&user_config_path) {
                    log::info!("Config restored from backup successfully");
                    // 检查是否需要版本升级
                    if let Err(e) = Self::update_user_config_if_needed(&user_config_path, &template_path) {
                        log::warn!("Failed to update restored config: {}", e);
                    }
                    return Ok(user_config_path);
                }
            }

            // 备份恢复失败或不存在，从模板复制
            log::info!(
                "Copying config template from {:?} to {:?}",
                template_path,
                user_config_path
            );
            std::fs::copy(&template_path, &user_config_path)
                .map_err(|e| format!("Failed to copy config template: {}", e))?;
            log::info!("Initial config created successfully");
        } else if !config_valid {
            // 配置文件存在但内容无效
            log::warn!(
                "Config file exists but is invalid at {:?}",
                user_config_path
            );

            // 尝试备份损坏的配置
            let corrupted_backup = user_config_path.with_extension("json.corrupted");
            if let Err(e) = std::fs::copy(&user_config_path, &corrupted_backup) {
                log::warn!("Failed to backup corrupted config: {}", e);
            } else {
                log::info!("Corrupted config backed up to {:?}", corrupted_backup);
            }

            // 尝试从备份恢复
            if Self::try_restore_from_backup(&user_config_path).is_some() {
                if Self::is_valid_config_content(&user_config_path) {
                    log::info!("Config restored from backup after corruption detected");
                    if let Err(e) = Self::update_user_config_if_needed(&user_config_path, &template_path) {
                        log::warn!("Failed to update restored config: {}", e);
                    }
                    return Ok(user_config_path);
                }
            }

            // 备份恢复失败，从模板重建
            log::info!("Recreating config from template");
            std::fs::copy(&template_path, &user_config_path)
                .map_err(|e| format!("Failed to copy config template: {}", e))?;
            log::info!("Config recreated from template");
        } else {
            // 配置文件存在且有效，检查是否需要更新
            log::info!(
                "User config exists and is valid at {:?}, checking for updates",
                user_config_path
            );
            if let Err(e) = Self::update_user_config_if_needed(&user_config_path, &template_path) {
                log::warn!("Failed to update user config: {}", e);
                // 不阻断流程，即使更新失败也继续使用现有配置
            }
        }

        Ok(user_config_path)
    }

    pub fn get_app_config(app: &tauri::AppHandle) -> Result<Value, String> {
        // 确保用户配置文件存在，并从用户配置目录读取
        let path = Self::ensure_user_config(app)?;

        log::info!("Reading config from: {:?}", path);

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("read config.json failed: {}", e))?;
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("parse config.json failed: {}", e))?;

        let os_key = match std::env::consts::OS {
            "macos" => "macos",
            "windows" => "windows",
            "linux" => "linux",
            other => other,
        };

        let per_os = json
            .get(os_key)
            .cloned()
            .ok_or_else(|| format!("config.json missing key for current OS: {}", os_key))?;

        Ok(per_os)
    }

    pub fn update_selection(
        app: &tauri::AppHandle,
        category: &str,
        protocol: &str,
        name: &str,
        new_path: Option<String>,
    ) -> Result<Value, String> {
        // 确保用户配置文件存在，并写入到用户配置目录
        let config_path = Self::ensure_user_config(app)?;

        log::info!("Updating config at: {:?}", config_path);

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read config.json failed: {}", e))?;
        let mut json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("parse config.json failed: {}", e))?;

        let os_key = match std::env::consts::OS {
            "macos" => "macos",
            "windows" => "windows",
            "linux" => "linux",
            other => other,
        };

        let arr = json
            .get_mut(os_key)
            .and_then(|os| os.get_mut(category))
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| format!("invalid config path: {}.{}", os_key, category))?;

        // 如果传入了路径，则只更新对应项的 path 与 is_set，不改变 match_first
        if let Some(p) = new_path.clone() {
            let trimmed = p.trim().to_string();
            if !trimmed.is_empty() {
                let mut found = false;
                for item in arr.iter_mut() {
                    let item_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if item_name == name {
                        found = true;
                        // 更新 path
                        item.as_object_mut()
                            .unwrap()
                            .insert("path".into(), Value::String(trimmed.clone()));
                        // 标记为已设置
                        item.as_object_mut()
                            .unwrap()
                            .insert("is_set".into(), Value::Bool(true));
                        break;
                    }
                }

                if !found {
                    return Err(format!(
                        "selected item '{}' not found under {}.{}",
                        name, os_key, category
                    ));
                }

                let pretty = serde_json::to_string_pretty(&json)
                    .map_err(|e| format!("serialize config.json failed: {}", e))?;
                Self::atomic_write_config(&config_path, &pretty)?;

                log::info!("Config path updated successfully at: {:?}", config_path);

                return Ok(json
                    .get(os_key)
                    .cloned()
                    .ok_or_else(|| format!("config.json missing key for current OS: {}", os_key))?);
            }
        }

        let mut found = false;

        for item in arr.iter_mut() {
            if let Some(mf) = item.get_mut("match_first") {
                if let Some(list) = mf.as_array_mut() {
                    list.retain(|v| v.as_str().map(|s| s != protocol).unwrap_or(true));
                }
            }
        }

        for item in arr.iter_mut() {
            let item_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if item_name == name {
                found = true;
                if !item.get("match_first").is_some() {
                    item.as_object_mut()
                        .unwrap()
                        .insert("match_first".into(), Value::Array(vec![]));
                }
                if let Some(list) = item.get_mut("match_first").and_then(|v| v.as_array_mut()) {
                    list.push(Value::String(protocol.to_string()));
                }
                break;
            }
        }

        if !found {
            return Err(format!(
                "selected item '{}' not found under {}.{}",
                name, os_key, category
            ));
        }

        let pretty = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("serialize config.json failed: {}", e))?;
        Self::atomic_write_config(&config_path, &pretty)?;

        log::info!("Config updated successfully at: {:?}", config_path);

        Ok(json
            .get(os_key)
            .cloned()
            .ok_or_else(|| format!("config.json missing key for current OS: {}", os_key))?)
    }
}
