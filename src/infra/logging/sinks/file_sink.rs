//! file_sink — 可轮转的文件日志输出后端
//!
//! 将结构化日志以 JSON 格式写入文件，支持按大小轮转。
//! 通过 `FileSinkConfig` 配置输出路径、轮转阈值。
//!
//! 还提供 `FileSinkLayer`，一个 tracing-subscriber Layer，
//! 可自动捕获所有 tracing 事件并写入 JSON 文件。

use bevy::prelude::Resource;

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};

use tracing_subscriber::{Layer, layer::Context};

/// 文件日志输出器配置。
#[derive(Debug, Clone, Resource)]
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
            eprintln!("[FileSink] 写入错误：{}", e);
        } else {
            writer.bytes_written += line_len;
        }
    }

    /// 轮转日志文件：重命名当前文件 → 写入新文件。
    fn rotate(config: &FileSinkConfig, current_path: &PathBuf) -> std::io::Result<File> {
        // 使用 epoch 秒作为文件名后缀（避免 ISO 8601 冒号导致 Windows 路径问题）
        let rotated_path = config
            .dir
            .join(format!("{}.{}.jsonl", config.prefix, epoch_secs()));
        fs::rename(current_path, &rotated_path)?;

        // 限制保留文件数
        cleanup_old_files(config);

        // 轮转后创建新文件继续写入，旧文件保留供日志分析工具消费
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(current_path)?;
        Ok(file)
    }
}

/// 返回 ISO 8601 格式的当前 UTC 时间戳（用于 JSON 日志输出）。
///
/// 格式示例：`2026-06-19T10:30:45.123Z`
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let days = total_secs / 86_400;
    let time_secs = total_secs % 86_400;

    let hours = time_secs / 3_600;
    let minutes = (time_secs % 3_600) / 60;
    let seconds = time_secs % 60;

    let (year, month, day) = civil_from_days(days as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

/// 返回 Unix 纪元秒数（仅用于日志文件名轮转）。
fn epoch_secs() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

/// 将天数（自 Unix 纪元）转换为公历 (year, month, day)。
///
/// 使用 Howard Hinnant 的 `civil_from_days` 算法，
/// 无需引入 chrono / time 等外部依赖。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // day of era
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
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

/// 一个 tracing-subscriber Layer，将结构化日志事件写入 JSON 文件。
///
/// 捕获所有 tracing 事件并通过 `FileSink` 以 JSON 行格式写入。
/// 每行包含：时间戳、级别、编码、事件和所有结构化字段。
pub struct FileSinkLayer {
    sink: FileSink,
}

impl FileSinkLayer {
    /// 使用给定配置创建新的 `FileSinkLayer`。
    pub fn new(config: FileSinkConfig) -> Self {
        Self {
            sink: FileSink::new(config),
        }
    }
}

impl<S: tracing::Subscriber + 'static> Layer<S> for FileSinkLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = Vec::new();
        let mut visitor = JsonFieldVisitor(&mut fields);
        event.record(&mut visitor);

        let code = fields
            .iter()
            .find(|(k, _)| *k == "code")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let event_name = fields
            .iter()
            .find(|(k, _)| *k == "event")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let level = format!("{:?}", event.metadata().level());

        let fields_ref: Vec<(&str, &str)> = fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let json = format_json(code, event_name, &level, &fields_ref);
        self.sink.write(&json);
    }
}

/// 一个 tracing `Visit` 实现，将所有事件字段收集到 Vec 中。
struct JsonFieldVisitor<'a>(&'a mut Vec<(&'static str, String)>);

impl<'a> tracing::field::Visit for JsonFieldVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.push((field.name(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.push((field.name(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.push((field.name(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.push((field.name(), value.to_string()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.push((field.name(), format!("{:.2}", value)));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.push((field.name(), format!("{:?}", value)));
    }
}
