//! file_sink — 可轮转的文件日志输出后端
//!
//! 将结构化日志以 JSON 格式写入文件，支持按大小轮转。
//! 通过 `FileSinkConfig` 配置输出路径、轮转阈值。

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

/// 文件日志输出器配置。
#[derive(Debug, Clone)]
pub struct FileSinkConfig {
    /// 日志文件目录
    pub dir: PathBuf,
    /// 日志文件前缀名
    pub prefix: String,
    /// 单文件最大字节数（超过后轮转）
    pub max_bytes: u64,
    /// 保留的最大轮转文件数
    pub max_files: usize,
    /// 是否启用
    pub enabled: bool,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("logs"),
            prefix: "game".to_string(),
            max_bytes: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            enabled: cfg!(debug_assertions),
        }
    }
}

/// 文件日志输出器。
///
/// 线程安全（内部使用 Mutex），可在 tracing 订阅器中跨线程使用。
pub struct FileSink {
    config: FileSinkConfig,
    writer: Mutex<FileWriter>,
}

struct FileWriter {
    file: File,
    path: PathBuf,
    bytes_written: u64,
}

impl FileSink {
    /// 使用指定配置创建文件日志输出器。
    pub fn new(config: FileSinkConfig) -> Self {
        fs::create_dir_all(&config.dir).expect("Failed to create log directory");

        let path = config.dir.join(format!("{}.jsonl", config.prefix));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("Failed to open log file");

        let bytes_written = file.metadata().map(|m| m.len()).unwrap_or(0);

        Self {
            writer: Mutex::new(FileWriter {
                file,
                path,
                bytes_written,
            }),
            config,
        }
    }

    /// 写入一条 JSON 格式的日志。
    ///
    /// 如果启用且文件超过 `max_bytes`，自动轮转。
    pub fn write(&self, json_line: &str) {
        if !self.config.enabled {
            return;
        }

        let mut writer = self.writer.lock().expect("FileSink lock poisoned");
        let line_len = json_line.len() as u64 + 1; // +1 for newline

        if writer.bytes_written + line_len > self.config.max_bytes
            && let Ok(rotated) = Self::rotate(&self.config, &writer.path)
        {
            writer.file = rotated;
            writer.bytes_written = 0;
        }

        if let Err(e) = writeln!(writer.file, "{}", json_line) {
            eprintln!("[FileSink] Write error: {}", e);
        } else {
            writer.bytes_written += line_len;
        }
    }

    /// 轮转日志文件：重命名当前文件 → 写入新文件。
    fn rotate(config: &FileSinkConfig, current_path: &PathBuf) -> std::io::Result<File> {
        // 清理最旧的文件
        let rotated_path = config
            .dir
            .join(format!("{}.{}.jsonl", config.prefix, chrono_now()));
        fs::rename(current_path, &rotated_path)?;

        // 限制保留文件数
        cleanup_old_files(config);

        // 创建新文件
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(current_path)?;
        Ok(file)
    }
}

/// 返回适用于文件名的当前时间戳字符串。
fn chrono_now() -> String {
    // 使用简单的时间格式，不引入 chrono 依赖
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

/// 保留最近的 max_files 个轮转文件，删除更旧的。
fn cleanup_old_files(config: &FileSinkConfig) {
    let mut entries: Vec<_> = Vec::new();
    if let Ok(dir) = fs::read_dir(&config.dir) {
        for entry in dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&config.prefix)
                && name_str.ends_with(".jsonl")
                && name_str.contains('.')
            {
                entries.push(entry.path());
            }
        }
    }
    // 按修改时间排序，删除最旧的
    entries.sort_by_key(|p| fs::metadata(p).ok().and_then(|m| m.modified().ok()));
    if entries.len() > config.max_files {
        for old in entries.iter().take(entries.len() - config.max_files) {
            let _ = fs::remove_file(old);
        }
    }
}

/// 将结构化日志事件格式化为 JSON 字符串。
pub fn format_json(code: &str, event: &str, level: &str, fields: &[(&str, &str)]) -> String {
    let mut map = serde_json::Map::new();
    map.insert("timestamp".into(), chrono_now().into());
    map.insert("level".into(), level.into());
    map.insert("code".into(), code.into());
    map.insert("event".into(), event.into());
    for (k, v) in fields {
        map.insert(k.to_string(), v.to_string().into());
    }
    serde_json::Value::Object(map).to_string()
}
