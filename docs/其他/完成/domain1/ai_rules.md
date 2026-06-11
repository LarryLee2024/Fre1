# AI 领域规则 (AI Rules)

## 1. 领域概述

AI 系统管理敌方单位的自动决策。采用数据驱动的行为配置 + 策略 trait 扩展点，替代硬编码的 AI 逻辑。新增策略只需实现对应 trait 并注册，无需修改已有代码。

### 核心原则

- **Trait 描述规则，不描述内容**：策略 trait 定义接口，具体行为由实现决定
- **数据驱动**：AI 行为从 RON 配置加载，不同单位使用不同行为模式
- **注册表分发**：通过 strategy_name 查找 trait 对象，替代 enum+match
- **Rule / Content 分离**：代码负责策略规则，配置负责行为内容

---

## 2. AI 决策流程

```
1. TurnOrder 当前单位是敌方？
2. AI 计时器到期（0.8秒）
3. 收集所有单位快照（UnitSnapshot）
4. 获取 AI 行为配置（AiBehaviorRegistry）
5. 选择目标（TargetSelector）
6. 计算可达范围（find_reachable_tiles）
7. 选择移动位置（MoveSelector）
8. 选择技能（SkillSelector）
9. 设置 CombatIntent → Effect Pipeline 处理
10. 标记已行动
```

---

## 3. AiBehavior — AI 行为配置

### 3.1 AiBehaviorDef（RON 反序列化）

```rust
pub struct AiBehaviorDef {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub target_strategy: TargetStrategy,
    pub move_strategy: MoveStrategy,
    pub skill_strategy: SkillStrategy,
    pub skill_priority: Vec<String>,
}
```

### 3.2 AiBehavior（运行时）

```rust
pub struct AiBehavior {
    pub id: String,
    pub name: String,
    pub target_strategy: String,  // 策略名称字符串
    pub move_strategy: String,
    pub skill_strategy: String,
    pub skill_priority: Vec<String>,
}
```

**规则**：enum variant 名转为字符串，与 trait 的 `strategy_name()` 对应。

### 3.3 内置默认行为

| ID | 名称 | 目标策略 | 移动策略 | 技能策略 | 技能优先级 |
|----|------|----------|----------|----------|-----------|
| `default` | 默认 | Nearest | Aggressive | PreferSpecial | — |
| `aggressive` | 激进 | Weakest | Aggressive | PreferSpecial | — |
| `cautious` | 谨慎 | Nearest | Cautious | PreferSpecial | — |
| `support` | 辅助 | Nearest | Support | ByPriority | heal, cleanse_skill |

---

## 4. 策略 Trait 扩展点

### 4.1 TargetSelector — 目标选择

```rust
pub trait TargetSelector: Send + Sync + 'static {
    fn strategy_name(&self) -> &'static str;
    fn select(&self, candidates: &[UnitSnapshot], my_coord: IVec2) -> Option<IVec2>;
}
```

| 实现 | strategy_name | 选择规则 |
|------|---------------|----------|
| `NearestTarget` | "Nearest" | 曼哈顿距离最近的敌人 |
| `WeakestTarget` | "Weakest" | HP 最低的敌人 |
| `MostDangerousTarget` | "MostDangerous" | 攻击力最高的敌人 |
| `LowestHpPercentTarget` | "LowestHpPercent" | HP 百分比最低的敌人 |

### 4.2 MoveSelector — 移动选择

```rust
pub trait MoveSelector: Send + Sync + 'static {
    fn strategy_name(&self) -> &'static str;
    fn select(&self, reachable: &HashMap<IVec2, u32>, my_coord: IVec2, target_coord: IVec2, attack_range: u32) -> IVec2;
}
```

| 实现 | strategy_name | 选择规则 |
|------|---------------|----------|
| `AggressiveMove` | "Aggressive" | 贪心靠近目标 |
| `CautiousMove` | "Cautious" | 保持攻击距离（范围内最远位置） |
| `SupportMove` | "Support" | 优先靠近友军 |

**CautiousMove 规则**：
1. 筛选在攻击范围内的可达位置
2. 有范围内位置 → 选择距目标最远的（保持距离）
3. 无范围内位置 → 贪心靠近目标

### 4.3 SkillSelector — 技能选择

```rust
pub trait SkillSelector: Send + Sync + 'static {
    fn strategy_name(&self) -> &'static str;
    fn select<'a>(&self, skill_ids: &'a [String], cooldowns: &SkillCooldowns, priority: &'a [String]) -> &'a str;
}
```

| 实现 | strategy_name | 选择规则 |
|------|---------------|----------|
| `PreferSpecialSkill` | "PreferSpecial" | 优先特殊技能（跳过冷却），否则基础攻击 |
| `PreferBasicSkill` | "PreferBasic" | 优先基础攻击 |
| `ByPrioritySkill` | "ByPriority" | 按优先级列表选择，空则回退 PreferSpecial |

**ByPrioritySkill 规则**：
1. 遍历 priority 列表
2. 技能在 skill_ids 中且不在冷却 → 选择
3. 全部不可用 → 回退 PreferSpecial 逻辑

---

## 5. AiStrategyRegistry — 策略注册表

```rust
#[derive(Resource)]
pub struct AiStrategyRegistry {
    pub target_selectors: HashMap<String, Box<dyn TargetSelector>>,
    pub move_selectors: HashMap<String, Box<dyn MoveSelector>>,
    pub skill_selectors: HashMap<String, Box<dyn SkillSelector>>,
}
```

### 5.1 查找与回退

| 查找方法 | 回退策略 |
|----------|----------|
| `target_selector(name)` | 未知名称 → "Nearest" |
| `move_selector(name)` | 未知名称 → "Aggressive" |
| `skill_selector(name)` | 未知名称 → "PreferSpecial" |

---

## 6. UnitSnapshot — 单位快照

```rust
pub struct UnitSnapshot {
    pub entity: Entity,
    pub faction: Faction,
    pub coord: IVec2,
    pub atk: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub mov: u32,
    pub attack_range: u32,
    pub acted: bool,
    pub skill_ids: Vec<String>,
    pub cooldowns: SkillCooldowns,
    pub ai_behavior_id: String,
    pub tags: GameplayTags,
}
```

**规则**：纯数据快照，避免 ECS 借用冲突。

---

## 7. enemy_ai_system — AI 主系统

### 7.1 执行条件

1. `TurnPhase == SelectUnit`
2. `TurnOrder.current_unit()` 是敌方
3. AI 计时器到期（0.8秒）

### 7.2 决策流程

```
1. 收集所有单位快照
2. 获取当前敌方单位的 AI 行为配置
3. select_target_coord → 目标坐标
4. resolve_from_tags → 地形成本计算器
5. find_reachable_tiles → 可达范围
6. select_move_coord → 移动位置
7. select_skill → 技能 ID
8. 计算有效攻击范围
9. 检查攻击目标是否在范围内
10. 设置 CombatIntent / MovingUnit
```

### 7.3 行动结果

| 情况 | 处理 |
|------|------|
| 有攻击目标 + 需移动 | MovingUnit → ExecuteAction |
| 有攻击目标 + 不需移动 | ExecuteAction |
| 无攻击目标 + 需移动 | MovingUnit → WaitAction |
| 无攻击目标 + 不需移动 | WaitAction |

---

## 8. AiBehaviorRegistry — 行为注册表

```rust
#[derive(Resource)]
pub struct AiBehaviorRegistry {
    pub behaviors: HashMap<String, AiBehavior>,
}
```

| 方法 | 说明 |
|------|------|
| `get(id)` | 查找行为 |
| `register(behavior)` | 注册行为 |
| `default_behavior()` | 获取默认行为（"default"，否则第一个） |

**数据加载**：`assets/ai/` 目录，通过 `RegistryLoader` 加载 RON 文件。

---

## 9. RON 配置格式

```ron
(
    id: "aggressive",
    name: "激进",
    target_strategy: Weakest,
    move_strategy: Aggressive,
    skill_strategy: PreferSpecial,
    skill_priority: [],
)
```

带技能优先级：

```ron
(
    id: "support",
    name: "辅助",
    target_strategy: Nearest,
    move_strategy: Support,
    skill_strategy: ByPriority,
    skill_priority: ["heal", "cleanse_skill", "basic_attack"],
)
```

---

## 10. 关键约束

1. **策略 trait 替代 enum+match**：新增策略只需实现 trait 并注册
2. **strategy_name 与 RON 对应**：enum variant 名转为字符串
3. **注册表回退机制**：未知策略名称回退到默认策略
4. **UnitSnapshot 避免借用冲突**：纯数据快照，不持有 ECS 引用
5. **AI 计时器 0.8 秒**：Once 模式，防止 AI 瞬间行动
6. **CombatIntent 统一处理**：AI 和玩家共用 Effect Pipeline
7. **行为配置数据驱动**：不同单位使用不同行为模式
8. **default_behavior 兜底**：找不到指定行为时回退
9. **技能冷却检查**：选择技能时跳过冷却中的技能
10. **地形成本由标签解析**：SWIMMING > FLYING > MOUNTED > ground
