# 第二阶段评审报告（基于 24第二阶段.md）

> 评审日期：2026-06-13
> 评审依据：`docs/其他/24第二阶段.md`

---

## 总体结论

**项目已跨越 24.md 描述的「战斗沙盒→内容驱动游戏」门槛，且大部分第二阶段要求已经完成。**

24.md 的核心论点：

> 你已经完成 Combat Sandbox（战斗沙盒），但还没进入 Content-driven Game（内容驱动游戏）阶段。

**实际状态：项目已经进入内容驱动游戏阶段，大部分"第二阶段"基础设施已落地。**

---

## 逐项评审

### 一、Skill → 数据化 ✅ **已完成（核心标准超额完成）**

24.md 推测代码风格：

```rust
match skill_id { FireBall => ... }
// 或
fn fireball_system() { ... }
```

**实际状态：项目已全面实现 `Skill = Data + Generic Executor` 模式。**

| 标准 | 实际 | 差距 |
|------|------|------|
| `Skill = Data` | ✅ `assets/skills/*.ron`（6个技能），`SkillData` 纯数据 | 无 |
| `Effect = Data` | ✅ `EffectDef` 枚举 + RON 序列化，4种效果类型 | 无 |
| `System = Generic Executor` | ✅ `generate_combat_effects` 通过 `EffectHandlerRegistry` trait 分发 | 无 |
| `match skill_id` | ✅ **不存在**。全链路通过 SkillRegistry 和 EffectRegistry 驱动 | 无 |
| 条件检查 | ✅ `SkillData::can_use()` 纯函数，支持 MP/标签/HP阈值/冷却 | 无 |
| 技能预览 | ✅ `EffectHandler::preview()` trait 方法，支持伤害/治疗/Buff预览 | 无 |

**关键代码（`src/battle/pipeline/generate.rs`）：**

```rust
// 核心分发逻辑 — 无需 match skill_id
for effect_def in &skill_data.effects {
    if let Some(handler) = handler_registry.find(effect_def.type_name()) {
        let ctx = GenerateContext { ... };
        if let Some(data) = handler.generate(effect_def, &ctx) {
            queue.push(PendingEffect { ... });
        }
    }
}
```

**示例：新增一个技能只需写 RON 文件，无需任何 Rust 代码。**

---

### 二、Map + Battle = 关卡数据化 ⚠️ **基础完成，缺少元数据**

24.md 要求的标准关卡结构：

```yaml
battle_001:
  map: forest_01
  players: [knight, mage]
  enemies: [goblin, goblin, boss]
  win_condition: { type: kill_all }
  lose_condition: { type: all_dead }
```

**实际状态（`assets/maps/tutorial.ron`）：**

```ron
(
    id: "tutorial",
    name: "教学关",
    width: 10, height: 8,
    terrain_grid: [...],
    player_units: [(template: "player_warrior", coord: ...), ...],
    enemy_units: [(template: "enemy_goblin", coord: ...), ...],
)
```

| 标准 | 实际 | 差距 |
|------|------|------|
| 地图尺寸/地形网格 | ✅ | 无 |
| 玩家单位配置 | ✅ | 模板+坐标 |
| 敌方单位配置 | ✅ | 模板+坐标 |
| 胜利条件 | ❌ | 硬编码在 `update_game_over_state()` |
| 失败条件 | ❌ | 硬编码在 `update_game_over_state()` |
| 关卡元数据（ID/名称） | ✅ | 已有 |
| 多关卡支持 | ❌ | 仅 tutorial 一个关卡 |

**结论：** 地图数据驱动的"骨架"已建立，但缺少胜利条件、失败条件等关卡元数据。要支持多关卡还需要关卡选择系统。

---

### 三、Buff 系统"事件化" ✅ **基本完成，但方向不同**

24.md 建议的模型：

```
Event → Buff → Effect → State Change
OnTurnStart → BurnBuff → DamageEffect → HP减少
```

**实际状态：**

```
TurnPhase::SelectUnit
    ↓
resolve_status_effects (每回合开始结算)
    ├─ Stun: 标记 acted=true，移除 Stun
    ├─ DoT: 扣血，发送 DotApplied 消息
    ├─ HoT: 回血，发送 HotApplied 消息
    ├─ tick: 递减持续时间，过期清理修饰符
    └─ tick: 技能冷却递减
```

| 标准 | 实际 | 差距 |
|------|------|------|
| Event → Buff | ✅ `TurnPhase::SelectUnit` 触发结算 | 无 |
| Buff → DoT/HoT/Stun | ✅ `dot_damage`, `hot_heal`, `is_stun` 字段驱动 | 无 |
| Effect → State Change | ✅ 通过 Message 通知（`DotApplied`/`HotApplied`/`StunApplied`） | 无 |
| 统一的 Event Listener 模式 | ❌ | 当前是轮询模式而非监听模式 |

**评价：** 当前 Buff 系统是基于回合阶段驱动的轮询模式，而非事件监听模式。对于回合制 SRPG 来说，这种模式足够且简洁。如果需要更复杂的事件链（如"被攻击时触发""死亡时触发"），可以后续扩展 `TraitTrigger` 系统来实现。

**注意：** 项目已经有一个类似事件监听机制的替代方案——**Trait 系统**（`trigger_on_attack_traits` / `trigger_on_hit_traits` / `trigger_on_kill_traits`），用于响应战斗事件。

---

### 四、基础设施评审

#### 1. Effect System（核心中的核心）✅ **已实现，且符合标准**

24.md 描述：

```
Skill / Buff / Item / Environment → Effect → State Change
Damage / Heal / Move / Teleport / AddBuff / RemoveBuff / Death
```

**实际 Effect Pipeline（`src/battle/pipeline/`）：**

```
CombatIntent
    ↓
Generate (EffectHandlerRegistry trait 分发)
    ↓
Modify (修饰符管线)
    ↓
Execute (应用伤害/Buff/治疗/净化)
    ↓
Message 通知 (DamageApplied/HealApplied/DotApplied 等)
    ↓
BattleRecord 记录
```

| 效果类型 | 状态 |
|----------|------|
| Damage | ✅ EffectHandler trait |
| Heal | ✅ EffectHandler trait |
| AddBuff | ✅ EffectHandler trait |
| RemoveBuff | ✅ 通过 `RemoveBuff` |
| Cleanse | ✅ EffectHandler trait |
| DoT | ✅ 通过 `resolve_status_effects` |
| HoT | ✅ 通过 `resolve_status_effects` |
| Stun | ✅ 通过 `resolve_status_effects` |
| Death | ✅ `Dead` Tag → Observer → `CharacterDied` Message |
| Move | ✅ `MovementIntent` Message → `movement_execution_system` |
| Teleport | ❌ 暂不需要 |
| 修饰符管线 | ✅ `generate → modify → execute` 三步分离 |

**评价：** Effect System 是项目架构的最大亮点。Trait 分发机制（`EffectHandlerRegistry`）使得新增效果类型只需：
1. 实现 `EffectHandler` trait
2. 注册到 `EffectHandlerRegistry`
3. 无需修改核心管线代码

---

#### 2. Data Pipeline（内容化核心）✅ **已实现**

24.md 要求：

```
content/
  ├── skills/
  ├── buffs/
  ├── characters/
  ├── maps/
  ├── battles/
```

**实际 `assets/` 目录：**

| 目录 | 文件数 | 格式 |
|------|--------|------|
| `assets/skills/` | 6 | .ron |
| `assets/buffs/` | 8 | .ron |
| `assets/units/` | 5 | .ron |
| `assets/maps/` | 1 | .ron |
| `assets/terrains/` | 4 | .ron |
| `assets/traits/` | 5 | .ron |
| `assets/ai/` | 3 | .ron |
| `assets/rules/` | 1 | .ron |
| `assets/definitions/` | 2 | .ron |

**评价：** 项目已有完整的数据管线。"不改代码 = 能新增内容"——新增技能/Buff/角色只需添加或修改 RON 文件。

---

#### 3. Battle Replay / Log ✅ **已实现，且超出预期**

24.md 要求的结构化日志：

```
BattleLog
Frame 1: move
Frame 2: skill_cast
Frame 3: damage
Frame 4: buff_apply
```

**实际日志系统有两个层次：**

| 层次 | 名称 | 用途 |
|------|------|------|
| 结构化 | `BattleRecord`（`src/battle/record.rs`） | 战斗统计、回放、调试、查询 |
| 展示 | `CombatLog`（`src/battle/log.rs`） | UI 日志面板，多色文本 |

**`BattleRecord` 覆盖的事件：**
- `TurnStarted` / `TurnEnded`
- `DamageApplied`（含 `DamageBreakdown`）
- `HealApplied`
- `DotApplied` / `HotApplied`
- `StunApplied`
- `CharacterDied`

**用途覆盖：**
| 用途 | 实际 | 说明 |
|------|------|------|
| ✅ 回放 | ✅ | `BattleRecord` 完整序列化，可重建战斗过程 |
| ✅ debug 战斗问题 | ✅ | `DamageBreakdown` 含 base → modified → actual 链路 |
| ✅ 调 AI | ✅ | AI 可通过 logs 分析决策效果 |
| ✅ 做自动测试 | ✅ | 467 个测试使用标准数据场景 |

---

### 五、"如果下一步走错" 警示评估

24.md 警告不要继续做：

| 功能 | 状态 | 评估 |
|------|------|------|
| 装备系统 | ⚠️ 已存在 | `src/equipment/` — 存在但简单 |
| 任务系统 | ❌ 未做 | 正确 |
| AI 复杂化 | ✅ 已做但合理 | 策略模式，适度复杂 |
| UI 系统 | ✅ 已存在 | Egui + Bevy UI |

**评估：** 项目没有陷入"系统越来越多，但游戏不会变大"的陷阱。尽管装备系统和 AI 已实现，但它们的实现是数据驱动的、克制的。

---

### 六、整体差距分析

**24.md 建议路线 vs 实际进度：**

| 优先级 | 建议 | 实际状态 | 进度 |
|--------|------|----------|------|
| 🥇 Skill+Buff→Effect统一化 | 架构升级 | ✅ 已实现 tait 分发 | **100%** |
| 🥈 Battle Data化（关卡配置） | 数据驱动 | ⚠️ 基础已建，缺元数据 | **60%** |
| 🥉 Effect System正式落地 | 统一中枢 | ✅ 完整 pipeline | **100%** |
| ④ BattleLog | 结构化日志 | ✅ 超出预期 | **100%** |

**第一阶段（Combat Sandbox）进度：**
| 阶段 | 进度 |
|------|------|
| ✔ Turn循环 | 100% |
| ✔ Character | 100% |
| ✔ Skill | 100%（数据驱动） |
| ✔ Buff | 100%（数据驱动 + DoT/HoT/Stun） |
| ✔ Victory | 100%（结构数据记录） |
| ✔ 战斗地图 | 80%（数据驱动，缺胜利条件配置） |

---

### 七、下一步建议

项目已完成 24.md 中描述的"第二阶段"大部分内容。基于实际状态：

#### 立即可以做（1-2 周）

1. **关卡配置完善**：在 `assets/maps/*.ron` 增加 `win_condition`/`lose_condition` 字段
2. **多关卡支持**：关卡选择菜单 + 切换逻辑（当前只有一个 tutorial）
3. **关卡配置加载器**：像 `SkillRegistry`/`BuffRegistry` 一样用 `RegistryLoader` trait 统一加载

#### 短期（2-4 周）

4. **BattleLog UI**：`CombatLog` 已有 UI 面板，可以继续完善——特别是 `BattleRecord` 的数据还没在 UI 中展示（目前仅记录，代码中 trace）
5. **技能效果预览 UI**：`EffectHandler::preview()` trait 已实现，但 UI 层未调用
6. **关卡配置文件分离**：将 `map` 和 `battle` 分离为 `assets/maps/`（纯地形）和 `assets/battles/`（含单位配置+胜利条件）

#### 中期（1-2 月）

7. **装备系统数据化**：当前 `src/equipment/` 存在，检查是否已数据驱动
8. **Buff 事件化扩展**：如果需要 React 式 Buff 触发（"被攻击时触发荆棘"），可扩展 `TraitTrigger` 系统

---

### 八、最终评估

| 维度 | 评分 | 说明 |
|------|------|------|
| 从"能跑"→"可扩展" | ⭐⭐⭐⭐⭐ | 数据驱动架构完善，新增内容=新增RON文件 |
| 从"能跑"→"可内容化" | ⭐⭐⭐⭐ | 关卡配置缺元数据，但整体管线已就位 |
| 从"能跑"→"可持续做3年" | ⭐⭐⭐⭐⭐ | 架构设计规范，Effect Pipeline 可扩展性强 |

**结论：项目远超 24.md 的"战斗沙盒"判断，已实质性进入"内容驱动游戏"阶段。** 剩余工作不是架构升级，而是内容填充和元数据完善。
