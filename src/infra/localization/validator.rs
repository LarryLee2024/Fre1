//! 启动校验 — 检查 Key 完整性、Orphan Key、覆盖率
//!
//! 必须在所有 .ftl 加载完成后运行。
//! 缺失 Key → panic（阻止启动），Orphan → warn，覆盖率低 → warn。
//!
//! 详见 `docs/03-technical/localization-design.md` §6

use bevy::prelude::*;

use super::database::LocalizationDatabase;
use crate::infra::localization::generated::ALL_KEYS;

/// 启动时校验 —— 必须在所有 .ftl 加载完成后运行
///
/// 注册方式：
/// ```ignore
/// app.add_systems(Startup, validation_system.after(load_all_ftl_system));
/// ```
pub fn validation_system(db: Res<LocalizationDatabase>) {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 已生成 key 的列表
    let all_keys: &[&str] = ALL_KEYS;

    // ── 1. 缺失 Key ──
    // Rust 代码引用了，但 en-US 的 .ftl 中没有
    for key in all_keys {
        if !db.has_key("en-US", key) {
            errors.push(format!(
                "MISSING KEY: '{}' — referenced in code, not found in en-US .ftl files",
                key
            ));
        }
    }

    // ── 2. Orphan Key ──
    // .ftl 中存在，但没有 Rust 代码引用
    for key in db.all_keys("en-US") {
        if !all_keys.contains(&key) {
            warnings.push(format!(
                "ORPHAN KEY: '{}' — defined in .ftl but never referenced in code",
                key
            ));
        }
    }

    // ── 3. 覆盖率检查 ──
    let coverage = db.coverage();
    if coverage < 0.80 {
        warnings.push(format!(
            "LOW COVERAGE: current locale '{}' has {:.1}% translation coverage (threshold: 80%)",
            db.current_locale(),
            coverage * 100.0,
        ));
    }

    // ── 输出 ──
    for warn in &warnings {
        warn!(target: "localization", "[Localization] {}", warn);
    }

    if !errors.is_empty() {
        error!(target: "localization", 
            "[Localization] 验证失败，共 {} 个错误：",
            errors.len()
        );
        for err in &errors {
            error!(target: "localization", "  {}", err);
        }
        panic!(
            "[Localization] 验证失败：{} 个错误（见上方）。\
             为阻止显示未翻译文本，已阻止启动。",
            errors.len()
        );
    }

    info!(target: "localization", 
        "[Localization] 验证通过。{} 个键正常，{} 个警告。",
        all_keys.len(),
        warnings.len()
    );
}
