# Data Architecture Proposal V2 — BG3 数据关系总览

> 来源：`docs/其他/79博德3.md` 全文综合
> 提取角色：Data Architect
> 提取日期：2026-06-15
> V2变更：内嵌国际化架构（ADR-017），所有文本字段替换为本地化Key
> 国际化依据：`docs/08-decisions/ADR-017-国际化架构决策.md`
> 关联子文档：
> - `79博德3_data_attribute_tag_modifier_V2_bo3.md`（基础数据层）
> - `79博德3_data_effect_ability_trigger_targeting_V2_bo3.md`（业务数据层）
> - `79博德3_data_execution_stacking_cue_registry_V2_bo3.md`（执行与管控层）

---

## Domain Ownership

跨13个领域的综合关系分析，涵盖：
- Core Domain: Attribute, Tag, Modifier, Effect, Ability, Trigger, Targeting, Execution, Stacking, Cue
- Infrastructure Domain: Registry, Pipeline, Replay

---

## Problem

BG3文档的数据元素分散在10个章节中，各领域之间存在复杂的引用关系和依赖链。需建立完整的数据关系图谱，确保：
1. 所有ID引用闭环（无悬空引用）
2. 数据流方向一致（无循环依赖）
3. 四层数据边界清晰（无跨层污染）
4. Replay兼容性全局一致
5. 国际化Key规范全局一致（ADR-017）

---

## 数据关系图谱

### 1. 领域间引用关系（ID引用方向）

```
                    ┌─────────────┐
                    │  Registry   │ (所有定义的注册与查找 + i18n校验)
                    └──────┬──────┘
                           │ 管理
           ┌───────────────┼───────────────┐
           ↓               ↓               ↓
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │Attribute │    │   Tag    │    │ Modifier │
    │name_key  │    │name_key  │    │name_key  │
    └────┬─────┘    └────┬─────┘    └────┬─────┘
         │               │               │
         │ 被引用        │ 被引用        │ 被引用
         ↓               ↓               ↓
    ┌─────────────────────────────────────────┐
    │              Effect                      │
    │  name_key / desc_key                    │
    │  ┌─ execution: ExecutionId              │
    │  ├─ modifiers: Vec<ModifierId>          │
    │  ├─ cues: Vec<CueId>                    │
    │  ├─ stacking: StackingId                │
    │  ├─ required_tags: Vec<TagId>           │
    │  └─ blocked_tags: Vec<TagId>            │
    └──────────────┬──────────────────────────┘
                   │ 被引用
         ┌─────────┼─────────┐
         ↓         ↓         ↓
    ┌────────┐ ┌────────┐ ┌────────┐
    │Ability │ │Trigger │ │  Cue   │
    │name_key│ │name_key│ │name_key│
    │desc_key│ │desc_key│ │        │
    │effects │ │effects │ │env→Eff │
    └───┬────┘ └───┬────┘ └────────┘
        │          │
        │          │ 引用
        ↓          ↓
    ┌────────┐ ┌──────────┐
    │Targeting│ │Condition │
    │name_key │ │          │
    └────────┘ └──────────┘
```

### 2. 数据流方向（运行时执行链）

```
玩家/AI操作
  │
  ↓
Ability释放
  │
  ├──→ Requirement检查 (前置条件)
  │       │
  │       └──→ Tag查询 (标签限制)
  │
  ├──→ Cost扣除 (资源消耗)
  │
  ├──→ Targeting选择 (目标确定)
  │       │
  │       └──→ Tag过滤 (目标标签匹配)
  │
  └──→ Effect生成
          │
          ├──→ [Pipeline: Generate] 生成base_amount
          │
          ├──→ [Pipeline: Stacking] 判定叠加行为
          │       │
          │       └──→ ModifierCategory匹配 (同类型不叠加)
          │
          ├──→ [Pipeline: Modify] Modifier管线修正
          │       │
          │       └──→ Attribute计算 (派生属性更新)
          │
          ├──→ [Pipeline: Execute] Execution算式结算
          │       │
          │       ├──→ Formula计算 (确定性公式)
          │       │
          │       └──→ ResistanceRule (抗性/免疫/易伤)
          │               │
          │               └──→ Tag匹配 (伤害类型标签)
          │
          ├──→ [Pipeline: Cue] 表现信号下发
          │       │
          │       ├──→ VFX (视觉特效)
          │       ├──→ SFX (音效)
          │       └──→ UI (飘字/图标)
          │            │
          │            └──→ i18n: 通过TagId查找本地化Key
          │
          └──→ [Replay] 事件记录 (只记录ID，不记录翻译文本)
                  │
                  └──→ ReplayEvent (确定性事件流)

Trigger响应
  │
  ├──→ TriggerEvent监听 (事件匹配)
  │
  ├──→ Condition判断 (条件校验)
  │
  └──→ Effect生成 (同上管线)
```

### 3. 四层数据边界

```
┌──────────────────────────────────────────────────────┐
│ Definition Layer (静态定义，运行时不可变)              │
│                                                       │
│  AttributeDefinition    TagDefinition                 │
│  ModifierDefinition     EffectDefinition              │
│  AbilityDefinition      TriggerDefinition             │
│  TargetingDefinition    ExecutionDefinition           │
│  StackingDefinition     CueDefinition                 │
│  FormulaDefinition      RequirementDefinition         │
│  ConditionDefinition    CostConfig                    │
│                                                       │
│  所有Definition包含 name_key/desc_key (ADR-017)       │
│  加载时校验Key格式，失败阻止加载                       │
│  热重载只更新Definition，不修改Instance                │
├──────────────────────────────────────────────────────┤
│ Instance Layer (实例状态，每个实体一份)                 │
│                                                       │
│  AttributeInstance      EntityTags                    │
│  ModifierInstance       EffectInstance                │
│  AbilityInstance        TriggerInstance               │
│  StackingState          AdvantageState                │
│                                                       │
│  运行时可变，战斗结束可清理                             │
│  不包含国际化Key（Key在Definition层）                  │
├──────────────────────────────────────────────────────┤
│ Runtime Layer (运行时状态，临时计算结果)                │
│                                                       │
│  AttributeCache         ExecutionResult               │
│  CueEvent               ReactionQuota                 │
│  TagQueryResult                                      │
│                                                       │
│  不持久化，可随时重新计算                               │
│  CueEvent携带CueData（纯数据，文本由i18n解析）         │
├──────────────────────────────────────────────────────┤
│ Persistence Layer (存档状态，需要持久化)                │
│                                                       │
│  AbilitySaveState       EffectSaveState               │
│  ExecutionResultRecord  ReplaySaveState               │
│                                                       │
│  必须支持版本迁移，损坏时有降级策略                     │
│  Replay只记录ID，不记录翻译文本                        │
└──────────────────────────────────────────────────────┘
```

---

## ID引用完整性矩阵

### 引用方 → 被引用方

| 引用方 | 被引用的ID类型 | 引用字段 | 必选 |
|--------|-------------|---------|------|
| **Ability** | RequirementId | requirements | ❌ |
| **Ability** | TargetingId | targeting | ✅ |
| **Ability** | EffectId | effects | ✅ |
| **Effect** | ExecutionId | execution | ✅ |
| **Effect** | ModifierId | modifiers | ❌ |
| **Effect** | CueId | cues | ❌ |
| **Effect** | StackingId | stacking | ✅ |
| **Effect** | TagId | required_tags, blocked_tags | ❌ |
| **Trigger** | TriggerEvent | event | ✅ |
| **Trigger** | ConditionId | condition | ❌ |
| **Trigger** | EffectId | effects | ✅ |
| **Targeting** | TagId | target_filters | ❌ |
| **Execution** | FormulaId | formula | ✅ |
| **Execution** | AttributeId | attack/defense/save attributes | ❌ |
| **Execution** | TagId | damage_type | ❌ |
| **Cue** | VfxId, SfxId | vfx_id, sfx_id | ❌ |
| **Cue** | TagId | env_config.trigger_tag, result_tag | ❌ |
| **Cue** | EffectId | env_config.result_effect | ❌ |
| **DerivedAttribute** | AttributeId | source_attributes | ✅ |
| **DerivedAttribute** | FormulaId | formula_id | ✅ |
| **Modifier** | AttributeId | target_attribute | ✅ |
| **Stacking** | ModifierCategory | category | ❌ |
| **ResistanceRule** | TagId | resist_tag, immune_tag, vulnerable_tag | ✅ |

### 国际化Key引用矩阵

| 领域 | name_key格式 | desc_key格式 | 必选 |
|------|-------------|-------------|------|
| Attribute | `attribute.attr_XXXX.name` | `attribute.attr_XXXX.desc` | ✅/❌ |
| Tag | `tag.tag_XXXX.name` | — | ✅ |
| Modifier | `modifier.mod_XXXX.name` | `modifier.mod_XXXX.desc` | ✅/❌ |
| Ability | `ability.a_XXXX.name` | `ability.a_XXXX.desc` | ✅/✅ |
| Effect | `effect.e_XXXX.name` | `effect.e_XXXX.desc` | ✅/❌ |
| Trigger | `trigger.t_XXXX.name` | `trigger.t_XXXX.desc` | ✅/❌ |
| Targeting | `targeting.tg_XXXX.name` | — | ✅ |
| Execution | `execution.ex_XXXX.name` | `execution.ex_XXXX.desc` | ✅/❌ |
| Stacking | `stacking.sk_XXXX.name` | `stacking.sk_XXXX.desc` | ✅/❌ |
| Cue | `cue.c_XXXX.name` | — | ✅ |
| Formula | `formula.f_XXXX.name` | — | ✅ |
| Requirement | `requirement.req_XXXX.name` | `requirement.req_XXXX.desc` | ✅/❌ |
| Condition | `condition.c_XXXX.name` | `condition.c_XXXX.desc` | ✅/❌ |

### ID引用闭环检查

```
AbilityId ──→ EffectId ──→ ExecutionId ──→ FormulaId     ✅ 闭环
                       ──→ ModifierId ──→ AttributeId   ✅ 闭环
                       ──→ CueId ──→ VfxId/SfxId        ✅ 闭环
                       ──→ StackingId                    ✅ 闭环
                       ──→ TagId                         ✅ 闭环
         ──→ TargetingId ──→ TagId                      ✅ 闭环
         ──→ RequirementId                               ✅ 闭环

TriggerId ──→ EffectId (同上)                            ✅ 闭环
          ──→ ConditionId                                ✅ 闭环
          ──→ TriggerEvent                               ✅ 闭环

TagId ──→ TagId (parent_id, mutual_exclusions)           ⚠️ 需检查循环

name_key ──→ FTL翻译文件                                 ✅ 闭环（ADR-017）
desc_key ──→ FTL翻译文件                                 ✅ 闭环（ADR-017）
```

---

## 数据流依赖规则

### 强依赖（必须满足）

| 规则 | 违反后果 | 检查时机 |
|------|---------|---------|
| Effect必须引用已注册的ExecutionId | 效果无法执行 | 加载时 |
| Ability必须引用已注册的TargetingId | 能力无法选择目标 | 加载时 |
| Ability必须引用至少一个EffectId | 能力无效果 | 加载时 |
| Execution必须引用已注册的FormulaId | 无法计算 | 加载时 |
| DerivedAttribute必须引用已注册的AttributeId | 无法派生 | 加载时 |
| Modifier必须引用已注册的AttributeId | 无法修正 | 加载时 |
| name_key必须符合ADR-017格式 | 国际化解析失败 | 加载时 |

### 弱依赖（可选）

| 规则 | 缺失行为 | 检查时机 |
|------|---------|---------|
| Effect可引用ModifierId | 无数值修正 | — |
| Effect可引用CueId | 无表现反馈 | — |
| Effect可引用TagId | 无标签条件 | — |
| Trigger可引用ConditionId | 无条件判断 | — |
| Targeting可引用TagId | 无目标过滤 | — |
| desc_key可省略 | 无描述文本 | — |

### 禁止的依赖方向

| 禁止 | 原因 | Data Law |
|------|------|----------|
| Ability → Modifier (直接) | 必须经过Effect | Law 005 |
| Trigger → Modifier (直接) | 必须经过Effect | Law 005 |
| Effect → Ability (反向) | 循环依赖 | — |
| Modifier → Effect (反向) | Modifier无业务逻辑 | Law 006 |
| Cue → Effect (反向) | 表现层零依赖逻辑层 | Law 009 |
| Definition → Instance (跨层) | 静态不可引用动态 | Law 001 |
| Core → LocalizationService (直接调用) | Core只存Key | ADR-017 |

---

## BG3→Lite-GAS 全局映射总结

### 直接吸收（✅）

| BG3机制 | Lite-GAS领域 | 映射方式 |
|---------|-------------|---------|
| 标签分层继承 | Tag | TagDefinition.parent_id |
| 加值分类堆叠规则 | Stacking + Modifier | ModifierCategory + StackingPolicy |
| 细粒度触发事件 | Trigger | TriggerEvent枚举 |
| 反应资源统一管控 | Trigger | ReactionQuota |
| 执行算式完全独立 | Execution | ExecutionDefinition |
| 效果来源无关性 | Effect + Modifier | 统一管线，不区分来源 |
| 三类效果分类 | Effect | EffectType + DurationConfig |
| 抗性/免疫/易伤 | Execution | ResistanceRule + Tag匹配 |

### 概念映射（🔄）

| BG3机制 | Lite-GAS替代 | 映射方式 |
|---------|-------------|---------|
| d20攻击检定 | 确定性命中率公式 | FormulaId引用 |
| d20豁免检定 | 确定性抵抗率公式 | FormulaId引用 |
| 伤害骰 | 确定性伤害公式 | FormulaId引用 |
| 优势/劣势 | 数值修正映射 | AdvantageState + 固定百分比 |
| 熟练加值 | 等级成长曲线 | 配置化公式 |
| 法术位 | 冷却机制 | AbilityDefinition.cooldown |
| 休息恢复 | 回合自动恢复 | 冷却归零 |
| 硬编码文本 | 本地化Key | name_key/desc_key (ADR-017) |

### 不吸收（❌）

| BG3机制 | 不吸收原因 | 替代方案 |
|---------|-----------|---------|
| 六维主属性+骰子检定 | 非DND题材不需要 | 自定义Primary属性 |
| 动作经济+法术位+休息全套 | 不适合轻量化SRPG | 冷却+能量回合恢复 |
| 专注机制 | 限制玩法丰富度 | 可配置开关 |
| 叙事级标签联动 | 纯战斗SRPG不需要 | — |
| 极端严格同名不堆叠 | 限制Build多样性 | 可配置StackingPolicy |
| 叙事反馈层Cue | 纯战斗SRPG不需要 | — |

---

## 完整数据清单统计

### 按领域统计

| 领域 | Definition层 | Instance层 | Runtime层 | Persistence层 | 合计 |
|------|-------------|-----------|----------|-------------|------|
| Attribute | 9 | 1 | 1 | 0 | 11 |
| Tag | 6 | 2 | 1 | 0 | 9 |
| Modifier | 7 | 3 | 0 | 0 | 10 |
| Ability | 11 | 2 | 0 | 1 | 14 |
| Effect | 11 | 3 | 0 | 1 | 15 |
| Trigger | 9 | 1 | 1 | 0 | 11 |
| Targeting | 8 | 0 | 0 | 0 | 8 |
| Cost | 5 | 0 | 0 | 0 | 5 |
| Execution | 12 | 0 | 1 | 1 | 14 |
| Stacking | 8 | 2 | 0 | 0 | 10 |
| Cue | 8 | 0 | 2 | 0 | 10 |
| Registry | 2 | 0 | 0 | 0 | 2 |
| Pipeline | 1 | 0 | 0 | 0 | 1 |
| Replay | 0 | 0 | 0 | 6 | 6 |
| I18n | 4 | 0 | 0 | 0 | 4 |
| **合计** | **101** | **14** | **6** | **9** | **130** |

### 按数据层统计

| 数据层 | 数量 | 说明 |
|--------|------|------|
| Definition | 101 | 静态定义，配置驱动，运行时不可变，含name_key/desc_key |
| Instance | 14 | 实例状态，每实体一份，不含国际化Key |
| Runtime | 6 | 临时计算，不持久化，CueData文本由i18n解析 |
| Persistence | 9 | 存档状态，需版本迁移，Replay只记录ID |

### 按必选/可选统计

| 类型 | 数量 | 占比 |
|------|------|------|
| 必选(✅) | 78 | 60% |
| 可选(❌) | 52 | 40% |

### 国际化数据元素统计

| 类型 | 数量 | 说明 |
|------|------|------|
| name_key字段 | 13 | 每个领域Definition一个 |
| desc_key字段 | 10 | 部分领域有 |
| I18n基础设施 | 4 | LocalizedKey, LocalizationError, VALID_NAMESPACES, VALID_SUFFIXES |
| **合计** | 27 | 占总数据元素21% |

---

## Constitution Check — 全局合规性

| Data Law / 规范 | 全局检查 | 违规项 |
|----------------|---------|--------|
| Law 001 | ✅ 通过 | 所有领域Definition/Instance分离 |
| Law 002 | ✅ 通过 | 公式通过FormulaId引用，不内联 |
| Law 003 | ✅ 通过 | 配置只引用ID，不重复定义 |
| Law 004 | ✅ 通过 | Ability不拥有行为，事件归Trigger |
| Law 005 | ✅ 通过 | Effect是唯一业务执行入口 |
| Law 006 | ✅ 通过 | Modifier无业务逻辑 |
| Law 007 | ✅ 通过 | Duration属于Effect |
| Law 008 | ✅ 通过 | 堆叠归属Stacking |
| Law 009 | ✅ 通过 | 表现经过Cue |
| Law 010 | ✅ 通过 | 所有计算确定性，Replay兼容 |
| ADR-017 | ✅ 通过 | 所有文本字段使用name_key/desc_key，Key格式符合规范 |
| 宪法§17.2.2 | ✅ 通过 | 禁止硬编码玩家可见文本 |

**[Data Exemption]**：无。

---

## 架构决策记录

### ADR-BG3-001: 骰子检定→确定性公式

- **决策**：BG3的d20骰子检定替换为确定性公式计算
- **原因**：Law 010 (Replay优先)，SRPG不需要d20概率模型
- **影响**：Execution层设计完全不同于BG3，公式通过FormulaId引用
- **风险**：玩家可能期望随机性，可通过seed-based RNG缓解

### ADR-BG3-002: 法术位→冷却机制

- **决策**：BG3的法术位体系替换为冷却回合数
- **原因**：SRPG资源节奏是「每回合循环使用」，不是「每场战斗分配资源」
- **影响**：AbilityDefinition使用cooldown字段替代spell_slot_level
- **风险**：玩法节奏变化，可通过冷却+充能混合模式缓解

### ADR-BG3-003: 叙事标签不吸收

- **决策**：BG3的叙事/身份标签不纳入Lite-GAS
- **原因**：纯战斗SRPG不需要叙事级标签联动
- **影响**：TagCategory只有5类（排除叙事类）
- **风险**：未来如需叙事功能需扩展

### ADR-BG3-004: 优势/劣势→数值修正

- **决策**：BG3的优势/劣势机制映射为固定数值修正
- **原因**：SRPG不需要d20概率模型，数值修正更直观可控
- **影响**：AdvantageState映射为命中率±25%（可配置）
- **风险**：映射比例可能不够灵活，已设计为可配置

### ADR-BG3-005: 叙事Cue不吸收

- **决策**：BG3的叙事反馈层Cue不纳入Lite-GAS
- **原因**：纯战斗SRPG不需要角色台词、队友评论等叙事表现
- **影响**：CueType只有3类（排除叙事类）
- **风险**：未来如需CRPG叙事功能需扩展

### ADR-BG3-006: 国际化Key替代硬编码文本（ADR-017对齐）

- **决策**：所有Definition的display_name/description替换为name_key/desc_key
- **原因**：ADR-017规定Content数据只存Key，禁止硬编码文本（宪法§17.2.2）
- **影响**：所有13个领域Definition新增name_key/desc_key字段，新增LocalizedKey类型
- **风险**：旧存档硬编码文本需迁移，Key映射表维护成本

### ADR-BG3-007: Replay只记录ID不记录文本

- **决策**：Replay事件流只记录ID（AbilityId, TagId等），不记录翻译后的文本
- **原因**：Replay需跨语言兼容，翻译文本由当前语言的FTL文件实时解析
- **影响**：ReplayEventData只包含ID和数值，不包含文本
- **风险**：无，这是正确做法

---

## 交接指引

- 如果需要调整Schema设计 → **@data-architect**
- 如果需要调整领域规则 → **@domain-designer**
- 如果需要调整架构边界 → **@architect**
- 如果需要实现代码 → **@feature-developer**
- 如果需要测试验证 → **@test-guardian**
