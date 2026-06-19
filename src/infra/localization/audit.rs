//! 运行时审计 — 覆盖率报告和 Orphan Key 检测
//!
//! 定期输出当前 locale 的翻译覆盖率和状态概览。
//!
//! 详见 `docs/03-technical/localization-design.md`

use bevy::prelude::*;

use super::database::LocalizationDatabase;

/// 审计周期（秒）
const AUDIT_INTERVAL_SECONDS: f64 = 300.0; // 每 5 分钟

/// 运行时计时器资源
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AuditTimer {
    timer: Timer,
}

impl Default for AuditTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(AUDIT_INTERVAL_SECONDS as f32, TimerMode::Repeating),
        }
    }
}

/// 运行时审计系统：定期输出覆盖率报告
pub fn audit_system(
    time: Res<Time>,
    mut audit_timer: ResMut<AuditTimer>,
    db: Res<LocalizationDatabase>,
) {
    audit_timer.timer.tick(time.delta());
    if !audit_timer.timer.just_finished() {
        return;
    }

    let locales = db.loaded_locales();
    let mut locale_count = 0;
    let mut total_patterns = 0;

    for locale in &locales {
        let count = db.pattern_count(locale);
        total_patterns += count;
        locale_count += 1;
    }

    let coverage = db.coverage();
    let missing = db.missing_keys();

    info!(
        "[Localization Audit] {} locales, {} total patterns. Current: '{}'",
        locale_count,
        total_patterns,
        db.current_locale()
    );

    if !missing.is_empty() {
        warn!(
            "[Localization Audit] Current locale '{}' is missing {} keys (coverage: {:.1}%)",
            db.current_locale(),
            missing.len(),
            coverage * 100.0,
        );
    } else {
        info!(
            "[Localization Audit] Current locale '{}' has 100% coverage",
            db.current_locale()
        );
    }

    if coverage < 0.80 {
        warn!(
            "[Localization Audit] Low coverage ({:.1}%) for locale '{}'",
            coverage * 100.0,
            db.current_locale()
        );
    }
}
