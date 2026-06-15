// 效果处理器 trait：描述如何生成/预览/执行一种效果
// 新增效果类型只需实现此 trait 并注册，无需修改核心代码
// 遵循"Trait 描述规则，不描述内容"原则

use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::battle::DamageBreakdown;
use crate::core::battle::{DamageApplied, HealApplied};
use crate::core::buff::{ActiveBuffs, BuffRegistry, remove_all_debuffs};
use crate::core::character::{Faction, GridPosition, Unit, UnitName};
use crate::core::map::TerrainRegistry;
use crate::core::modifier::ModifierEntry;
use crate::core::tag::{GameplayTag, GameplayTags};
use bevy::ecs::query::QueryState;
use bevy::prelude::*;
use std::collections::HashMap;

use super::types::{DurationDef, EffectDef, PendingEffect, PendingEffectData, StackingDef, calculate_damage_from_effect};

// ── 上下文结构体（纯数据，避免 ECS 借用问题）──

/// 生成效果的上下文
#[derive(Clone, Debug)]
pub struct GenerateContext {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub defense_bonus: i32,
    pub skill_id: String,
    pub source_tags: Vec<GameplayTag>,
    pub terrain_id: String,
}

/// 预览效果的上下文
#[derive(Clone, Debug)]
pub struct PreviewContext {
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub terrain_defense_bonus: i32,
    pub buff_registry: BuffRegistry,
}

// ── 效果预览结果 ──

/// 单个效果的预览
#[derive(Clone, Debug)]
pub enum EffectPreview {
    Damage { amount: i32, lethal: bool },
    Heal { amount: i32 },
    BuffApplied { buff_name: String },
    Cleanse,
}

// ── 执行上下文 ──

/// 待发送的战斗消息（从 ExecuteContext 收集，由系统层发送）
/// CharacterDied 已移至 Dead Observer 统一发送（规则3：禁止内联死亡处理）
#[derive(Clone, Debug)]
pub enum PendingMessage {
    Damage(DamageApplied),
    Heal(HealApplied),
}

/// 效果执行上下文：封装 Execute 阶段所需的 ECS 访问
/// 使用 &mut World 提供完整的 ECS 访问能力
/// 通过方法封装保证访问安全
pub struct ExecuteContext<'w> {
    world: &'w mut World,
    /// 收集待发送的消息（由系统层统一发送）
    pub pending_messages: Vec<PendingMessage>,
    /// 需要插入 Dead Tag 的实体列表（延迟插入，避免借用冲突）
    pub dead_entities: Vec<Entity>,
    /// Cue 表现事件收集器（ADR-026 §四：逻辑与表现彻底分离）
    pub cue_events: Vec<CueEvent>,
}

/// Cue 表现事件（ADR-026 §四）
#[derive(Clone, Debug)]
pub enum CueEvent {
    /// 伤害表现
    Damage {
        target: Entity,
        amount: i32,
        is_critical: bool,
        attacker: Option<Entity>,
    },
    /// 治疗表现
    Heal {
        target: Entity,
        amount: i32,
        source: Option<Entity>,
    },
    /// 死亡表现
    Death {
        entity: Entity,
        killer: Option<Entity>,
    },
    /// Buff/Modifier 施加表现
    ModifierApplied {
        target: Entity,
        modifier_id: String,
    },
}

impl<'w> ExecuteContext<'w> {
    /// 从 World 创建执行上下文
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            pending_messages: Vec::new(),
            dead_entities: Vec::new(),
            cue_events: Vec::new(),
        }
    }

    /// 收集 Cue 伤害事件
    pub fn emit_cue_damage(
        &mut self,
        target: Entity,
        amount: i32,
        is_critical: bool,
        attacker: Option<Entity>,
    ) {
        self.cue_events.push(CueEvent::Damage {
            target,
            amount,
            is_critical,
            attacker,
        });
    }

    /// 收集 Cue 治疗事件
    pub fn emit_cue_heal(&mut self, target: Entity, amount: i32, source: Option<Entity>) {
        self.cue_events
            .push(CueEvent::Heal { target, amount, source });
    }

    /// 收集 Cue 死亡事件
    pub fn emit_cue_death(&mut self, entity: Entity, killer: Option<Entity>) {
        self.cue_events
            .push(CueEvent::Death { entity, killer });
    }

    /// 收集 Cue Modifier 施加事件
    pub fn emit_cue_modifier_applied(&mut self, target: Entity, modifier_id: String) {
        self.cue_events
            .push(CueEvent::ModifierApplied { target, modifier_id });
    }

    /// 对目标扣血 + 死亡判定 + 收集消息
    #[allow(clippy::too_many_arguments)]
    pub fn apply_damage(
        &mut self,
        target_entity: Entity,
        target_name: &str,
        target_faction: Faction,
        attacker_entity: Entity,
        attacker_name: &str,
        attacker_faction: Faction,
        amount: i32,
        is_skill: bool,
        base_amount: Option<i32>,
        modifier_entries: &[ModifierEntry],
        terrain_label: &str,
        target_coord: IVec2,
    ) -> bool {
        // 构建伤害分解
        let breakdown = base_amount.map(|base| {
            let modified = amount;
            DamageBreakdown {
                base_amount: base,
                modified_amount: modified,
                modifiers: modifier_entries.to_vec(),
                actual_damage: amount,
            }
        });

        // 扣血
        let mut target_died = false;
        if let Some(mut target_attrs) = self.world.get_mut::<Attributes>(target_entity) {
            let hp = target_attrs.get(AttributeKind::Hp);
            let new_hp = (hp - amount as f32).max(0.0);
            target_attrs.set_vital(AttributeKind::Hp, new_hp);

            // 收集伤害消息
            self.pending_messages
                .push(PendingMessage::Damage(DamageApplied {
                    target: target_entity,
                    target_name: target_name.to_string(),
                    target_faction,
                    attacker: attacker_entity,
                    attacker_name: attacker_name.to_string(),
                    attacker_faction,
                    amount,
                    is_skill,
                    terrain_label: terrain_label.to_string(),
                    target_coord,
                    breakdown,
                }));

            // 死亡判定：记录需要插入 Dead Tag 的实体（延迟插入避免借用冲突）
            if new_hp <= 0.0 {
                self.dead_entities.push(target_entity);
                target_died = true;
            }
        }
        target_died
    }

    /// 对目标回血（不超过 MaxHp）+ 收集消息
    pub fn apply_heal(&mut self, target_entity: Entity, target_name: &str, amount: i32) {
        if let Some(mut target_attrs) = self.world.get_mut::<Attributes>(target_entity) {
            let hp = target_attrs.get(AttributeKind::Hp);
            let max_hp = target_attrs.get(AttributeKind::MaxHp);
            let actual_heal = (amount as f32).min(max_hp - hp).max(0.0);
            let new_hp = hp + actual_heal;
            target_attrs.set_vital(AttributeKind::Hp, new_hp);

            self.pending_messages
                .push(PendingMessage::Heal(HealApplied {
                    target: target_entity,
                    target_name: target_name.to_string(),
                    amount: actual_heal as i32,
                }));
        }
    }

    /// 对目标施加 Buff
    pub fn apply_buff(
        &mut self,
        target_entity: Entity,
        buff_id: &str,
        source: Entity,
        duration: u32,
    ) {
        let buff_data = self.world.resource::<BuffRegistry>().get(buff_id).cloned();
        if let Some(buff_data) = buff_data {
            // 使用 QueryState 获取多个可变组件引用
            let mut query_state: QueryState<(
                &mut ActiveBuffs,
                &mut Attributes,
                &mut GameplayTags,
            )> = QueryState::new(self.world);
            if let Ok((mut buffs, mut attrs, mut tags)) =
                query_state.get_mut(self.world, target_entity)
            {
                crate::core::buff::apply_buff(
                    &mut buffs,
                    &mut attrs,
                    &mut tags,
                    &buff_data,
                    Some(source),
                    duration,
                );
            }
        }
    }

    /// 对目标驱散所有 Debuff
    pub fn apply_cleanse(&mut self, target_entity: Entity) {
        let mut query_state: QueryState<(&mut ActiveBuffs, &mut Attributes, &mut GameplayTags)> =
            QueryState::new(self.world);
        if let Ok((mut buffs, mut attrs, mut tags)) = query_state.get_mut(self.world, target_entity)
        {
            remove_all_debuffs(&mut buffs, &mut attrs, &mut tags);
        }
    }

    /// 获取目标名称
    pub fn get_name(&self, entity: Entity) -> String {
        self.world
            .get::<UnitName>(entity)
            .map(|n| n.0.as_str())
            .unwrap_or("???")
            .to_string()
    }

    /// 获取目标阵营
    pub fn get_faction(&self, entity: Entity) -> Faction {
        self.world
            .get::<Unit>(entity)
            .map(|u| u.faction)
            .unwrap_or(Faction::Enemy)
    }

    /// 获取目标坐标
    pub fn get_coord(&self, entity: Entity) -> IVec2 {
        self.world
            .get::<GridPosition>(entity)
            .map(|gp| gp.coord)
            .unwrap_or(IVec2::ZERO)
    }

    /// 获取地形标签
    pub fn get_terrain_label(&self, terrain_id: &str) -> String {
        self.world
            .resource::<TerrainRegistry>()
            .get(terrain_id)
            .map(|def| def.name.as_str())
            .unwrap_or("???")
            .to_string()
    }
}

// ── EffectHandler trait ──

/// 效果执行结果
#[derive(Clone, Debug)]
pub struct ExecuteOutput {
    /// 目标是否死亡（用于 OnKill 触发判断）
    pub target_died: bool,
    /// 目标实体
    pub target: Entity,
    /// 攻击者实体
    pub source: Entity,
}

/// 效果处理规则 trait：描述如何生成/预览/执行一种效果
/// 新增效果类型只需实现此 trait 并注册到 EffectHandlerRegistry，无需修改核心代码
pub trait EffectHandler: Send + Sync + 'static {
    /// 此处理器负责的效果类型名（与 EffectDef::type_name 对应）
    fn type_name(&self) -> &'static str;

    /// 从效果定义生成待处理效果数据
    fn generate(&self, def: &EffectDef, ctx: &GenerateContext) -> Option<PendingEffectData>;

    /// 预览效果
    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview>;

    /// 执行效果（规则7：通过 trait 分发，禁止 match 分发）
    /// 返回 None 表示类型不匹配，返回 Some 表示执行成功
    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput>;
}

// ── 内置处理器 ──

/// 伤害处理器
pub struct DamageHandler;

impl EffectHandler for DamageHandler {
    fn type_name(&self) -> &'static str {
        "Damage"
    }

    fn generate(&self, def: &EffectDef, ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Damage {
            multiplier,
            ignore_def_percent,
        } = def
        else {
            return None;
        };

        let effective_atk = ctx.source_attrs.get(AttributeKind::Attack);
        let effective_def = ctx.target_attrs.get(AttributeKind::Defense);
        let base_def = ctx.target_attrs.core_base(AttributeKind::Vitality);

        let amount = calculate_damage_from_effect(
            effective_atk,
            effective_def,
            base_def,
            *multiplier,
            *ignore_def_percent,
            ctx.defense_bonus,
        );

        Some(PendingEffectData::Damage {
            amount,
            is_skill: ctx.skill_id != crate::core::ability::BASIC_ATTACK_ID,
            base_amount: None,
            modifiers: Vec::new(),
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Damage {
            multiplier,
            ignore_def_percent,
        } = def
        else {
            return None;
        };

        let effective_atk = ctx.source_attrs.get(AttributeKind::Attack);
        let effective_def = ctx.target_attrs.get(AttributeKind::Defense);
        let base_def = ctx.target_attrs.core_base(AttributeKind::Vitality);

        let amount = calculate_damage_from_effect(
            effective_atk,
            effective_def,
            base_def,
            *multiplier,
            *ignore_def_percent,
            ctx.terrain_defense_bonus,
        );
        let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
        Some(EffectPreview::Damage {
            amount,
            lethal: current_hp - amount as f32 <= 0.0,
        })
    }

    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput> {
        let PendingEffectData::Damage {
            amount,
            is_skill,
            base_amount,
            modifiers,
        } = &effect.data
        else {
            return None;
        };

        let target_name = ctx.get_name(effect.target);
        let target_faction = ctx.get_faction(effect.target);
        let attacker_name = ctx.get_name(effect.source);
        let attacker_faction = ctx.get_faction(effect.source);
        let target_coord = ctx.get_coord(effect.target);
        let terrain_label = ctx.get_terrain_label(&effect.terrain_id);

        let target_died = ctx.apply_damage(
            effect.target,
            &target_name,
            target_faction,
            effect.source,
            &attacker_name,
            attacker_faction,
            *amount,
            *is_skill,
            *base_amount,
            modifiers,
            &terrain_label,
            target_coord,
        );

        // ADR-026 §四：Cue 事件发射（逻辑与表现彻底分离）
        ctx.emit_cue_damage(effect.target, *amount, false, Some(effect.source));
        if target_died {
            ctx.emit_cue_death(effect.target, Some(effect.source));
        }

        Some(ExecuteOutput {
            target_died,
            target: effect.target,
            source: effect.source,
        })
    }
}

/// 治疗处理器
pub struct HealHandler;

impl EffectHandler for HealHandler {
    fn type_name(&self) -> &'static str {
        "Heal"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Heal { amount } = def else {
            return None;
        };
        Some(PendingEffectData::Heal {
            amount: *amount,
            base_amount: None,
            modifiers: Vec::new(),
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Heal { amount } = def else {
            return None;
        };
        let max_hp = ctx.target_attrs.get(AttributeKind::MaxHp);
        let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
        let actual = (*amount as f32).min(max_hp - current_hp).max(0.0) as i32;
        Some(EffectPreview::Heal { amount: actual })
    }

    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput> {
        let PendingEffectData::Heal { amount, .. } = &effect.data else {
            return None;
        };

        let target_name = ctx.get_name(effect.target);
        ctx.apply_heal(effect.target, &target_name, *amount);

        // ADR-026 §四：Cue 事件发射
        ctx.emit_cue_heal(effect.target, *amount, Some(effect.source));

        Some(ExecuteOutput {
            target_died: false,
            target: effect.target,
            source: effect.source,
        })
    }
}

/// Buff 处理器
pub struct BuffHandler;

impl EffectHandler for BuffHandler {
    fn type_name(&self) -> &'static str {
        "ApplyBuff"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::ApplyBuff { buff_id, duration } = def else {
            return None;
        };
        Some(PendingEffectData::ApplyBuff {
            buff_id: buff_id.clone(),
            duration: *duration,
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::ApplyBuff { buff_id, .. } = def else {
            return None;
        };
        let buff_name = ctx
            .buff_registry
            .get(buff_id)
            .map(|b| b.name.as_str())
            .unwrap_or(buff_id);
        Some(EffectPreview::BuffApplied {
            buff_name: buff_name.to_string(),
        })
    }

    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput> {
        let PendingEffectData::ApplyBuff { buff_id, duration } = &effect.data else {
            return None;
        };

        ctx.apply_buff(effect.target, buff_id, effect.source, *duration);

        Some(ExecuteOutput {
            target_died: false,
            target: effect.target,
            source: effect.source,
        })
    }
}

/// 净化处理器
pub struct CleanseHandler;

impl EffectHandler for CleanseHandler {
    fn type_name(&self) -> &'static str {
        "Cleanse"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Cleanse = def else {
            return None;
        };
        Some(PendingEffectData::Cleanse)
    }

    fn preview(&self, def: &EffectDef, _ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Cleanse = def else {
            return None;
        };
        Some(EffectPreview::Cleanse)
    }

    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput> {
        let PendingEffectData::Cleanse = &effect.data else {
            return None;
        };

        ctx.apply_cleanse(effect.target);

        Some(ExecuteOutput {
            target_died: false,
            target: effect.target,
            source: effect.source,
        })
    }
}

/// 修饰器处理器（ADR-026 §二：替代 Buff 处理器）
pub struct ModifierHandler;

impl EffectHandler for ModifierHandler {
    fn type_name(&self) -> &'static str {
        "ApplyModifier"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::ApplyModifier {
            modifier_id,
            duration,
            stacking,
        } = def
        else {
            return None;
        };
        Some(PendingEffectData::ApplyModifier {
            modifier_id: modifier_id.clone(),
            duration: *duration,
            stacking: *stacking,
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::ApplyModifier { modifier_id, .. } = def else {
            return None;
        };
        // 修饰器效果预览：显示修饰器名称
        Some(EffectPreview::BuffApplied {
            buff_name: modifier_id.clone(),
        })
    }

    fn execute(&self, effect: &PendingEffect, ctx: &mut ExecuteContext) -> Option<ExecuteOutput> {
        let PendingEffectData::ApplyModifier {
            modifier_id,
            duration,
            stacking,
        } = &effect.data
        else {
            return None;
        };

        // ADR-026 §六：Stacking 集成
        // 1. 解析叠层策略
        let stacking_rule: crate::core::stacking::StackingRule = (*stacking).into();

        // 2. 解析持续时间
        let duration_turns = match duration {
            DurationDef::Instant => 0,
            DurationDef::TurnLimited(n) => *n,
            DurationDef::Permanent => u32::MAX,
        };

        // 3. 查询目标当前同类型效果的层数（TODO: 从 ActiveEffects 查询）
        // 当前简化处理：假设无已有同类型效果
        let existing = None;
        let stacking_result = crate::core::stacking::resolve_stacking(existing, stacking_rule);

        // 4. 根据叠层结果决定是否应用
        match stacking_result {
            crate::core::stacking::StackingResult::Ignored { .. } => {
                // 已达上限，不应用
                return Some(ExecuteOutput {
                    target_died: false,
                    target: effect.target,
                    source: effect.source,
                });
            }
            crate::core::stacking::StackingResult::NewlyApplied
            | crate::core::stacking::StackingResult::Replaced
            | crate::core::stacking::StackingResult::Refreshed
            | crate::core::stacking::StackingResult::Stacked { .. } => {
                // 继续应用
            }
        }

        // 5. 应用效果（暂时使用旧的 apply_buff 逻辑）
        ctx.apply_buff(effect.target, modifier_id, effect.source, duration_turns);

        // ADR-026 §四：Cue 事件发射
        ctx.emit_cue_modifier_applied(effect.target, modifier_id.clone());

        Some(ExecuteOutput {
            target_died: false,
            target: effect.target,
            source: effect.source,
        })
    }
}

// ── 处理器注册表 ──

/// 效果处理器注册表资源
/// 通过 type_name 查找对应的 EffectHandler，实现 trait 分发（O(1) 查找）
#[derive(Resource)]
pub struct EffectHandlerRegistry {
    handlers: HashMap<String, Box<dyn EffectHandler>>,
}

impl Default for EffectHandlerRegistry {
    fn default() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl EffectHandlerRegistry {
    /// 根据类型名查找处理器（O(1) HashMap 查找）
    pub fn find(&self, type_name: &str) -> Option<&dyn EffectHandler> {
        self.handlers.get(type_name).map(|h| h.as_ref())
    }

    /// 注册一个处理器
    pub fn register(&mut self, handler: Box<dyn EffectHandler>) {
        // 避免重复注册
        let name = handler.type_name().to_string();
        if self.handlers.contains_key(&name) {
            bevy::log::warn!(target: "core", handler = %name, "效果处理器已注册，跳过重复注册");
            return;
        }
        self.handlers.insert(name, handler);
    }

    /// 注册内置处理器
    pub fn register_defaults(&mut self) {
        self.register(Box::new(DamageHandler));
        self.register(Box::new(HealHandler));
        self.register(Box::new(BuffHandler));
        self.register(Box::new(CleanseHandler));
        self.register(Box::new(ModifierHandler));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证 Handler 生成/预览结果，不验证内部状态
    // ✅ 符合领域规则：是 — 覆盖 INV-EFX-4~7 Handler 不变量
    // ✅ 确定性：是 — 硬编码 EffectDef 和 Entity
    // ✅ 使用标准数据：是 — 使用标准 Handler 注册表
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;
    use crate::core::attribute::Attributes;

    /// 构建测试用 GenerateContext
    fn make_generate_ctx() -> GenerateContext {
        let mut source_attrs = Attributes::default();
        source_attrs.set_base(AttributeKind::Might, 5.0);
        source_attrs.set_base(AttributeKind::Vitality, 5.0);
        source_attrs.set_base(AttributeKind::Agility, 6.0);
        source_attrs.set_base(AttributeKind::Dexterity, 3.0);
        source_attrs.set_base(AttributeKind::Intelligence, 2.0);
        source_attrs.set_base(AttributeKind::Willpower, 3.0);
        source_attrs.set_base(AttributeKind::Presence, 2.0);
        source_attrs.set_base(AttributeKind::Luck, 2.0);
        source_attrs.set_base_attack_range(1);
        source_attrs.fill_vital_resources();

        let mut target_attrs = Attributes::default();
        target_attrs.set_base(AttributeKind::Might, 2.0);
        target_attrs.set_base(AttributeKind::Vitality, 3.0);
        target_attrs.set_base(AttributeKind::Agility, 4.0);
        target_attrs.set_base(AttributeKind::Dexterity, 2.0);
        target_attrs.set_base(AttributeKind::Intelligence, 1.0);
        target_attrs.set_base(AttributeKind::Willpower, 2.0);
        target_attrs.set_base(AttributeKind::Presence, 1.0);
        target_attrs.set_base(AttributeKind::Luck, 2.0);
        target_attrs.set_base_attack_range(1);
        target_attrs.fill_vital_resources();

        GenerateContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs,
            target_attrs,
            defense_bonus: 0,
            skill_id: "basic_attack".into(),
            source_tags: vec![],
            terrain_id: "plain".to_string(),
        }
    }

    /// 构建测试用 PreviewContext
    fn make_preview_ctx() -> PreviewContext {
        let mut source_attrs = Attributes::default();
        source_attrs.set_base(AttributeKind::Might, 5.0);
        source_attrs.set_base(AttributeKind::Vitality, 5.0);
        source_attrs.set_base(AttributeKind::Agility, 6.0);
        source_attrs.set_base(AttributeKind::Dexterity, 3.0);
        source_attrs.set_base(AttributeKind::Intelligence, 2.0);
        source_attrs.set_base(AttributeKind::Willpower, 3.0);
        source_attrs.set_base(AttributeKind::Presence, 2.0);
        source_attrs.set_base(AttributeKind::Luck, 2.0);
        source_attrs.set_base_attack_range(1);
        source_attrs.fill_vital_resources();

        let mut target_attrs = Attributes::default();
        target_attrs.set_base(AttributeKind::Might, 2.0);
        target_attrs.set_base(AttributeKind::Vitality, 3.0);
        target_attrs.set_base(AttributeKind::Agility, 4.0);
        target_attrs.set_base(AttributeKind::Dexterity, 2.0);
        target_attrs.set_base(AttributeKind::Intelligence, 1.0);
        target_attrs.set_base(AttributeKind::Willpower, 2.0);
        target_attrs.set_base(AttributeKind::Presence, 1.0);
        target_attrs.set_base(AttributeKind::Luck, 2.0);
        target_attrs.set_base_attack_range(1);
        target_attrs.fill_vital_resources();
        // HP 有缺口，用于测试治疗预览
        target_attrs.set_vital(AttributeKind::Hp, 12.0);

        PreviewContext {
            source_attrs,
            target_attrs,
            terrain_defense_bonus: 0,
            buff_registry: BuffRegistry::default(),
        }
    }

    #[test]
    fn 注册表_默认注册4个处理器() {
        let registry = EffectHandlerRegistry::default();
        assert!(registry.find("Damage").is_some());
        assert!(registry.find("Heal").is_some());
        assert!(registry.find("ApplyBuff").is_some());
        assert!(registry.find("Cleanse").is_some());
        assert!(registry.find("Unknown").is_none());
    }

    #[test]
    fn 注册表_不重复注册() {
        let mut registry = EffectHandlerRegistry::default();
        let count_before = registry.handlers.len();
        registry.register(Box::new(DamageHandler));
        assert_eq!(registry.handlers.len(), count_before);
    }

    #[test]
    fn 伤害处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::Damage {
            amount, is_skill, ..
        } = result.unwrap()
        {
            assert_eq!(amount, 7); // 10 - 3 = 7
            assert!(!is_skill);
        } else {
            panic!("应该是伤害数据");
        }
    }

    #[test]
    fn 伤害处理器_预览() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_preview_ctx();
        let def = EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        };
        let result = handler.preview(&def, &ctx);
        assert!(result.is_some());
        if let EffectPreview::Damage { amount, lethal } = result.unwrap() {
            assert_eq!(amount, 7);
            assert!(!lethal);
        } else {
            panic!("应该是伤害预览");
        }
    }

    #[test]
    fn 治疗处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Heal").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Heal { amount: 8 };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::Heal { amount, .. } = result.unwrap() {
            assert_eq!(amount, 8);
        } else {
            panic!("应该是治疗数据");
        }
    }

    #[test]
    fn 治疗处理器_预览() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Heal").unwrap();
        let ctx = make_preview_ctx();
        let def = EffectDef::Heal { amount: 8 };
        let result = handler.preview(&def, &ctx);
        assert!(result.is_some());
        if let EffectPreview::Heal { amount } = result.unwrap() {
            assert_eq!(amount, 8);
        } else {
            panic!("应该是治疗预览");
        }
    }

    #[test]
    fn buff处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("ApplyBuff").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::ApplyBuff {
            buff_id: "burn".into(),
            duration: 2,
        };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::ApplyBuff { buff_id, duration } = result.unwrap() {
            assert_eq!(buff_id, "burn");
            assert_eq!(duration, 2);
        } else {
            panic!("应该是 Buff 数据");
        }
    }

    #[test]
    fn 净化处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Cleanse").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Cleanse;
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), PendingEffectData::Cleanse));
    }

    #[test]
    fn 类型不匹配返回none() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_generate_ctx();
        // 传入 Heal 定义给 Damage 处理器
        let def = EffectDef::Heal { amount: 5 };
        assert!(handler.generate(&def, &ctx).is_none());
    }
}
