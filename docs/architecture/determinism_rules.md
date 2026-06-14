# 确定性执行规范

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第224-230行）

---

## 1. 概述

确定性执行是 SRPG 的**物理定律**：给定相同的输入序列，游戏必须产出完全相同的状态序列。这是回放系统、Bug 复现、自动化测试、未来多人同步的基石（宪法 18.4.1）。

没有确定性保证：
- 回放系统无法验证（重放结果与录制不一致）
- Bug 无法稳定复现（随机因素导致间歇性出现）
- 自动化测试不可靠（同一测试用例不同运行结果不同）
- 多人联机无法实现（客户端状态分歧）

本规范定义项目中所有影响战斗结果的底层执行约束。

---

## 2. 设计

### 2.1 确定性随机数生成器（PRNG） 🟥

🟥 所有战斗随机性必须通过**确定性 PRNG 管线**管理（宪法 11.9.1、11.9.2）。核心设计原则：**预生成随机流 + 派生种子策略**，消除 `ResMut<BattleRng>` 借用冲突，释放 ECS 多线程并行能力。

> **优化来源**：`docs/其他/48.md` — "BattleRng 的 ResMut 借用冲突导致多线程瘫痪"、"随机流/派生种子"方案。

#### 架构方案：预生成随机流（推荐）

```
UnitSpawn 阶段（战斗前）
  → 从 LevelConfig 或 ReplaySeed 读取 master_seed
  → 为每个 Unit 生成预计算随机流 Vec<u32>，存入 RandomStream Component
  → 战斗 System 只需 Res<RandomStream>（只读锁）+ 当前 ActionIndex 获取随机数
  → 完美并行：无 ResMut 借用冲突
```

#### 派生种子策略

```
master_seed（关卡种子，存入回放文件）
  ├── derive("combat", turn_number)     → 回合级种子（每回合重置）
  ├── derive("unit", entity_logical_id) → 单位级种子（Buff 触发、被动判定）
  └── derive("action", action_index)    → 动作级种子（伤害浮动、暴击判定）

NEVER 在回放文件中存储实际随机值，只存储 seed。
```

#### PRNG 选型：ChaCha8Rng 🟥

🟥 **必须使用 `rand_chacha::ChaCha8Rng`**，禁止使用 `xorshift64*` 或其他非标准 PRNG（宪法 11.9.1）。

| 特性 | ChaCha8Rng | xorshift64* |
|------|-----------|-------------|
| `Send + Sync` | ✅ | 需手动保证 |
| 可 clone（并行迭代器） | ✅ | ❌ |
| 跨平台确定性 | ✅ 有标准保证 | ⚠️ 依赖实现 |
| 性能 | 足够（< 1ns/call） | 更快但不安全 |

```rust
// ✅ 正确：ChaCha8Rng 支持 clone，可安全用于并行迭代器
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

let master = ChaCha8Rng::seed_from_u64(master_seed);
let unit_rng = master.clone(); // 每个线程/单位获得独立副本
```

#### 结构定义

```rust
/// 确定性随机数生成器 Resource（master seed 管理）
#[derive(Resource)]
pub struct BattleRng {
    /// ChaCha8Rng 内部状态
    rng: ChaCha8Rng,
    /// 种子值（存储在回放文件中）
    seed: u64,
}

/// 单位级预生成随机流 Component
#[derive(Component)]
pub struct RandomStream {
    /// 预生成的随机数序列
    values: Vec<u32>,
    /// 当前消费位置
    cursor: usize,
}

impl RandomStream {
    /// 消费下一个随机数（只读操作，无借用冲突）
    pub fn pop_random(&mut self, n: u32) -> u32 {
        let val = self.values[self.cursor % self.values.len()];
        self.cursor += 1;
        val % n
    }
}
```

#### 使用规则

| 规则 | 强制等级 | 说明 |
|------|---------|------|
| 所有战斗随机通过 `RandomStream` 或 `BattleRng` | 🟥 | 暴击判定、闪避判定、伤害浮动、Buff 触发等 |
| 种子在战斗开始时确定 | 🟥 | 从回放文件或关卡配置读取 |
| 禁止使用 `rand::thread_rng()` | 🟥 | 系统随机不可复现 |
| 禁止使用 `rand::rngs::SmallRng::from_entropy()` | 🟥 | 同上 |
| 禁止在 System 内创建独立 Rng 实例 | 🟥 | 破坏全局状态一致性 |
| 必须使用 `ChaCha8Rng` | 🟥 | 跨平台确定性 + Send + Sync |
| 种子格式与回放文件兼容 | 🟩 | 支持从回放文件恢复种子 |
| 回放文件只存 seed，不存随机值 | 🟥 | 反序列化兼容 + 安全 |

#### 种子来源

```
关卡开始 → 从 LevelConfig 或 ReplaySeed 读取 master_seed
         → 初始化 BattleRng::from_seed(master_seed)
         → 为每个 Unit 派生 unit_seed = derive(master_seed, unit_logical_id)
         → 预生成 RandomStream Component 插入到 Unit Entity
         → 战斗系统从 RandomStream 获取随机数（只读，完美并行）
```

---

### 2.2 数值精度 🟥

🟥 核心战斗数值**禁止使用浮点数**（宪法 15.0.1 正确性优先）。

#### 整数约束

| 类型 | 要求 | 替代方案 |
|------|------|---------|
| HP、MP、伤害值 | 🟥 必须为 `i32` 或 `u32` | — |
| 攻击力、防御力 | 🟥 必须为整数 | — |
| 百分比修饰（如 +15% 攻击） | 🟥 用整数表示（15 = 15%） | 不使用 `f32` |
| 属性公式中的乘除 | 🟥 先乘后除，避免精度丢失 | `value * multiplier / 100` |
| 地形防御加成 | 🟥 整数 | `defense_bonus: i32` |
| 移动消耗 | 🟥 整数 | `move_cost: u32` |

#### 公式计算规则

```
// ✅ 正确：整数运算，先乘后除
let damage = (attack * 100 / defense).max(1);

// 🟥 错误：浮点运算
let damage = (attack as f32 / defense as f32).max(1.0) as i32;
```

#### 允许使用浮点数的场景

| 场景 | 允许原因 | 约束 |
|------|---------|------|
| 世界坐标计算 | 渲染层需要 | 不影响战斗逻辑 |
| 动画插值 | 视觉平滑 | 不影响战斗状态 |
| UI 布局 | 像素对齐 | 不影响游戏逻辑 |
| 音频音量 | 体验参数 | 不影响战斗结果 |

#### 强制舍入规则

当不可避免地涉及非整数运算时：

```
截断舍入（默认）：let result = (a * b) / c;
四舍五入（仅限 UI 显示）：let display = (a * b * 10 / c + 5) / 10;
禁止向上取整用于战斗数值（会膨胀伤害）
```

#### 整数溢出防护

🟥 **核心公式计算时，中间变量必须提升为 `i64`**，计算完成后再截断回 `i32`。

> **优化来源**：`docs/其他/48.md` — "Rust 整数运算的溢出 Panic"。

```rust
// ❌ 危险：i32 乘法可能溢出（Debug 模式 Panic，Release 模式 Wrap Around）
let damage = (attack * 100 / defense).max(1);

// ✅ 安全：中间变量提升为 i64，防溢出
let damage = ((attack as i64 * 100 * crit_mult as i64) / defense as i64).max(1) as i32;

// ✅ 或使用 checked_mul / saturating_mul
let damage = attack.checked_mul(100)
    .and_then(|v| v.checked_mul(crit_mult as i32))
    .map(|v| (v / defense).max(1))
    .unwrap_or(i32::MAX);  // 溢出时安全降级
```

**适用场景**：所有涉及多步乘法的公式（攻击力 × 暴击倍率 × 元素克制等），中间值极易突破 `i32::MAX`（21亿）。

---

### 2.3.5 浮点污染隔离墙（Float Contamination Firewall）

> **优化来源**：`docs/其他/48.md` — "浮点污染隔离墙"章节建议。

#### 物理隔离

战斗逻辑 System（LogicSet）的代码中，**禁止出现 `f32` / `f64` 类型**。可通过 Clippy 自定义 Lint（`#[deny(float_operations)]`）静态扫描。

#### 表现层降级

当 ViewModelSet 读取逻辑层数据用于 UI 显示（如血条百分比、伤害预估）时，允许将整数转换为浮点数进行渲染，但**严禁将计算结果回写到逻辑层 Component**。

```rust
// ✅ ViewModelSet 中：整数 → 浮点用于渲染
fn update_hp_bar(attrs: &Attributes, mut bar: Mut<HealthBar>) {
    bar.percentage = attrs.current_hp as f32 / attrs.max_hp as f32;  // 仅用于显示
}

// 🟥 禁止：浮点计算结果回写逻辑层
fn calculate_and_write(attrs: &mut Attributes) {
    attrs.current_hp = (attrs.current_hp as f32 * 0.9) as i32;  // 污染！
}
```

---

### 2.3 迭代顺序与确定性逻辑 ID 🟥

🟥 Bevy 的 ECS 查询默认**不保证迭代顺序**，这在战斗逻辑中会导致不确定性（宪法 18.4.1）。

> **优化来源**：`docs/其他/48.md` — "Entity ID 排序的跨平台/跨存档不确定性"、"引入确定性逻辑 ID (LogicalId)"。

#### LogicalId：确定性排序的基石

🟥 **禁止使用 Bevy Entity ID 排序**。Bevy Entity 由内部计数器生成（Index + Generation），不是确定性的！不同加载时序可能导致同一 Unit 获得不同的 Entity ID，引发状态分歧（Desync）。

```rust
/// 确定性逻辑 ID — 随存档/Replay 保存，不依赖 Bevy Entity
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalId(pub u32);  // 战斗开始时按加载顺序从 0 递增分配

/// 所有排序必须使用 LogicalId，而非 Bevy Entity
fn process_units(query: Query<(&LogicalId, &Unit, &mut Attributes)>) {
    let mut units: Vec<_> = query.iter().collect();
    units.sort_by_key(|(lid, _, _)| lid.0);  // ✅ 确定性排序
}
```

**分配时机**：UnitSpawn 阶段，按 Unit 数据库 ID 或加载顺序从 0 递增分配，写入 `LogicalId` Component。

#### 需要排序的场景

| 场景 | 强制等级 | 排序依据 |
|------|---------|---------|
| 行动队列构建 | 🟥 | Initiative 降序 + LogicalId 升序 |
| 同时触发的 Buff 结算 | 🟥 | Buff 注册顺序（InsertionOrder） |
| 同回合多个死亡判定 | 🟥 | LogicalId 升序 |
| 属性 Modifier 栈计算 | 🟥 | ModifierSource 优先级 |
| AOE 伤害目标遍历 | 🟥 | LogicalId 升序 |
| 非战斗逻辑的 ECS 查询 | 🟩 | 无强制要求 |

#### 排序实现规范

```rust
// ✅ 正确：使用 LogicalId 显式排序后处理
fn process_units(query: Query<(&LogicalId, &Unit, &mut Attributes)>) {
    let mut units: Vec<_> = query.iter().collect();
    units.sort_by_key(|(lid, _, _)| lid.0);  // 确定性排序
    
    for (lid, unit, attrs) in units {
        // 处理逻辑
    }
}

// 🟥 错误：依赖默认迭代顺序
fn process_units_bad(query: Query<(&Unit, &mut Attributes)>) {
    for (unit, attrs) in query.iter() {
        // 顺序不确定
    }
}

// 🟥 错误：使用 Bevy Entity 排序（跨存档不确定）
fn process_units_bad2(query: Query<(&Unit, &mut Attributes)>) {
    let mut units: Vec<_> = query.iter().collect();
    units.sort_by_key(|(unit, _)| unit.entity);  // Entity 不确定！
}
```

#### 排序键规范

| 用途 | 排序键 | 方向 |
|------|--------|------|
| 行动顺序 | Initiative | 降序（高先行动） |
| 相同 Initiative | LogicalId | 升序（稳定排序） |
| 伤害结算 | LogicalId | 升序（确定性） |
| Buff 结算 | 插入顺序 | 升序 |
| 目标选择 | LogicalId | 升序 |

#### 寻路确定性（SRPG 专项）

寻路算法（如 A*）在遇到"代价相同的多条路径"时，必须引入确定性的 **Tie-breaker**（平局打破机制）。必须规定"优先 X 轴"或"优先 Y 轴"，否则不同平台可能选出不同路径。

#### 空间查询确定性（SRPG 专项）

范围技能（AOE）的目标获取，禁止依赖物理引擎（如 bevy_xpbd）的 Broad-phase 返回顺序。获取 AOE 范围内的 Entity 后，必须强制按 LogicalId 排序再执行伤害结算。

---

### 2.4 系统执行顺序 🟥

🟥 同一 Schedule 内的系统必须有**明确的执行顺序**（宪法 2.3.8）。

#### SystemSet 排序约束

```
InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
```

#### 关键顺序要求

| 先执行 | 后执行 | 原因 |
|--------|--------|------|
| `damage_calculation` | `buff_application` | 伤害必须在 Buff 应用前完成 |
| `buff_application` | `attribute_recalculate` | Buff 修改属性后需重算 |
| `effect_generate` | `effect_modify` | 管线三步顺序 |
| `effect_modify` | `effect_execute` | 管线三步顺序 |
| `combat_intent_system` | `effect_generate` | 意图必须在效果前 |
| `turn_end_cleanup` | `victory_check` | 清理后才能正确判定 |
| `all logic systems` | `view_model_update` | 逻辑完成后刷新 UI |

#### 禁止事项

- 🟥 同一 Set 内的系统依赖隐式执行顺序
- 🟥 使用 `before()` / `after()` 但不声明在 Set 定义中
- 🟥 系统间存在循环依赖（A after B 且 B after A）

---

### 2.5 状态哈希 🟥

🟥 每个关键状态边界必须计算**确定性哈希**，用于检测执行分歧（宪法 18.4.1、13.10.3）。

> **优化来源**：`docs/其他/48.md` — "状态哈希的遗漏与顺序陷阱"、"排序后哈希 + 宏驱动生成"。

#### 哈希计算时机

| 时机 | 强制等级 | 说明 |
|------|---------|------|
| 每回合结束（TurnEnd） | 🟥 | 回合级一致性校验 |
| 每次伤害结算后 | 🟩 | 战斗级一致性校验（可选） |
| 回放验证时 | 🟥 | 与参考哈希对比 |

#### 哈希内容

```rust
/// 计算战斗状态的确定性哈希
/// 包含所有影响战斗结果的状态
fn compute_state_hash(world: &World) -> u64 {
    let mut hasher = DeterministicHasher::new();
    
    // 1. 所有存活单位的关键属性 — 必须按 LogicalId 排序
    let mut units: Vec<_> = query.iter().collect();
    units.sort_by_key(|(lid, _, _)| lid.0);  // ✅ 确定性顺序
    
    for (lid, unit, attrs) in units {
        hasher.write_u32(lid.0);
        hasher.write_u32(attrs.current_hp);
        hasher.write_u32(attrs.max_hp);
        hasher.write_i32(attrs.attack);
        hasher.write_i32(attrs.defense);
        hasher.write_u32(unit.grid_position.x);
        hasher.write_u32(unit.grid_position.y);
    }
    
    // 2. 活跃 Buff 列表
    for (buff, active) in buff_query.iter() {
        hasher.write_u32(buff.buff_id);
        hasher.write_u32(buff.remaining_turns);
        hasher.write_u32(buff.stack_count);
    }
    
    // 3. 当前回合号
    hasher.write_u32(turn_order.turn_number);
    
    // 4. 回合阶段
    hasher.write_u8(turn_phase as u8);
    
    hasher.finish()
}
```

#### 哈希安全规则

- 🟥 哈希计算前**必须按 LogicalId 排序**，否则遍历顺序不同会导致哈希不一致
- 🟥 使用 `#[derive(DeterministicHash)]` 宏自动为所有战斗 Component 生成哈希代码，防止人工遗漏字段
- 🟥 哈希算法必须跨平台/跨编译器一致（推荐 `xxHash` 或 `FxHash`，避免不同系统下结果不同）

#### 哈希比较

```
回放验证流程：
1. 录制时：每个 TurnEnd 计算 state_hash，存入 ReplayFrame
2. 重放时：相同 TurnEnd 计算 state_hash，与 ReplayFrame 中的值对比
3. 不一致时：记录分歧位置，输出差异详情（对比哪些字段、输出差异日志）
4. 分歧处理：中断回放，输出 diff report 用于排查
```

---

## 3. 规则总结

### 确定性保证链

```
单一种子 → 确定性 PRNG → 所有随机可复现
整数运算 → 精度可控 → 数值结果一致
显式排序 → 迭代顺序确定 → 处理结果一致
状态哈希 → 分歧可检测 → 问题可定位
```

### 不变量

1. **不变量1**：给定相同种子和输入序列，BattleRng 的随机数序列完全一致
2. **不变量2**：所有战斗数值计算结果为整数，无浮点精度差异
3. **不变量3**：相同 ECS 状态下，确定性排序（基于 LogicalId）的查询结果完全一致
4. **不变量4**：每个 TurnEnd 的 state_hash 可复现
5. **不变量5**：整数运算中间值使用 i64 提升，绝不溢出

---

## 4. 禁止事项

| 禁止项 | 理由 | 违反后果 |
|--------|------|---------|
| 🟥 使用 `rand::thread_rng()` | 系统随机不可复现 | 回放失败、测试不可靠 |
| 🟥 使用 `rand::rngs::SmallRng::from_entropy()` | 同上 | 同上 |
| 🟥 在 System 内创建独立 Rng | 破坏全局状态 | 随机序列不可控 |
| 🟥 使用非 ChaCha8Rng 的 PRNG | 跨平台不确定 | 不同平台结果不同 |
| 🟥 使用 Bevy Entity ID 排序 | Entity 不确定 | 跨存档状态分歧 |
| 🟥 核心战斗数值使用 `f32`/`f64` | 浮点精度不可控 | 伤害值不一致 |
| 🟥 核心公式中间值使用 i32 乘法 | 溢出 Panic/Wrap | Debug 崩溃或 Release 异常值 |
| 🟥 依赖 Bevy 默认迭代顺序 | 顺序不确定 | 处理结果不一致 |
| 🟥 同一 Set 内系统无显式排序 | 执行顺序隐式依赖 | 竞态条件 |
| 🟥 系统间存在循环 Set 依赖 | 调度器死锁 | 游戏卡死 |
| 🟥 跳过状态哈希计算 | 分歧不可检测 | Bug 隐藏 |
| 🟥 使用 `println!`/`dbg!` 影响执行路径 | 输出是副作用 | 不确定性 |
| 🟥 ViewModelSet 浮点结果回写逻辑层 | 浮点污染 | 数值分歧 |
| 🟥 回放文件存储实际随机值 | 序列化不兼容 | 回放失败 |
| 🟥 使用时钟时间戳排序事件 | 时钟不确定 | 排序分歧 |

---

## 5. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/domain/replay_rules.md` | 回放格式依赖确定性哈希和种子管理 |
| `docs/architecture/battle_fsm_design.md` | FSM 状态转换依赖确定性执行 |
| `docs/architecture/schedules_design.md` | 系统执行顺序是确定性的基础 |
| `docs/domain/turn_rules.md` | 行动队列排序必须确定性 |
| `docs/domain/battle_rules.md` | Effect Pipeline 执行必须确定性 |
| `docs/architecture.md` | 效果管线、属性系统的确定性要求 |
| `docs/architecture/events_audit_design.md` | 领域事件白名单（宪法 2.2.7）管理所有正式领域事件 |
