//! 诊断上下文，用于关联同一战斗/回合/行动中的多条日志。

use bevy::prelude::Entity;

use super::{CorrelationId, LogCode};

/// 诊断上下文，用于关联同一战斗/回合/行动中的多条日志。
///
/// 通过 Builder 模式构建，支持链式调用：
/// ```ignore
/// let ctx = DiagnosticContext::default()
///     .with_correlation(CorrelationId::Battle(42))
///     .with_entity(entity)
///     .with_tag("combat");
/// ```
///
/// 日志输出方法自动携带上下文字段：
/// ```ignore
/// ctx.log_info(LogCode::BAT001, "battle_started");
/// ```
///
/// 可通过 `with_extra` 添加事件私有结构化字段：
/// ```ignore
/// ctx.with_extra("spec_id", "abl_000001")
///    .with_extra("damage", "150")
///    .log_info(LogCode::BAT007, "damage_applied");
/// ```
#[derive(Debug, Clone, Default)]
pub struct DiagnosticContext {
    /// 关联 ID（战斗/回合/行动）
    pub correlation: Option<CorrelationId>,
    /// 实体标识（哪个单位）
    pub entity: Option<Entity>,
    /// 帧号（确定性时间点）
    pub frame: Option<u64>,
    /// 回合号
    pub turn: Option<u32>,
    /// 轮次号
    pub round: Option<u32>,
    /// 额外标签（用于过滤/搜索）
    pub tags: Vec<String>,
    /// 事件私有扩展字段（如 spec_id, target, damage 等）
    pub extras: Vec<(String, String)>,
}

impl DiagnosticContext {
    /// 设置关联 ID。
    pub fn with_correlation(mut self, id: CorrelationId) -> Self {
        self.correlation = Some(id);
        self
    }

    /// 设置关联实体。
    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }

    /// 设置帧号。
    pub fn with_frame(mut self, frame: u64) -> Self {
        self.frame = Some(frame);
        self
    }

    /// 设置回合号。
    pub fn with_turn(mut self, turn: u32) -> Self {
        self.turn = Some(turn);
        self
    }

    /// 设置轮次号。
    pub fn with_round(mut self, round: u32) -> Self {
        self.round = Some(round);
        self
    }

    /// 添加额外标签。
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// 添加事件私有扩展字段。
    /// 用于传递事件特有的结构化数据（如 spec_id、damage、target）。
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extras.push((key.into(), value.into()));
        self
    }

    /// 输出 INFO 级别结构化日志，自动携带诊断上下文。
    #[track_caller]
    pub fn log_info(&self, code: LogCode, event: &'static str) {
        let mut fields = vec![format!("code = {:?}", code), format!("event = {:?}", event)];
        if let Some(ref corr) = self.correlation {
            fields.push(format!("correlation = {:?}", corr));
        }
        if let Some(entity) = self.entity {
            fields.push(format!("entity = {:?}", entity));
        }
        if let Some(frame) = self.frame {
            fields.push(format!("frame = {:?}", frame));
        }
        if let Some(turn) = self.turn {
            fields.push(format!("turn = {:?}", turn));
        }
        if let Some(round) = self.round {
            fields.push(format!("round = {:?}", round));
        }
        if !self.tags.is_empty() {
            fields.push(format!("tags = {:?}", self.tags));
        }
        for (k, v) in &self.extras {
            fields.push(format!("{} = {:?}", k, v));
        }
        tracing::info!(
            code = ?code,
            event = event,
            correlation = ?self.correlation,
            entity = ?self.entity,
            frame = ?self.frame,
            turn = ?self.turn,
            round = ?self.round,
            tags = ?self.tags,
            extras = ?self.extras,
            "{}", event
        );
    }

    /// 输出 WARN 级别结构化日志，自动携带诊断上下文。
    #[track_caller]
    pub fn log_warn(&self, code: LogCode, event: &'static str) {
        tracing::warn!(
            code = ?code,
            event = event,
            correlation = ?self.correlation,
            entity = ?self.entity,
            frame = ?self.frame,
            turn = ?self.turn,
            round = ?self.round,
            tags = ?self.tags,
            extras = ?self.extras,
            "{}", event
        );
    }

    /// 输出 ERROR 级别结构化日志，自动携带诊断上下文。
    #[track_caller]
    pub fn log_error(&self, code: LogCode, event: &'static str) {
        tracing::error!(
            code = ?code,
            event = event,
            correlation = ?self.correlation,
            entity = ?self.entity,
            frame = ?self.frame,
            turn = ?self.turn,
            round = ?self.round,
            tags = ?self.tags,
            extras = ?self.extras,
            "{}", event
        );
    }
}

impl std::fmt::Display for DiagnosticContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref corr) = self.correlation {
            write!(f, "[{}]", corr)?;
        }
        if let Some(entity) = self.entity {
            write!(f, " Entity({})", entity.index())?;
        }
        if let Some(frame) = self.frame {
            write!(f, " Frame({})", frame)?;
        }
        Ok(())
    }
}
