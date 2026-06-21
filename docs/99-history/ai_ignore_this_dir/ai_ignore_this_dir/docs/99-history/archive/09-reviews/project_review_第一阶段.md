# 项目评审报告（基于 23.md 优先级框架）

> 评审日期：2026-06-13
> 评审依据：`docs/其他/23.md` — 七优先级发展路线

---

## 总体结论

**项目成熟度远超 23.md 的假设。** 23.md 假设项目处于"系统框架基本齐全，但还没形成完整可玩的战斗循环"阶段，实际上大部分优先级已经完成或接近完成。

---

## 逐项评审

### 第一优先级：打通完整战斗闭环 ✅ 已完成

23.md 要求的完整流程：

| 步骤 | 状态 | 实现位置 |
|------|------|----------|
| 进入战斗 | ✅ | `AppState::InGame` + 地图加载 |
| 轮到角色 | ✅ | `TurnOrder` 按 Initiative 排序，`TurnPhase::SelectUnit` |
| 移动 | ✅ | `find_reachable_tiles` BFS 寻路 + `MovementIntent` 消息 |
| 选择技能 | ✅ | `SkillSlots` + `SkillRegistry` + UI 技能面板 |
| 选择目标 | ✅ | `TurnPhase::SelectTarget` |
| 造成伤害 | ✅ | Effect Pipeline: `generate → modify → execute` |
| Buff 生效 | ✅ | `resolve_status_effects` 在 `SelectUnit` 阶段结算 DoT/HoT/Stun |
| 结束回合 | ✅ | `TurnPhase::TurnEnd` → 重建队列 → 回合数+1 |
| 敌人行动 | ✅ | `enemy_ai_system` 决策 → 移动 → 攻击 |
| 战斗胜利/失败 | ✅ | `GameOverState` (Victory/Defeat) + `AppState::GameOver` |

**评价：** 战斗闭环已完整实现。状态机 `TurnPhase` 覆盖了完整的回合生命周期，Effect Pipeline 保证了伤害计算的规范性。

**唯一缺口：** `assets/maps/tutorial.ron` 中没有 `victory_condition` 字段，胜负判定目前是硬编码逻辑（全灭敌人=胜利，全灭玩家=失败）。建议在地图配置中增加胜利条件定义。

---

### 第二优先级：Character → Skill → Buff 联动 ✅ 已完成

三者连接关系：

```
Character
 ├─ Attributes (核心属性)
 ├─ SkillSlots (技能槽位)
 ├─ SkillCooldowns (冷却)
 └─ ActiveBuffs (活跃 Buff)
```

技能 → Buff 示例（`fireball.ron`）：

```ron
effects: [
    Damage(multiplier: 1.8, ignore_def_percent: 0.0),
    ApplyBuff(buff_id: "burn", duration: 2),
]
```

Buff 结算流程：

```
TurnPhase::SelectUnit → resolve_status_effects
  ├─ DoT: burn (dot_damage: 2)
  ├─ HoT: regen (hot_heal: 4)
  └─ Stun: stun (is_stun: true)
```

**评价：** Character-Skill-Buff 三者已通过 Effect Pipeline 和 Buff Resolve 系统完整连接。技能释放 → 附加 Buff → 持续效果结算的链路已打通。

---

### 第三优先级：数据驱动 ✅ 已完成

| 数据类型 | 文件格式 | 文件数 | 示例 |
|----------|----------|--------|------|
| 技能 | `assets/skills/*.ron` | 6 | fireball, heal, charge, pierce, basic_attack, cleanse_skill |
| Buff | `assets/buffs/*.ron` | 8 | burn, poison, regen, stun, attack_up/down, defense_up/down |
| 角色模板 | `assets/units/*.ron` | 5 | player_warrior, player_mage, player_archer, enemy_goblin, enemy_dark_knight |
| 地图 | `assets/maps/*.ron` | 1 | tutorial |
| 地形 | `assets/terrains/*.ron` | 4 | plain, mountain, water, forest |
| AI 行为 | `assets/ai/*.ron` | 3 | default, aggressive, cautious |
| 特性 | `assets/traits/*.ron` | 4 | warrior_mastery, mage_mastery, archer_mastery, fire_affinity, heavy_armor |

**评价：** 数据驱动架构已全面建立。所有核心游戏数据均通过 RON 文件外部化，使用 `RegistryLoader` trait 统一加载。新增内容只需添加 RON 文件，无需修改代码。

**23.md 中"不要再写 `if skill_id == 1`"的警告——项目已完全遵守。**

---

### 第四优先级：AI 行动系统 ✅ 已完成（且超出预期）

23.md 建议"最简单版本：攻击范围内有敌人→攻击，否则→靠近最近敌人"。

实际实现（`src/ai/decision.rs`）：

```
enemy_ai_system
  ├─ 行为配置读取 (AiBehaviorRegistry)
  ├─ 目标选择策略 (AiStrategyRegistry)
  │   ├─ 最近敌人
  │   ├─ 最弱敌人
  │   └─ 最危险敌人
  ├─ 移动策略
  │   ├─ 激进 (aggressive)
  │   ├─ 谨慎 (cautious)
  │   └─ 默认
  ├─ 技能选择策略
  │   └─ 基于优先级的技能选择
  └─ 设置 CombatIntent → Effect Pipeline 执行
```

**评价：** AI 系统远超"最简单版本"。已实现策略模式的 AI 框架，支持多种移动/技能/目标选择策略，通过 RON 配置文件驱动。这是项目的一个显著亮点。

**注意：** 23.md 建议"不要上来 GOAP/BT/Utility AI"，项目使用的是自定义策略注册表，不是复杂的 AI 框架，符合建议。

---

### 第五优先级：胜负系统 ✅ 已完成

实现位置：

- `src/ui/view_models.rs`: `GameOverState` 枚举 (Playing/Victory/Defeat)
- `src/ui/view_models.rs`: `update_game_over_state()` 检查胜负条件
- `src/ui/panels/turn_indicator.rs`: 读取 `GameOverState`，触发 `AppState::GameOver`
- `src/turn/state.rs`: `AppState::GameOver` 状态定义

判定逻辑：

```rust
// 所有敌人死亡 → Victory
// 所有玩家死亡 → Defeat
```

**评价：** 胜负系统已实现。全灭敌人→胜利，全灭玩家→失败，状态切换到 `GameOver`。

**建议增加：** 撤退（Retreat）选项、地图配置中的胜利条件（kill_all / survive N turns / escort 等）。

---

### 第六优先级：战斗数据记录 ✅ 已完成（且超出预期）

23.md 要求的 `BattleLog`：

实际实现（`src/battle/record.rs`）：`BattleRecord` 资源，远超简单的文本日志。

| 功能 | 状态 | 说明 |
|------|------|------|
| 结构化记录 | ✅ | `BattleEntry` 枚举，覆盖所有战斗事件 |
| 伤害分解 | ✅ | `DamageBreakdown`（base → modified → actual） |
| 实体统计 | ✅ | `EntityBattleStats`（伤害输出/承伤/治疗/击杀） |
| 按回合查询 | ✅ | `entries_for_turn(turn)` |
| 按实体查询 | ✅ | `entries_for(entity)` |
| 序列化 | ✅ | 支持 RON 序列化/反序列化，可用于回放 |
| 调试面板 | ✅ | `BattleRecord` 注册为 Reflect，可在 World Inspector 中查看 |

**评价：** 战斗记录系统设计精良，为未来的回放系统、战斗分析、测试验证奠定了坚实基础。

---

### 第七优先级：地图内容编辑 ✅ 已完成（基础版）

`assets/maps/tutorial.ron`：

```ron
(
    id: "tutorial",
    name: "教学关",
    width: 10,
    height: 8,
    terrain_grid: ["MMMMMMMMMM", ...],
    player_units: [
        (template: "player_warrior", coord: (4, 3)),
        ...
    ],
    enemy_units: [
        (template: "enemy_goblin", coord: (7, 5)),
        ...
    ],
)
```

**已实现：**
- 地图尺寸定义
- 地形网格配置
- 玩家单位出生点
- 敌方单位配置

**未实现：**
- 胜利条件（victory_condition）
- 回合限制（turn_limit）
- 奖励配置（rewards）
- 地形效果（terrain_bonuses）

**评价：** 基础的地图数据驱动已建立，但缺少关卡设计所需的元数据。新增关卡 = 新增 RON 文件，符合 23.md 的"新增数据 = 新增关卡"目标。

---

## 暂时不要做的东西（23.md 清单）

| 功能 | 状态 | 建议 |
|------|------|------|
| 剧情系统 | ❌ 未做 | 正确，暂不需要 |
| 任务系统 | ❌ 未做 | 正确，暂不需要 |
| 存档系统 | ❌ 未做 | 正确，暂不需要 |
| 装备词条 | ❌ 未做 | 正确，暂不需要 |
| 锻造系统 | ❌ 未做 | 正确，暂不需要 |
| 抽卡系统 | ❌ 未做 | 正确，暂不需要 |
| 复杂AI | ✅ 已做 | 项目已有策略模式 AI，但未过度复杂化 |
| 联机 | ❌ 未做 | 正确，暂不需要 |
| 编辑器 | ❌ 未做 | 正确，暂不需要 |
| 动画状态机优化 | ❌ 未做 | 正确，暂不需要 |
| 性能优化 | ❌ 未做 | 正确，暂不需要 |

**评价：** 项目严格遵守了"不做暂不需要的功能"的原则。唯一例外是 AI 系统，但其实现适度，没有过度工程化。

---

## 23.md 建议路线 vs 实际进度

| 23.md 建议 | 实际状态 | 差距 |
|------------|----------|------|
| 1. 移动 + 攻击 + 死亡 | ✅ 完成 | 无 |
| 2. 回合切换 | ✅ 完成 | 无 |
| 3. Skill 系统接入 | ✅ 完成 | 无 |
| 4. Buff 系统接入 | ✅ 完成 | 无 |
| 5. 胜负判定 | ✅ 完成 | 无 |
| 6. 简单 AI | ✅ 完成（超出预期） | AI 已有策略模式 |
| 7. BattleLog | ✅ 完成（超出预期） | 已有结构化 BattleRecord |
| 8. 数据驱动 Skill/Buff | ✅ 完成 | 无 |
| 9. 数据驱动角色 | ✅ 完成 | 无 |
| 10. 数据驱动关卡 | ⚠️ 基础完成 | 缺少胜利条件等元数据 |
| 11. 美术资源替换 | ❌ 未做 | 正确，暂不需要 |
| 12. 第一张完整战斗关卡 | ⚠️ 基础完成 | tutorial.ron 存在但缺元数据 |

---

## 下一步建议（基于项目实际状态）

项目已完成 23.md 的大部分优先级。基于当前状态，建议的下一步：

### 短期（1-2 周）
1. **完善关卡配置**：在 `assets/maps/*.ron` 中增加 `victory_condition`、`turn_limit`、`rewards` 字段
2. **BattleLog UI**：将 `BattleRecord` 在调试面板中可视化展示（已有基础，只需 UI 渲染）
3. **关卡选择**：实现简单的关卡选择菜单，支持加载不同地图

### 中期（2-4 周）
4. **更多关卡数据**：创建 3-5 个不同地形/敌人配置的关卡
5. **关卡胜利条件多样性**：支持 kill_all / survive / escort 等多种胜利条件
6. **简单的装备系统**：如果需要，可以开始数据驱动的装备定义

### 长期（暂缓）
- 剧情系统、任务系统、存档系统 — 等核心玩法完全打磨后再考虑
- 联机、编辑器 — 等单人模式完全可玩后再考虑

---

## 代码质量评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 架构规范 | ⭐⭐⭐⭐⭐ | 严格的 ECS 分离，Effect Pipeline 设计精良 |
| 数据驱动 | ⭐⭐⭐⭐⭐ | 全面 RON 外部化，RegistryLoader 统一加载 |
| 测试覆盖 | ⭐⭐⭐⭐⭐ | 467 个测试，覆盖规则/特性/场景/金标准 |
| 代码风格 | ⭐⭐⭐⭐ | 模块头部 `///` 注释规范，命名一致 |
| 可维护性 | ⭐⭐⭐⭐⭐ | 模块化清晰，新增内容 = 新增 RON 文件 |

---

## 总结

**项目状态：** 已完成 23.md 描述的 7 个优先级中的 6 个（第 7 个基础完成）。

**核心战斗循环：** 已完整打通。Character → Skill → Buff → Turn → Victory 链路已连接。

**数据驱动：** 已全面实现。技能、Buff、角色、地图、AI 行为均通过 RON 文件配置。

**最大亮点：**
1. Effect Pipeline（生成→修饰→执行）设计精良，保证了战斗计算的规范性
2. AI 策略模式实现适度，既有策略性又不过度复杂
3. BattleRecord 结构化记录为未来回放/分析奠定基础

**主要缺口：**
1. 关卡配置缺少胜利条件等元数据
2. 没有关卡选择/进度系统
3. 没有 BattleLog 的 UI 可视化（数据已有，缺展示）

**结论：** 项目已远超"框架基本齐全但还没形成完整可玩战斗循环"的阶段。建议将精力集中在关卡内容丰富和 UI 打磨上，而非继续扩展新系统。
