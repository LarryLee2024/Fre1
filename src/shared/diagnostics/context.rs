//! 诊断上下文，用于关联同一战斗/回合/行动中的多条日志。

use bevy::prelude::Entity;

use super::CorrelationId;

/// 诊断上下文，用于关联同一战斗/回合/行动中的多条日志。
///
/// 通过 Builder 模式构建，支持链式调用：
/// ```ignore
/// let ctx = DiagnosticContext::default()
///     .with_correlation(CorrelationId::Battle(42))
///     .with_entity(entity)
///     .with_tag("combat");
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
