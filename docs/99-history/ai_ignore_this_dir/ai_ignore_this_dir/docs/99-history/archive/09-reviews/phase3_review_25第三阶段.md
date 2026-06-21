# 第三阶段评审报告（基于 25.第三阶段.md）

> 评审日期：2026-06-13
> 评审依据：`docs/其他/25.第三阶段.md` — Vertical Slice Phase 路线图

---

## 总体结论

**项目已实质性完成 Framework Phase，正在进入 Vertical Slice Phase 的起点。** 25.md 描述的转型判断完全正确，但项目实际状态比文档假设的更成熟：技术框架已就位，真正的瓶颈不是系统缺失，而是内容生产管线尚未建立。

---

## 逐项评审

### 一、框架完整性确认 ✅ 完全符合

25.md 的前提条件：

```
✔ Character
✔ Skill
✔ Buff
✔ Effect
✔ Turn
✔ Victory
✔ Data Driven Skill
✔ Data Driven Buff
✔ Data Driven Character
✔ Data Driven Battle
✔ Data Driven Map
✔ Battle Log
✔ Effect Pipeline
```

**以上 13 项全部验证通过。** 此前两次评审（第一阶段、第二阶段）已逐项核实。项目确实脱离"框架阶段"。

---

### 二、Battle_001 可行性评估 ⚠️ **技术上可行，内容上需要打磨**

25.md 建议的第一张可玩关卡：

```
森林边境
玩家：骑士 × 1, 弓箭手 × 1, 法师 × 1
敌人：哥布林 × 4, 哥布林队长 × 1
```

#### 现有资产（assets/）能否搭出 Battle_001？

| 需求 | 现有资产 | 差距 |
|------|----------|------|
| 骑士（战士） | ✅ `player_warrior.ron` | 有 charge 技能，无 |
| 弓箭手 | ✅ `player_archer.ron` | **缺少 pierce 技能引用**（pierce.ron 文件存在但 archer 未引用） |
| 法师 | ✅ `player_mage.ron` | 有 fireball 技能，无 |
| 哥布林 | ✅ `enemy_goblin.ron` | 只有 basic_attack，无 |
| 哥布林队长 | ⚠️ 无 | 需要新建 `enemy_goblin_leader.ron` |
| 地图 20×20 | ⚠️ 当前 10×8 | 需要新建地图配置 |
| 胜利条件 | ✅ KillAll / AllDead | 已数据驱动 |

#### 关键缺口

1. **`player_archer.ron` 缺失技能**：`skill_ids: ["basic_attack"]`，pierce.ron（穿透箭）文件存在但未引用
2. **无哥布林队长模板**：需要新增 `enemy_goblin_leader.ron`
3. **地图只有 tutorial（10×8）**：需新建 20×20 的森林地图
4. **无关卡选择入口**：游戏直接启动 InGame，无法选择不同关卡

---

### 三、缺失的游戏层模块 🔴 **最大缺口**

25.md 指出的关键问题：

```
你没有看到：campaign, level, content, scenario, enemy, ai, progression
```

**完全正确。** 当前 `src/` 目录：

```
ai/     battle/   buff/     character/   core/
debug/  equipment/  input.rs  inventory/
lib.rs  main.rs   map/      skill/    turn/    ui/
```

**没有以下任一模块：**

| 缺失模块 | 影响 |
|----------|------|
| `content/` | 没有内容资产管线，关卡配置与地图配置混合 |
| `campaign/` | 没有战役/关卡序列概念 |
| `scenario/` | 没有剧本/事件系统 |
| `progression/` | 没有成长/解锁系统 |

**当前关卡配置的设计（`assets/maps/tutorial.ron`）：**
```
assets/maps/
  tutorial.ron     ← 地图+单位+胜利条件 杂糅在一个文件
```

**建议的未来结构：**
```
content/
  battles/
    battle_001.ron   ← 只包含：map_ref + units + victory_condition
  maps/
    forest_01.ron    ← 只包含：尺寸 + 地形网格
  encounters/
    goblin_patrol.ron ← 敌人编队模板
```

这种分离的好处：
- 地图可复用（同一张地图可被多个关卡引用）
- 敌人编队可复用（同一编队出现在不同关卡）
- 关卡是"地图 + 编队 + 条件"的组合，而非一个巨大的配置文件

---

### 四、技能系统：表面够用，内容验证未通过 ⚠️

25.md 预测的反问：

> 你以为 Skill 足够通用？推人、召唤、范围伤害 — 立刻出问题。

#### 现有技能类型（`SkillTargeting`）：
```
SingleEnemy  ✅  basic_attack, charge, fireball, pierce
SingleAlly   ✅  heal, cleanse_skill
SelfOnly     ✅  已定义，未使用
AoeEnemies   ✅  已定义，未使用
AoeAllies    ✅  已定义，未使用
NoTarget     ✅  已定义，未使用
```

#### 实际使用的技能（6个）：
| 技能 | 类型 | 效果 | 冷却 | 使用单位 |
|------|------|------|------|----------|
| basic_attack | SingleEnemy | Damage×1.0 | 0 | 所有单位 |
| charge | SingleEnemy | Damage×1.5 | 0 | warrior, dark_knight |
| pierce | SingleEnemy | Damage×1.3, 无视50%防 | 2 | **无人使用（未引用）** |
| fireball | SingleEnemy | Damage×1.8, 附加灼烧2回合 | 3 | mage |
| heal | SingleAlly | Heal 8 HP | 2 | **无人使用（未引用）** |
| cleanse_skill | SingleAlly | Cleanse | 3 | **无人使用（未引用）** |

#### 发现的问题

1. **3个技能（pierce, heal, cleanse_skill）定义了但无人使用** — archer 没引用 pierce，无人引用 heal/cleanse
2. **AOE 类型定义了但无内容使用** — `AoeEnemies`/`AoeAllies` 是空架子
3. **无推拉/击退效果** — EffectDef 中没有 `Knockback`/`Pull` 变体
4. **无召唤效果** — EffectDef 中没有 `Summon` 变体
5. **无范围伤害** — 所有技能都是单体

#### 预测验证
- 推人：❌ 不支持，需要新增 `KnockbackEffect` + 推人物理系统
- 召唤：❌ 不支持，需要新增 `SummonEffect` + 单位生成 + 回合插入
- 范围伤害：⚠️ `AoeEnemies` 已定义但 Effect 管线未验证

**结论：** Skill 数据化框架是好的，但**内容覆盖面太窄**，3 个关键扩展方向（推人/召唤/范围伤害）都没被实际内容验证过。

---

### 五、Buff 系统：够用但缺复杂模式 ⚠️

25.md 预测的反问：

> 你以为 Buff 架构很好？反击、吸血、光环 — 立刻出问题。

#### 现有 Buff（8个）：
| Buff | 效果 | 类型 |
|------|------|------|
| burn | DoT 2, -2 Defense | DoT + Debuff |
| poison | DoT 3 | DoT |
| regen | HoT 4 | HoT |
| stun | 眩晕 | 控场 |
| attack_up | +3 Attack | 增益 |
| attack_down | -2 Attack | 减益 |
| defense_up | +3 Defense | 增益 |
| defense_down | -2 Defense | 减益 |

#### 缺失模式

| 模式 | 需要 | 现有基础 |
|------|------|----------|
| **反击** | 受伤时触发攻击 | `TraitTrigger` 有 `on_hit` 触发，但缺反击效果类型 |
| **吸血** | 造成伤害时治疗自己 | 需要在 Damage 效果后附加 Heal |
| **光环** | 每回合影响周围单位 | 需要范围检测 + 状态同步 |
| **层数系统** | 多 Buff 叠加 | 当前是同源刷新，不支持叠层 |
| **驱散免疫** | 特定标签不能被驱散 | 当前 Cleanse 驱散所有 Debuff |

#### 预测验证
- 反击：⚠️ 有 `TraitTrigger::OnHit` 基础，但没实际效果实现
- 吸血：⚠️ 通过 Effect Pipeline 可以组合（Damage + Heal），但无实际内容测试
- 光环：❌ 目前只支持"被动特质"（Passive Trait），不支持区域光环

---

### 六、回合系统：基础稳固，极限情况未验证 ⚠️

25.md 预测的反问：

> 你以为 Turn 完整？死亡插队、复活、召唤物 — 立刻出问题。

#### 当前状态
| 场景 | 状态 | 说明 |
|------|------|------|
| 正常回合流转 | ✅ | TurnOrder + TurnPhase 完整 |
| 死亡清除 | ✅ | Dead 标记 → 从队列移除 |
| 死亡插队 | ⚠️ | 死亡不影响后续行动者顺序 |
| 复活 | ❌ | 无 Dead→Alive 逆转机制 |
| 召唤物 | ❌ | 无战斗中生成新单位并插入队列 |
| 多阶段Boss | ❌ | 无阶段转换触发 |

#### 分析
当前 TurnOrder 使用 `Vec<(Entity, u32)>`（entity → initiative），是一个静态的快照。如果战斗中加入新单位（召唤），需要重新插入队列。如果单位复活，需要从 Dead 移除并重新加入。这些都需要对 TurnOrder 进行动态修改。

---

### 七、AI 系统：已完成回合制基础，但战术深度不足 ⚠️

#### 现有能力
| 能力 | 状态 | 说明 |
|------|------|------|
| 行为配置数据驱动 | ✅ | RON 文件配置 |
| 目标选择策略 | ✅ | Nearest / Weakest / MostDangerous |
| 移动策略 | ✅ | Aggressive / Cautious |
| 技能选择 | ✅ | PreferSpecial / PreferBasic / ByPriority |
| 计时器延迟 | ✅ | AI 决策有延迟，能观察到"思考" |
| 不直接操作 ECS | ✅ | 通过 MovementIntent + CombatIntent 合规 |

#### 缺失能力
| 能力 | 重要性 | 说明 |
|------|--------|------|
| 合作行为 | 高 | 多个敌人各自决策，不协作 |
| 战术站位 | 中 | 不会考虑高地/地形优势 |
| 治疗AI | 中 | 不支持友方治疗行为 |
| 撤退AI | 低 | 不会因为劣势撤退 |
| AOE 使用 | 中 | 不支持范围技能目标选择 |

---

### 八、UI：基础完整，但缺少游戏框架层 UI 🔴

#### 已有 UI
| 组件 | 状态 |
|------|------|
| 行动菜单（攻击/技能/待机） | ✅ |
| 移动范围高亮 | ✅ |
| 攻击范围高亮 | ✅ |
| 单位信息面板 | ✅ |
| 回合指示器 | ✅ |
| 战斗日志面板 | ✅ |
| 摄像机控制 | ✅ |
| 格子信息浮窗 | ✅ |
| 伤害飘字 | ✅ |
| 已行动单位颜色变灰 | ✅ |

#### 缺失 UI
| 组件 | 状态 | 影响 |
|------|------|------|
| **主菜单** | ❌ | 进入游戏即开始战斗 |
| **关卡选择** | ❌ | 不能选关 |
| **胜利/失败结果画面** | ❌ | GameOverState 变了但无 UI 展示 |
| **战斗重开** | ❌ | 结束后只能强退 |
| **伤害预览数字** | ⚠️ | `CombatPreviewView` 存在但 UI 未渲染 |

---

### 九、25.md 四周期计划对齐评估

25.md 建议的 4 周路线：

```
第1周：Battle_001
第2周：Battle_002
第3周：Battle_003
第4周：整理暴露的问题（Effect/Buff/AI/Map 重构）
```

#### 准备度评估

**第1周 Battle_001 的准备度：70%**

可以直接做的：
- ✅ 3个职业模板已有（战士/弓箭手/法师）
- ✅ 基础攻击和技能数据模型已就位
- ✅ 胜利条件已数据驱动
- ✅ AI 能完成基础战斗
- ✅ 地图格式支持

需要补充的：
- ❌ `player_archer.ron` 缺 `pierce` 技能引用（pierce.ron 文件存在，但模板没引用）
- ❌ 无哥布林队长模板
- ❌ 地图配置混合（未分离地图/关卡/编队）
- ❌ 战场大小 10×8，不是 20×20
- ❌ 无关卡选择入口

**第4周重构预测：**

基于当前架构分析，Battle_001 → Battle_003 过程中最可能暴露的问题：

1. **Effect 缺乏范围/区域效果**：当前 `AoeEnemies` 已枚举但 Effect 管线没处理范围目标选择
2. **Buff 缺乏触发式效果**：`TraitTrigger` 可以作基础，但缺少"受伤触发""死亡触发"的事件化 Buff
3. **AI 缺乏战术深度**：当前 AI 不关心地形、不合作、不会用 AOE
4. **模块边界模糊**：map/level/battle 边界不清晰，MapPlugin 负责太多

---

### 十、内容生产管线评估 🔴 **最大建议**

25.md 的核心建议：

```
技术资产 ≈ 20%
内容资产 ≈ 80%
```

#### 当前内容资产统计

| 类型 | 数量 | 评估 |
|------|------|------|
| 单位模板 | 5（3玩家+2敌人） | 太少，至少需要 8-10 个 |
| 技能 | 6（3个无人使用） | 6 个够，但覆盖不全 |
| Buff | 8 | 够，但缺光环/反击/吸血 |
| 地图 | 1（10×8） | 严重不足 |
| AI 行为 | 3 | 够 |
| 关卡 | 1 | 太少 |

#### 建议的内容管线结构

```
content/
  battles/        ← 关卡配置（组合 maps + units + conditions）
    battle_001.ron
    battle_002.ron
  maps/           ← 纯地图数据（仅尺寸+地形）
    forest_01.ron
    plains_01.ron
  encounters/     ← 敌人编队模板（可复用）
    goblin_patrol.ron
    boss_guard.ron
  waves/          ← 增援波次（可选）
```

**目标：** 新增一个关卡 ≈ 写一个 RON 文件，组合已有地图 + 敌人编队 + 条件。

---

### 十一、第一阶段 vs 第三阶段评审对比

| 维度 | 第一阶段评审 | 本次评审 | 变化 |
|------|------------|----------|------|
| 核心战斗循环 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 无变化 |
| 数据驱动 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 无变化 |
| 测试覆盖 | ⭐⭐⭐⭐⭐ (467) | ⭐⭐⭐⭐⭐ (498) | +31 个测试 |
| 内容管线 | ❌ 未评估 | ⭐⭐ | 基础文件有，但无分离抽象 |
| UI 完整性 | ❌ 未评估 | ⭐⭐⭐ | 战斗 UI 完整，框架 UI 缺失 |
| 战役/关卡系统 | ❌ 不存在 | ❌ 不存在 | 无变化 |
| 胜利条件配置 | ❌ 硬编码 | ✅ 数据驱动 | **已修复** |

---

### 十二、结论与建议

#### 项目当前定位

```
Framework Phase ──── 100% ────→ ✓
       ↓
Vertical Slice ─────── 起步 ──→ 当前在这里
       ↓
Content Production ─── 0% ────→ 下个目标
```

#### 最紧迫的 3 件事

**P0 — 让框架 UI 完整，让游戏可玩可结束**
1. 实现 Victory/Defeat 结果画面（`AppState::GameOver` 已有，无 UI）
2. 实现主菜单（至少"开始游戏" + "退出"）
3. 实现关卡选择（可以选择不同地图）

**P1 — 完善第一张关卡的内容质量**
4. 修复 `player_archer.ron` 缺失 pierce 技能引用
5. 新增 `enemy_goblin_leader.ron` 模板
6. 新增 20×20 森林地图
7. 分离关卡配置：地图只存地形，关卡只存编队+条件

**P2 — 为内容管线搭建基础设施**
8. 新增 `content/` 模块（或 `scenario/`），提供关卡→地图→编队的组合抽象
9. 实现 `CampaignRegistry`，类似已有的 `SkillRegistry`/`BuffRegistry`
10. 允许多关卡连续加载（当前只加载第一个关卡）

#### 不做的事（与 25.md 一致）

- ❌ Quest 系统 — 等有 5 个以上关卡再考虑
- ❌ Save/Load — 等关卡能往返再考虑
- ❌ Dialogue — 等有剧情需求再考虑
- ❌ Crafting — 等核心 SRPG 体验验证再考虑
- ❌ 编辑器 — 手动 RON 足够支撑 10-20 个关卡

#### 风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 内容生产缓慢（写 RON 烦） | 中 | 高 | 考虑简单的关卡可视化工具 |
| 新效果类型（推人/召唤）需加 EffectHandler | 中 | 中 | EffectHandler trait 支持扩展 |
| AOE 管线未验证 | 中 | 高 | Battle_002 立即测试 AOE |
| AI 不合作导致关卡无聊 | 高 | 高 | Battle_001 手动验证后迭代 AI |
| Effect/Buff/AI/Map 重构积压 | 中 | 高 | 每做一个新关卡留 1 天整理日 |

---

## 总结

**项目状态合格，方向正确。** 25.md 的"做一关 → 发现问题 → 修架构 → 做下一关"循环是正确的下一步。

当前最大的价值杠杆不是加新系统，而是：
1. **让第一个关卡变成真正可玩的体验**（完整的 UI 流程：开始→选关→战斗→结果→重玩）
2. **建立内容管线**（地图/关卡/编队分离，复用）
3. **用真实内容暴露架构缺口**（AOE、召唤、推拉等效果验证）

**最关键的一步：在写第二关之前，先把第一关从"技术验证"变成"可玩关卡"。**
