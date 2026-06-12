# ADR: 统一移动系统架构

## 状态
Proposed

## 背景

当前项目中 AI 和玩家棋子使用相同的底层移动机制（`MovingUnit` 组件、寻路算法、动画系统），但在决策层存在代码重复和逻辑分散：

- **玩家移动**：`src/ui/command_handler.rs` 处理 `UiCommand::MoveUnit`，直接调用寻路 + 添加 `MovingUnit`
- **AI 移动**：`src/ai/decision.rs` 执行决策后，同样调用寻路 + 添加 `MovingUnit`
- **共同模式**：两者都执行 `find_reachable_tiles()` → `reconstruct_path()` → `spawn_path_arrows()` → `insert(MovingUnit)`

这种结构导致：
1. 路径构建逻辑在两处重复
2. 移动速度硬编码为 `0.15` 秒/格
3. 新增移动相关功能（如延迟移动、移动轨迹记录）需修改两处代码
4. AI 和玩家的移动行为可能因实现差异产生不一致

## 决策

### 核心原则：意图与执行分离

将移动拆分为两个阶段：

```
决策阶段（Intent）→ 执行阶段（Execution）
```

**决策阶段**：确定"去哪里"
- 玩家：从 UI 输入获取目标坐标
- AI：从决策系统计算目标坐标
- 输出：统一的 `MovementIntent` 事件

**执行阶段**：确定"怎么去"
- 验证移动合法性（在范围内、路径可达）
- 计算路径并显示导航箭头
- 添加 `MovingUnit` 触发动画
- 输出：统一的移动执行逻辑

### 架构设计

#### 1. 引入 `MovementIntent` 事件

```rust
/// 移动意图事件 - 决策层的输出
#[derive(Event)]
pub struct MovementIntent {
    pub entity: Entity,
    pub target_coord: IVec2,
    pub source: IntentSource,
}

#[derive(Clone, Copy)]
pub enum IntentSource {
    Player,  // 玩家输入
    Ai,      // AI 决策
}
```

**职责**：跨层通信，将决策结果传递给执行层

**订阅者**：`movement_execution_system`

#### 2. 提取 `MovementExecutor` 系统

```rust
/// 移动执行系统 - 监听 MovementIntent，统一处理移动逻辑
pub fn movement_execution_system(
    mut commands: Commands,
    mut intent_reader: EventReader<MovementIntent>,
    // ... 其他资源
) {
    for intent in intent_reader.read() {
        execute_movement(&mut commands, intent);
    }
}

/// 统一的移动执行逻辑
fn execute_movement(commands: &mut Commands, intent: &MovementIntent) {
    // 1. 验证移动合法性
    // 2. 计算路径
    // 3. 显示导航箭头
    // 4. 添加 MovingUnit
}
```

**职责**：封装所有移动执行细节，确保 AI 和玩家行为一致

#### 3. 重构现有代码

**玩家侧** (`src/ui/command_handler.rs`)：
```rust
// 之前：直接调用寻路 + 添加 MovingUnit
// 之后：发送 MovementIntent 事件
commands.trigger(MovementIntent {
    entity: selected_entity,
    target_coord,
    source: IntentSource::Player,
});
```

**AI 侧** (`src/ai/decision.rs`)：
```rust
// 之前：直接调用寻路 + 添加 MovingUnit
// 之后：发送 MovementIntent 事件
commands.trigger(MovementIntent {
    entity: ai_entity,
    target_coord: best_coord,
    source: IntentSource::Ai,
});
```

#### 4. 移动速度配置化

从硬编码改为从单位属性读取：

```rust
let move_speed = character_attrs.get(AttributeKind::MoveSpeed)
    .unwrap_or(DEFAULT_MOVE_SPEED);

MovingUnit {
    speed: move_speed,  // 从属性读取
    ...
}
```

或在 `CharacterDefinition` 中添加 `move_animation_speed` 字段。

### 边界定义

**统一移动系统负责**：
- 路径计算和验证
- 移动动画触发
- 导航箭头显示
- 位置更新

**统一移动系统不负责**：
- 目标选择（由决策层决定）
- 移动范围计算（由寻路模块提供）
- 地形成本计算（由 `TerrainCostCalculator` 提供）

### 迁移策略

1. **第一阶段**：创建 `MovementIntent` 事件和 `movement_execution_system`
2. **第二阶段**：重构玩家移动逻辑，改为发送事件
3. **第三阶段**：重构 AI 移动逻辑，改为发送事件
4. **第四阶段**：删除重复的路径构建代码
5. **第五阶段**：添加回归测试验证行为一致性

### 测试策略

**单元测试**（`tests/rule/`）：
- 测试 `MovementIntent` 事件的序列化和反序列化（如有需要）
- 测试移动速度从属性读取的逻辑

**功能测试**（`tests/feature/`）：
- 测试玩家移动：发送 `MovementIntent { source: Player }`，验证 `MovingUnit` 正确添加
- 测试 AI 移动：发送 `MovementIntent { source: Ai }`，验证 `MovingUnit` 正确添加
- 测试非法移动：目标超出范围时，验证不添加 `MovingUnit`

**场景测试**（`tests/scenario/`）：
- 测试完整回合：玩家移动 → AI 移动，验证两者行为一致
- 测试多单位移动：验证事件处理顺序不影响最终结果

**回归测试**：
- 复用现有移动相关测试用例，确保重构后行为不变

## 后果

### 正面影响

1. **单一真相源**：移动执行逻辑集中在一处，修改只需一处
2. **行为一致性**：AI 和玩家使用完全相同的移动执行路径，消除潜在差异
3. **易于扩展**：新增移动相关功能（如移动轨迹、延迟移动）只需修改执行系统
4. **符合 ECS 原则**：事件驱动，系统间松耦合
5. **便于测试**：可单独测试执行系统，无需区分 AI/玩家

### 负面影响

1. **增加间接层**：从直接调用改为事件驱动，调试时需要追踪事件流
2. **初期工作量**：需要重构两处现有代码
3. **性能开销**：事件系统有微小开销（可忽略，移动频率低）

### 风险缓解

- **调试复杂性**：通过清晰的日志和 tracing 记录事件流转
- **回归风险**：在重构前后运行相同的集成测试套件
- **性能问题**：移动事件频率低（每回合几次），事件开销可忽略

## 替代方案

### 方案 B：提取辅助函数（不采用）

提取 `execute_movement()` 辅助函数供双方调用。

**拒绝原因**：
- 仍然是命令式调用，不符合 ECS 事件驱动风格
- 无法解耦决策和执行，测试时仍需构造完整上下文
- 未来若需异步移动或延迟执行，辅助函数难以扩展

### 方案 C：保持现状（不采用）

维持当前 AI 和玩家各自实现移动逻辑。

**拒绝原因**：
- 代码重复违反 DRY 原则
- 新增功能需修改多处，容易遗漏
- 长期维护成本高，容易产生行为差异

## 参考

- `src/character/movement.rs` - 当前移动动画系统
- `src/ai/decision.rs` - 当前 AI 移动决策
- `src/ui/command_handler.rs` - 当前玩家移动处理
- `src/map/pathfinding/mod.rs` - 寻路算法
- AGENTS.md - ECS 架构约束（实体即 ID、组件=数据、系统=行为）
