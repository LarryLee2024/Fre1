# 确定性执行规范

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第224-230行）

---

## 1. 概述

确定性执行是 SRPG 的**物理定律**：给定相同的输入序列，游戏必须产出完全相同的状态序列。这是回放系统、Bug 复现、自动化测试、未来多人同步的基石。

没有确定性保证：
- 回放系统无法验证（重放结果与录制不一致）
- Bug 无法稳定复现（随机因素导致间歇性出现）
- 自动化测试不可靠（同一测试用例不同运行结果不同）
- 多人联机无法实现（客户端状态分歧）

本规范定义项目中所有影响战斗结果的底层执行约束。

---

## 2. 设计

### 2.1 确定性随机数生成器（PRNG）

所有战斗随机性必须通过**单一确定性 PRNG** 管理。

#### 结构定义

```rust
/// 确定性随机数生成器 Resource
/// 所有战斗随机必须从此获取
#[derive(Resource)]
pub struct BattleRng {
    /// 内部状态（使用 xorshift64* 或类似确定性算法）
    state: u64,
    /// 种子值（存储在回放文件中）
    seed: u64,
}
```

#### 使用规则

| 规则 | 强制等级 | 说明 |
|------|---------|------|
| 所有战斗随机通过 `BattleRng` | 🟥 | 暴击判定、闪避判定、伤害浮动、Buff 触发等 |
| 种子在战斗开始时确定 | 🟥 | 从回放文件或关卡配置读取 |
| `BattleRng` 存储在 `Resource` 中 | 🟥 | 全局唯一，避免多个随机源 |
| 禁止使用 `rand::thread_rng()` | 🟥 | 系统随机不可复现 |
| 禁止使用 `rand::rngs::SmallRng::from_entropy()` | 🟥 | 同上 |
| 禁止在 System 内创建独立 Rng 实例 | 🟥 | 破坏全局状态一致性 |
| 种子格式与回放文件兼容 | 🟩 | 支持从回放文件恢复种子 |

#### 种子来源

```
关卡开始 → 从 LevelConfig 或 ReplaySeed 读取种子
         → 初始化 BattleRng::from_seed(seed)
         → 插入 World Resource
         → 所有战斗系统从此 Resource 获取随机数
```

#### 伪代码示例

```rust
// ✅ 正确：通过 BattleRng 获取随机数
fn check_critical_hit(rng: &mut BattleRng) -> bool {
    rng.gen_range(0..100) < 15  // 15% 暴击率
}

// 🟥 错误：使用系统随机
fn check_critical_hit_bad() -> bool {
    use rand::Rng;
    rand::thread_rng().gen_range(0..100) < 15
}
```

---

### 2.2 数值精度

核心战斗数值**禁止使用浮点数**。

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

---

### 2.3 迭代顺序

Bevy 的 ECS 查询默认**不保证迭代顺序**，这在战斗逻辑中会导致不确定性。

#### 需要排序的场景

| 场景 | 强制等级 | 排序依据 |
|------|---------|---------|
| 行动队列构建 | 🟥 | Initiative 降序 + Entity ID 稳定排序 |
| 同时触发的 Buff 结算 | 🟥 | Buff 注册顺序（InsertionOrder） |
| 同回合多个死亡判定 | 🟥 | Entity ID 升序 |
| 属性 Modifier 栈计算 | 🟥 | ModifierSource 优先级 |
| AOE 伤害目标遍历 | 🟥 | Entity ID 升序 |
| 非战斗逻辑的 ECS 查询 | 🟩 | 无强制要求 |

#### 排序实现规范

```rust
// ✅ 正确：显式排序后处理
fn process_units(mut query: Query<(&Unit, &mut Attributes)>) {
    let mut units: Vec<_> = query.iter_mut().collect();
    units.sort_by_key(|(unit, _)| unit.entity_id);  // 稳定排序
    
    for (unit, attrs) in units {
        // 处理逻辑
    }
}

// 🟥 错误：依赖默认迭代顺序
fn process_units_bad(mut query: Query<(&Unit, &mut Attributes)>) {
    for (unit, attrs) in query.iter_mut() {
        // 顺序不确定
    }
}
```

#### 排序键规范

| 用途 | 排序键 | 方向 |
|------|--------|------|
| 行动顺序 | Initiative | 降序（高先行动） |
| 相同 Initiative | Entity ID | 升序（稳定排序） |
| 伤害结算 | Entity ID | 升序（确定性） |
| Buff 结算 | 插入顺序 | 升序 |
| 目标选择 | Entity ID | 升序 |

---

### 2.4 系统执行顺序

同一 Schedule 内的系统必须有**明确的执行顺序**。

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

### 2.5 状态哈希

每个关键状态边界必须计算**确定性哈希**，用于检测执行分歧。

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
    
    // 1. 所有存活单位的关键属性
    for (unit, attrs) in query.iter() {
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

#### 哈希比较

```
回放验证流程：
1. 录制时：每个 TurnEnd 计算 state_hash，存入 ReplayFrame
2. 重放时：相同 TurnEnd 计算 state_hash，与 ReplayFrame 中的值对比
3. 不一致时：记录分歧位置，输出差异详情
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
3. **不变量3**：相同 ECS 状态下，确定性排序的查询结果完全一致
4. **不变量4**：每个 TurnEnd 的 state_hash 可复现

---

## 4. 禁止事项

| 禁止项 | 理由 | 违反后果 |
|--------|------|---------|
| 🟥 使用 `rand::thread_rng()` | 系统随机不可复现 | 回放失败、测试不可靠 |
| 🟥 使用 `rand::rngs::SmallRng::from_entropy()` | 同上 | 同上 |
| 🟥 在 System 内创建独立 Rng | 破坏全局状态 | 随机序列不可控 |
| 🟥 核心战斗数值使用 `f32`/`f64` | 浮点精度不可控 | 伤害值不一致 |
| 🟥 依赖 Bevy 默认迭代顺序 | 顺序不确定 | 处理结果不一致 |
| 🟥 同一 Set 内系统无显式排序 | 执行顺序隐式依赖 | 竞态条件 |
| 🟥 系统间存在循环 Set 依赖 | 调度器死锁 | 游戏卡死 |
| 🟥 跳过状态哈希计算 | 分歧不可检测 | Bug 隐藏 |
| 🟥 使用 `println!`/`dbg!` 影响执行路径 | 输出是副作用 | 不确定性 |

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
