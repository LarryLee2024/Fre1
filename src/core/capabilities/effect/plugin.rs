use bevy::prelude::*;

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, _app: &mut App) {
        // ── 当前 Phase B 状态 ──────────────────────────────────
        //
        // Effect 领域层实现已完成（foundation + mechanism/lifecycle 纯函数），
        // EffectApplied/EffectRemoved/EffectTicked/EffectImmunityTriggered 事件已定义。
        //
        // ECS 集成注册时机（将在后续 Phase 实现）：
        //   1. ActiveEffectContainer 注册为 Component（当前在 foundation/values.rs 中是值对象）
        //   2. app.add_observer(on_effect_applied)    — 将 EffectInstance 写入容器
        //   3. app.add_observer(on_effect_removed)    — 从容器中移除 EffectInstance
        //   4. app.add_observer(on_effect_ticked)     — 处理 Tick 变更
        //
        // 详见 docs/02-domain/effect_domain.md
        // 详见 docs/04-data/capabilities/effect_schema.md
    }
}
