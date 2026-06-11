# Test Specification

Version: 1.0

测试圣经——AI 修改测试或业务逻辑前必须先读此文件。

---

# Truth Source 优先级

```
Domain Rules > Test Spec > Tests > Implementation
```

测试失败时，禁止直接修改代码。必须判断：

| 判定 | 输出格式 | 处理方式 |
|------|----------|----------|
| 业务规则错误 | `Root Cause: Business Rule Violation` | 修改 Domain Rules，同步更新测试 |
| 测试规范错误 | `Root Cause: Test Specification Violation` | 修改 Test Spec，同步更新测试 |
| 实现错误 | `Root Cause: Implementation Bug` | 修复实现，不修改测试 |

---

# Test Pyramid

| 层级 | 比例 | 验证目标 | 示例 |
|------|------|----------|------|
| Unit Test | 70% | 单个领域规则 | DamageFormula、AttributeModifier、BuffDuration |
| Integration Test | 20% | 多 Feature 协作 | Equipment→Stats、Skill→Buff→Damage |
| Scenario Test | 10% | 完整玩家流程 | 战斗回合循环、击杀→死亡→胜负 |

---

# Forbidden

禁止：
- 修改测试让错误逻辑通过
- 修改逻辑让错误测试通过
- 删除失败测试
- 测试断言实现细节而非领域规则
- 为了覆盖率数字而写测试

判定标准：
- 测试在领域规则成立时必须通过
- 测试在领域规则被违反时必须失败
- 实现细节变化但领域规则不变时，测试不应受影响

---

# Required Output

修改测试时必须说明：
- 原因（Business Rule / Test Spec / Implementation 哪个变了）
- 影响范围
- 风险评估

修改业务逻辑时必须说明：
- 原因
- 影响范围
- 风险评估

---

# Unit Test（70%）

验证单个领域规则，不依赖 ECS World。

## 1. 属性系统规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-ATTR-01 | 衍生属性实时计算 | `MaxHp = 5 + Vitality * 5`，修改 Vitality 后 MaxHp 立即变化 | stat_system §3 |
| U-ATTR-02 | 衍生属性不可设置基础值 | `set_base(MaxHp, 100)` 被 warn 并忽略 | stat_system §5.3 |
| U-ATTR-03 | 修饰符先加后乘 | `(base + Add) × Multiply`，顺序固定 | stat_system §4.1 |
| U-ATTR-04 | 乘法零值保护 | 乘法修饰符乘积为 0 时视为 1.0 | stat_system §4.1 |
| U-ATTR-05 | ModifierSource 区间隔离 | Trait/Buff/Equipment 区间不冲突，可按来源精确移除 | stat_system §4.2 |
| U-ATTR-06 | 减益判定统一 | `Add<0` 或 `Multiply<1.0` 为减益 | stat_system §4.4 |
| U-ATTR-07 | fill_vital_resources 初始化 | HP/MP/Stamina 初始化为最大值 | stat_system §5.3 |
| U-ATTR-08 | set_vital 仅限 HP/MP/Stamina | 对非生命资源调用 warn 并忽略 | stat_system §5.3 |
| U-ATTR-09 | AttributeKind 分类互斥 | 每个 kind 恰好属于 core/vital/derived 之一 | stat_system §2.2 |

## 2. 伤害公式规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-DMG-01 | 伤害最低为 1 | 任意输入 `calculate_damage_from_effect` 结果 ≥ 1 | battle_rules §5.1, effect_pipeline §6.3 |
| U-DMG-02 | 无视防御基于基础防御 | `ignore_def_percent` 基于 Vitality 而非修饰后 Defense | effect_pipeline §6.3 |
| U-DMG-03 | 地形防御加成减少伤害 | terrain_bonus > 0 时伤害降低 | battle_rules §3.3 |
| U-DMG-04 | 倍率越高伤害越高 | multiplier=2.0 伤害 ≥ multiplier=1.0 伤害 | effect_pipeline §6.3 |
| U-DMG-05 | 无视防御比例越高伤害越高 | ignore_def=0.5 伤害 ≥ ignore_def=0.0 伤害 | effect_pipeline §6.3 |

## 3. 修饰规则规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-MOD-01 | 双标签匹配 | source_tag 和 target_tag 必须同时满足 | modifier_rules §4.3 |
| U-MOD-02 | 链式修饰 | 多规则按顺序叠加 | modifier_rules §7.3 |
| U-MOD-03 | 伤害修饰下限为 1 | `apply_damage_modifiers` 结果 ≥ 1 | modifier_rules §7.2 |
| U-MOD-04 | 治疗修饰下限为 0 | `apply_heal_modifiers` 结果 ≥ 0 | modifier_rules §7.2 |
| U-MOD-05 | ModifierEntry 记录每步 | breakdown 记录 before/after/rule_name | modifier_rules §5 |
| U-MOD-06 | Calculator trait 分发 | 新增计算器只需实现 trait 并注册 | modifier_rules §3 |

## 4. Buff 规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-BUFF-01 | Cleanse 不创建实例 | 净化 Buff 立即驱散所有 Debuff，返回 BuffInstanceId(0) | buff_rules §7.1 |
| U-BUFF-02 | 同源刷新不新增 | 同 source_entity + 同 buff_id 只刷新 remaining_turns | buff_rules §6.3 |
| U-BUFF-03 | 不同源可共存 | 不同来源的同 ID Buff 各自独立 | buff_rules §6.3 |
| U-BUFF-04 | 晕眩消耗后移除 | consume_stun() 移除所有 STUN 标签 Buff | buff_rules §6.2 |
| U-BUFF-05 | tick 两步走 | 先移除 remaining=0 的，再递减所有 | buff_rules §8.2 |
| U-BUFF-06 | 标签安全移除 | 移除 Buff 时检查共享标签，其他 Buff 仍提供的标签不删除 | buff_rules §7.2 |
| U-BUFF-07 | Buff 必须有来源 | BuffInstance.source_entity 记录来源 | buff_rules §5 |
| U-BUFF-08 | Buff 必须有过期条件 | remaining_turns 有限，回合结束递减 | buff_rules §8.1 |

## 5. 技能规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-SKILL-01 | can_use 是纯函数 | 不修改任何状态 | skill_rules §3.3 |
| U-SKILL-02 | 冷却检查优先 | current_cooldown > 0 → OnCooldown | skill_rules §3.3 |
| U-SKILL-03 | 冷却 tick 自动清理 | 归零后从 HashMap 移除 | skill_rules §5 |
| U-SKILL-04 | range=0 使用基础范围 | effective_skill_range 回退 base_attack_range | skill_rules §4.1 |
| U-SKILL-05 | 预览不修改状态 | preview_skill_effects 是纯函数 | skill_rules §7.4 |
| U-SKILL-06 | TargetRequireTag 无目标跳过 | target_tags 为 None 时不检查 | skill_rules §3.3 |

## 6. 装备规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-EQUIP-01 | 需求检查先于穿戴 | 不满足需求返回 Failed，不执行穿戴 | equipment_rules §5 |
| U-EQUIP-02 | 替换旧装备 | 槽位已占用时先脱卸再穿戴 | equipment_rules §6.1 |
| U-EQUIP-03 | 修饰符来源追踪 | ModifierSource::equipment_source(instance_id) 精确关联 | equipment_rules §7.2 |
| U-EQUIP-04 | 标签分层管理 | 装备标签存入 from_equipment，不与 Buff 混淆 | equipment_rules §7.3 |
| U-EQUIP-05 | 注册表幂等 | 重复调用 register_defaults 不重复注册 | equipment_rules §9.2 |

## 7. 背包规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-INV-01 | 堆叠合并条件严格 | def_id + bind + enhance_level + enchantments 全部匹配 | inventory_rules §6.1 |
| U-INV-02 | 容量与重量双限制 | add_stack 逐步检查，部分成功返回已添加数量 | inventory_rules §7.3 |
| U-INV-03 | 消耗品仅 Consumable 可使用 | Equipment 类型忽略 UseItem | inventory_rules §9.2 |
| U-INV-04 | 空堆叠自动清理 | count=0 时自动移除 | inventory_rules §7.3 |
| U-INV-05 | 转移校验容量 | 目标满/超重时返回 Full/Overweight | inventory_rules §10.2 |

## 8. 标签系统规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-TAG-01 | 标签位运算幂等 | `tags | tag | tag == tags | tag` | domain_rules §GameplayTag |
| U-TAG-02 | 标签三层架构 | Trait > Equipment > Buff 按优先级重建 | buff_rules §8.3 |

## 9. 寻路规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-PATH-01 | BFS 可达范围 | find_reachable_tiles 返回正确可达集合 | map_rules §7.4 |
| U-PATH-02 | 地形成本计算器 | Ground/Flying/Mounted/Swimming 各自正确 | map_rules §7.2 |
| U-PATH-03 | 标签解析优先级 | SWIMMING > FLYING > MOUNTED > ground | map_rules §7.3 |
| U-PATH-04 | 占用排除自身 | 寻路时自身位置不算被占用 | map_rules §7.4 |

## 10. Trait 规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-TRAIT-01 | Passive 效果仅 GrantTag 和 ModifyAttribute | ApplyBuff 在 Passive 下无意义 | trait_rules §3.1 |
| U-TRAIT-02 | 触发型效果仅 ApplyBuff | GrantTag/ModifyAttribute 不需要触发 | trait_rules §3.1 |
| U-TRAIT-03 | Handler 分发扩展 | 新增效果类型只需实现 Handler 并注册 | trait_rules §4 |
| U-TRAIT-04 | 来源精确追踪 | Intrinsic 和 Equipment { slot } 区分来源 | trait_rules §6 |
| U-TRAIT-05 | 修饰符独立 source | 每个 Trait 分配独立 ModifierSource | trait_rules §8 |

## 11. AI 规则

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| U-AI-01 | 策略 trait 替代 enum+match | 新增策略只需实现 trait 并注册 | ai_rules §4 |
| U-AI-02 | 注册表回退机制 | 未知策略名称回退到默认策略 | ai_rules §5.1 |
| U-AI-03 | CombatIntent 统一处理 | AI 和玩家共用 Effect Pipeline | ai_rules §7.2 |

---

# Integration Test（20%）

验证多 Feature 协作，在 ECS App 中运行。

## 1. 装备→属性→标签

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-EQ-ATTR-01 | 穿戴装备增加属性 | 装备铁剑后 Attack 增加 | equipment_rules §7 |
| I-EQ-ATTR-02 | 脱卸装备移除属性 | 脱下铁剑后 Attack 恢复 | equipment_rules §6.2 |
| I-EQ-ATTR-03 | 穿戴装备授予标签 | 装备炎龙长剑后获得 FIRE 标签 | equipment_rules §7.3 |
| I-EQ-ATTR-04 | 穿戴装备授予 Trait | 装备炎龙长剑后 TraitCollection 包含 flaming_weapon | equipment_rules §7.1 |
| I-EQ-ATTR-05 | 脱卸装备移除 Trait | 脱下后 TraitCollection 不再包含 | equipment_rules §6.2 |
| I-EQ-ATTR-06 | 穿脱后 Trait 重建 | 装备穿脱后 Passive Trait 效果正确重建 | equipment_rules §8 |
| I-EQ-ATTR-07 | 穿脱后 GameplayTags 重建 | 标签与装备状态一致 | equipment_rules §8.2 |

## 2. 技能→效果管线→伤害

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-SK-BAT-01 | 技能走三步管线 | Generate→Modify→Execute 完整执行 | battle_rules §2 |
| I-SK-BAT-02 | 修饰规则在 Modify 阶段应用 | source_tag + target_tag 匹配时伤害被修饰 | battle_rules §4 |
| I-SK-BAT-03 | 伤害执行扣血 | Execute 后目标 HP 减少 | battle_rules §5.1 |
| I-SK-BAT-04 | 伤害执行发送 Message | DamageApplied Message 包含正确信息 | battle_rules §9 |
| I-SK-BAT-05 | 治疗不超过 MaxHp | 回血后 HP ≤ MaxHp | battle_rules §5.2 |

## 3. Buff→属性→结算

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-BUF-ATTR-01 | Buff 施加增加属性 | 攻+5 Buff 后 Attack 增加 | buff_rules §7.1 |
| I-BUF-ATTR-02 | Buff 移除恢复属性 | Buff 过期后 Attack 恢复 | buff_rules §7.2 |
| I-BUF-ATTR-03 | DoT 结算扣血 | 每回合 DoT 伤害正确扣血 | buff_rules §8.1 |
| I-BUF-ATTR-04 | HoT 结算回血 | 每回合 HoT 治疗正确回血 | buff_rules §8.1 |
| I-BUF-ATTR-05 | DoT 可致死 | DoT 伤害可导致角色死亡 | buff_rules §8.1 |
| I-BUF-ATTR-06 | 晕眩跳过行动 | 晕眩单位标记已行动 | buff_rules §8.1 |

## 4. 死亡→消息→清理

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-DEATH-01 | HP≤0 插入 Dead 组件 | 致命伤害后实体有 Dead 组件 | battle_rules §5.1 |
| I-DEATH-02 | Dead Hook 标记已行动 | 添加 Dead 后 Unit.acted = true | character_rules §2.6 |
| I-DEATH-03 | Dead Hook 移除选中 | 添加 Dead 后无 Selected 组件 | character_rules §2.6 |
| I-DEATH-04 | CharacterDied Message | 死亡后发送 CharacterDied | battle_rules §9 |
| I-DEATH-05 | 死亡单位从队列移除 | TurnOrder 不再包含死亡实体 | battle_rules §9.2 |

## 5. 消耗品→效果

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-CONS-01 | 消耗品恢复 HP | 使用治疗药水后 HP 增加 | inventory_rules §9 |
| I-CONS-02 | 消耗品施加 Buff | 使用 Buff 药水后 ActiveBuffs 增加 | inventory_rules §9 |
| I-CONS-03 | 使用后堆叠减少 | 使用后 count - 1 | inventory_rules §9 |

## 6. Trait 触发→效果管线

| ID | 规则 | 测试内容 | 对应 Domain Rule |
|----|------|----------|------------------|
| I-TRAIT-TRIG-01 | OnAttack 触发施加 Buff | 攻击时 ApplyBuff 效果推入 EffectQueue | battle_rules §8 |
| I-TRAIT-TRIG-02 | OnKill 触发 | 击杀时触发 Trait 效果 | battle_rules §8 |
| I-TRAIT-TRIG-03 | 仅 ApplyBuff 效果被触发 | GrantTag/ModifyAttribute 不在触发器中处理 | battle_rules §8.2 |

---

# Scenario Test（10%）

验证完整玩家流程，Given-When-Then 风格。

## 1. 完整战斗回合

| ID | 场景 | Given | When | Then | 对应 Domain Rule |
|----|------|-------|------|------|------------------|
| S-BATTLE-01 | 基础战斗 | 战士 vs 哥布林 | 战士攻击哥布林 | 哥布林 HP 减少，DamageApplied Message 发出 | battle_rules 全流程 |
| S-BATTLE-02 | 致命伤害 | 战士 vs 低 HP 敌人 | 攻击致死 | Dead 组件插入，CharacterDied 发出，队列移除 | battle_rules §5.1 |
| S-BATTLE-03 | 治疗战斗 | 受伤角色 + 治疗者 | 治疗者使用治疗技能 | HP 恢复，不超过 MaxHp | battle_rules §5.2 |
| S-BATTLE-04 | 回合结束→新回合 | 所有单位已行动 | 队列耗尽 | TurnEnded 发出，acted 重置，新 TurnOrder 重建 | turn_rules §8.2 |

## 2. Buff 生命周期

| ID | 场景 | Given | When | Then | 对应 Domain Rule |
|----|------|-------|------|------|------------------|
| S-BUFF-01 | DoT 战斗 | 中毒单位 | 每回合结算 | DoT 扣血，tick 递减，过期后移除 | buff_rules §8 |
| S-BUFF-02 | 晕眩跳过 | 晕眩单位 | 回合开始 | 标记已行动，消耗后移除晕眩 | buff_rules §8.1 |
| S-BUFF-03 | Cleanse 驱散 | 多 Debuff 单位 | 使用净化 | 所有 Debuff 移除 | buff_rules §7.1 |

## 3. 地形影响战斗

| ID | 场景 | Given | When | Then | 对应 Domain Rule |
|----|------|-------|------|------|------------------|
| S-TERRAIN-01 | 森林防御加成 | 目标在森林 | 攻击 | 伤害比平地低 | battle_rules §3.3, map_rules §2 |

## 4. 装备穿脱完整流程

| ID | 场景 | Given | When | Then | 对应 Domain Rule |
|----|------|-------|------|------|------------------|
| S-EQUIP-01 | 穿戴→属性→标签→Trait | 无装备角色 | 穿戴炎龙长剑 | Attack 增加，FIRE 标签获得，flaming_weapon Trait 激活 | equipment_rules §6.1 |
| S-EQUIP-02 | 脱卸→属性恢复→标签移除 | 装备铁剑角色 | 脱下铁剑 | Attack 恢复，SWORD 标签移除 | equipment_rules §6.2 |

## 5. AI 决策流程

| ID | 场景 | Given | When | Then | 对应 Domain Rule |
|----|------|-------|------|------|------------------|
| S-AI-01 | AI 攻击流程 | 敌方单位 + 玩家单位 | AI 回合 | 设置 CombatIntent → Effect Pipeline 执行 | ai_rules §7 |
| S-AI-02 | AI 技能选择 | 敌方有特殊技能 | AI 回合 | 优先使用特殊技能（PreferSpecial） | ai_rules §4.3 |

---

# Golden Test（Battle Replay）

快照测试验证战斗状态流，保证确定性。

| ID | 场景 | 快照内容 |
|----|------|----------|
| G-01 | 基础战斗：战士攻击哥布林 | 伤害值、HP 变化、Message 序列 |
| G-02 | 治疗战斗：角色受伤后治疗 | 治疗量、HP 恢复 |
| G-03 | 致命伤害：角色死亡 | Dead 组件、CharacterDied |

快照文件路径：`tests/golden/snapshots/`

快照变更时必须人工审查，确认是预期行为变更而非回归。

---

# 测试文件组织

```
tests/
├── common/              # 测试辅助
│   ├── app_builder.rs   # App 构建器（combat_app / equipment_app / full_battle_app）
│   ├── assertions.rs    # 自定义断言
│   ├── combat_helpers.rs # 战斗辅助函数
│   └── fixtures.rs      # UnitBuilder 等测试夹具
├── rule/                # Unit Test — 属性测试（proptest）
│   └── rules.rs
├── feature/             # Integration Test — Feature 协作
│   ├── buff.rs
│   ├── consumable.rs
│   ├── death.rs
│   ├── equipment.rs
│   ├── inventory.rs
│   ├── skill.rs
│   ├── traits.rs
│   └── turn.rs
├── scenario/            # Scenario Test — 完整流程
│   └── scenarios.rs
├── system/              # System Test — 单系统 ECS 行为
│   └── systems.rs
├── golden/              # Golden Test — 快照对比
│   ├── golden_battle.rs
│   └── snapshots/
└── legacy/              # 遗留测试（待迁移）
    ├── buff_damage.rs
    ├── buff_lifecycle.rs
    ├── combat_pipeline.rs
    ├── edge_cases.rs
    ├── skill_system.rs
    ├── terrain_combat.rs
    └── turn_flow.rs
```

### 内联单元测试

模块内部的 `#[cfg(test)] mod tests` 验证纯函数和结构体方法，与 `tests/` 目录的外部测试互补。

---

# 现有测试覆盖评估

## 已覆盖的领域规则

| 领域 | 覆盖状态 | 说明 |
|------|----------|------|
| 伤害公式 | ✅ 良好 | proptest 属性测试 + 场景测试 |
| 属性修饰符 | ✅ 良好 | proptest + 内联测试 |
| Buff 生命周期 | ✅ 良好 | legacy + feature + scenario |
| 装备穿脱 | ✅ 良好 | feature + system 测试 |
| 背包操作 | ✅ 良好 | feature + 内联测试 |
| 死亡流程 | ✅ 良好 | feature + scenario |
| 效果管线 | ✅ 良好 | legacy + system + scenario |
| 回合流转 | ✅ 良好 | legacy + feature |
| 技能系统 | ✅ 良好 | legacy + feature |
| Trait 触发 | ✅ 良好 | feature + system |
| 消耗品 | ✅ 良好 | feature + system |
| 修饰规则 | ✅ 良好 | 内联 + proptest |
| 标签位运算 | ✅ 良好 | proptest + 内联 |

## 覆盖不足的领域规则

| 领域 | 缺口 | 建议补充 |
|------|------|----------|
| AI 策略选择 | 缺少 TargetSelector/MoveSelector/SkillSelector 的单元测试 | 补充 U-AI-01/02/03 |
| 寻路算法 | 缺少 BFS 边界条件测试（空地图、单格、不可通行环绕） | 补充 U-PATH-01/02/03/04 |
| 回合状态机 | 缺少 TurnPhase 转换的完整性测试 | 补充 S-BATTLE-04 |
| DamageBreakdown | 缺少 generate→modify→execute 全链路明细验证 | 补充 I-SK-BAT-02 的 breakdown 验证 |
| 标签三层重建 | 缺少 Trait→Equipment→Buff 三层叠加后完整重建的测试 | 补充 I-BUF-ATTR-06 的标签重建验证 |
| 装备需求检查 | 缺少 RequireTag / AttributeMin 的否定测试 | 补充 U-EQUIP-01 的失败路径 |
| 战场背包 | 缺少战斗开始复制/战斗结束归还的完整测试 | 补充集成测试 |

---

# 测试编写规范

## 命名

- Unit Test：`test_{规则简称}_{场景}`，如 `test_damage_always_at_least_1`
- Integration Test：`test_{feature}_{scenario}`，如 `test_equip_adds_modifier`
- Scenario Test：Given-When-Then 注释 + `test_{scenario_name}`
- Golden Test：中文描述场景名

## 断言原则

正确：
```rust
// 验证领域规则：伤害最低为 1
assert!(damage >= 1);
// 验证领域规则：Buff 过期后属性恢复
assert_eq!(attrs.get(Attack), base_attack);
```

错误：
```rust
// 验证实现细节：修饰符数量
assert_eq!(attrs.modifiers.len(), 3);
// 验证实现细节：HashMap key 存在
assert!(attrs.base.contains_key(&AttributeKind::MaxHp));
```

## 测试隔离

- 每个测试创建独立的 App 实例
- 不依赖文件系统（使用 register_defaults 而非 RON 加载）
- 不依赖执行顺序
- 不共享可变状态

## proptest 使用

- 伤害公式、属性计算、标签位运算使用 proptest
- 参数范围覆盖边界值（0、负数、极大值）
- 避免浮点精确比较，使用 `>=` / `<=` / 近似比较

---

# 停止条件

发现以下情况，必须停止并报告：

1. 测试断言实现细节而非领域规则
2. 测试在领域规则被违反时仍然通过
3. 测试在领域规则成立时仍然失败
4. 修改业务逻辑让错误测试通过
5. 修改测试让错误业务逻辑通过
6. 删除失败测试来消除失败
7. 测试依赖其他测试的执行顺序
8. 测试依赖文件系统中的特定文件
