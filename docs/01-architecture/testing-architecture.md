---
id: 01-architecture.testing-architecture
title: Testing Architecture
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - layer
---

# Testing Architecture — 完整测试体系架构

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第五节 — 完整测试体系

本文档定义 SRPG 项目的五层测试体系，涵盖领域单元测试、领域集成测试、系统集成测试、端到端回归测试和 Testbeds 调试沙盒。

交叉引用：
- `docs/01-architecture/README.md` — 测试总纲（所有功能必须优先编写测试）
- `docs/01-architecture/layer-contracts.md` — 七层架构边界定义
- `docs/test_spec.md` — 测试体系规范（如已存在）

---

## 1. 设计原则

### 1.1 四大核心原则

1. **分层对齐**：测试层级与架构层级一一对应，每层只验证自身职责，不越界测试下层实现
2. **权责封闭**：领域规则测试绝不依赖 Bevy 运行时，系统测试绝不硬编码业务数值
3. **确定性优先**：所有可自动化测试必须可复现，随机、时间、迭代顺序全部可控（宪法 §18.4.1）
4. **测试即工具**：测试资产同时是开发调试资产，测试夹具、沙盒环境可被全项目复用

### 1.0 宪法测试条款对齐

> **以下条款为宪法强制要求，本测试架构必须完全覆盖**：

| 宪法条款 | 要求 | 本文档覆盖 |
|---------|------|-----------|
| §18.1.1 测试优先 | 所有功能必须优先编写测试 | ✅ 分层测试体系 |
| §18.1.2 Bug 修复流程 | 先写重现测试，再修复 | ✅ 测试失败快照机制 |
| §18.2.1 三层测试体系 | 单元 → 集成 → 回放 | ✅ 五层扩展（含回放） |
| §18.2.2 SRPG 测试优先 | 复杂逻辑优先用 Battle Replay | ✅ replay_validator |
| §18.3.1 Builder 模式 | TestCharacterBuilder、TestBattleBuilder | ✅ shared/testing/fixtures |
| §18.3.2 Golden Test | 战斗回放、配置导出用金文件对比 | ✅ replay_validator |
| §18.4.1 确定性 | 同状态+同输入+同种子=同结果 | ✅ DeterministicRng |
| §18.4.2 禁止系统时间 | 核心逻辑用 GameTime 服务 | ⚠️ 待实现 |
| §18.4.3 战斗 Bug 规范 | 战斗 Bug → Battle Replay → 永久测试 | ✅ failure_snapshots |
| §18.4.5 资产沉淀 | 修复的 Bug → 测试资产 | ✅ failure_snapshots |

### 1.2 测试金字塔

> **宪法 §18.2.1 对齐**：宪法规定三层测试体系（单元测试 → 集成测试 → 回放测试）。本文档扩展为五层，将"集成测试"细分为"领域集成"和"系统集成"，并新增 Testbeds 调试层。核心三层与宪法完全对齐。

```
           ┌──────────┐
           │  E2E     │  少量，CI 门禁
           ├──────────┤
          │  系统集成  │  中量，验证架构落地
         ├────────────┤
        │  领域集成    │  大量，验证跨模块规则
       ├──────────────┤
      │  领域单元测试  │  最多，钉死单条规则
     ├────────────────┤
    │  Testbeds 沙盒   │  交互式调试资产
     └────────────────┘
```

### 1.3 目标比例（精确化）

> **优化来源**：`docs/其他/69.md` — 测试金字塔比例精确化

| 测试类型 | 目标占比 | 执行速度 | 覆盖目标 | 典型用例数 |
|---------|---------|---------|---------|-----------|
| 领域单元测试 | **70%** | 毫秒级 | 每条领域规则至少一组 | 500+ |
| 领域集成测试 | **15%** | 百毫秒级 | 跨模块链路完整覆盖 | 100+ |
| 系统集成测试 | **10%** | 秒级 | 核心系统调度与交互 | 50+ |
| E2E 回归测试 | **5%** | 分钟级 | 主干流程不崩溃 | 20+ |

覆盖率指标：
- 🟩 领域规则覆盖率：**100%**（每条规则必须有对应测试）
- 🟩 核心 ECS 系统覆盖率：**≥ 90%**（伤害/Buff/回合系统）
- 🟩 E2E 主流程覆盖率：**100%**（进入战斗→操作→胜利/失败）

### 1.4 领域层零 Bevy 依赖的 TDD 工作流

> **优化来源**：`docs/其他/69.md` — 领域层零 Bevy 依赖的 TDD 工作流示例

TDD（测试驱动开发）在领域层的最佳实践：先写测试，再实现，再验证。由于领域层零 Bevy 依赖，TDD 循环极快：

```rust
// ═══ Step 1: 写测试（先失败） ═══
// core/battle/tests/damage_calculation.rs
#[test]
fn critical_hit_doubles_damage() {
    let attacker = Attributes { attack: 100, crit_rate: 100.0, ..Default::default() };
    let defender = Attributes { defense: 20, ..Default::default() };
    let modifiers = vec![Modifier::Critical { multiplier: 2.0 }];

    let result = calculate_damage(100, &attacker, &defender, &modifiers);

    assert_eq!(result.value, 180); // (100 * 2.0) - 20 = 180
}

// ═══ Step 2: 实现（让测试通过） ═══
// core/battle/damage.rs
pub fn calculate_damage(
    base: i32,
    attacker: &Attributes,
    defender: &Attributes,
    modifiers: &[Modifier],
) -> DamageResult {
    let mut damage = base as f32;
    for m in modifiers {
        match m {
            Modifier::Critical { multiplier } => damage *= multiplier,
            _ => {}
        }
    }
    let final_damage = (damage - defender.defense as f32).max(1.0) as i32;
    DamageResult { value: final_damage }
}

// ═══ Step 3: 验证（cargo test，毫秒级完成） ═══
// $ cargo test critical_hit -- --nocapture
// test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**TDD 循环速度**：由于领域层零 Bevy 依赖，每次测试运行只需 < 100ms，开发者可以每 5-10 秒完成一次 TDD 循环。这比"改代码→重新编译 Bevy→启动游戏→手动验证"的反馈循环快 100 倍。

---

## 2. 五层测试定义

### 2.1 Layer 1：领域单元测试（纯逻辑，零运行时依赖）

**存放位置**：`core/[领域模块]/tests/`

**核心定位**：验证单条游戏规则的正确性，是测试金字塔的底座。

#### 测试范围

- 纯数值公式：伤害计算、属性加成、堆叠规则、消耗计算
- 单领域逻辑：技能命中判定、Buff 生效/失效规则、回合推进条件
- 边界与异常：非法输入处理、错误类型返回、极值下的规则表现

#### 强制约束

- 🟥 **绝对禁止引入 Bevy 任何依赖**，不使用 ECS、不加载资源
- 🟥 **不依赖任何 content 配置数据**，测试用例硬编码入参
- 🟩 全部为纯函数测试，输入输出一一对应，毫秒级执行
- 🟩 每条领域规则至少对应一组单元测试

#### 典型示例

```rust
// core/battle/tests/damage_calculation.rs
#[test]
fn physical_damage_respects_armor_reduction() {
    let attacker = StatBundle { attack: 100, ..Default::default() };
    let defender = StatBundle { armor: 30, ..Default::default() };
    let result = calculate_physical_damage(&attacker, &defender);
    assert_eq!(result.value, 70);
}

#[test]
fn damage_never_below_one() {
    let attacker = StatBundle { attack: 1, ..Default::default() };
    let defender = StatBundle { armor: 999, ..Default::default() };
    let result = calculate_physical_damage(&attacker, &defender);
    assert!(result.value >= 1, "Damage floor violated");
}
```

#### 对应领域规则文档

| 领域规则文档 | 对应测试文件 |
|------------|------------|
| `formula_rules.md` | `core/battle/tests/damage_calculation.rs` |
| `skill_rules.md` | `core/skill/tests/skill_validation.rs` |
| `stack_policy_rules.md` | `core/buff/tests/stack_policy.rs` |
| `attribute_rules.md` | `core/character/tests/attribute_modifier.rs` |
| `pathfinding_rules.md` | `core/map/tests/pathfinding.rs` |

---

### 2.2 Layer 2：领域集成测试（跨模块规则联动）

**存放位置**：`core/[聚合根模块]/tests/`（如 `core/battle/tests/`）

**核心定位**：验证多个领域模块组合后的规则一致性，不涉及引擎运行时。

#### 测试范围

- 跨领域链路：技能释放 → 伤害结算 → Buff 挂载 → 属性变更 全链路
- 状态流转：回合开始 → 行动 → 结算 → 回合结束的完整状态机
- 聚合不变量：战斗结束后所有单位状态合法、Buff 数量不超限等

#### 技术要点

- 仍然是纯 Rust 代码，不启动 Bevy
- 使用领域对象直接组合，模拟完整战斗上下文
- 重点验证模块间的契约是否符合 `layer-contracts.md` 约定

#### 典型示例

```rust
// core/battle/tests/full_combat_flow.rs
#[test]
fn skill_cast_damage_buff_full_flow() {
    // 1. 创建攻击者和目标
    let mut attacker = create_test_character(100, 50, 20);
    let mut target = create_test_character(200, 10, 30);
    
    // 2. 释放技能
    let skill = create_test_skill(SkillType::Fireball, 80, vec![BuffEffect::Burn]);
    let intent = CombatIntent::skill(&attacker, &skill, &target);
    
    // 3. 执行效果管线
    let result = execute_combat_intent(&intent, &mut attacker, &mut target);
    
    // 4. 验证伤害
    assert!(result.damage > 0, "Fireball should deal damage");
    
    // 5. 验证 Buff 施加
    assert!(target.has_buff(BuffType::Burn), "Burn should be applied");
    
    // 6. 验证状态合法
    assert!(target.hp() > 0 || target.is_dead());
    assert!(target.buff_count() <= MAX_BUFFS_PER_UNIT);
}
```

---

### 2.3 Layer 3：系统集成测试（Bevy ECS 运行时验证）

**存放位置**：`tests/integration/` 或对应插件模块内

**核心定位**：验证架构在 Bevy 引擎中的落地正确性，确认 ECS 系统、事件、组件的交互符合设计预期。

#### 测试范围

- 插件装配：注册指定 Plugin 后，必要的 Resource、System 是否正常初始化
- 系统调度：系统执行顺序是否符合 `schedules` 设计，是否存在顺序依赖 bug
- 事件流转：发送领域事件后，对应系统是否正确触发、组件是否正确变更
- ECS 通信：跨模块通过事件/组件交互是否符合 `docs/02-domain/ecs_communication_rules.md`

#### 技术要点

- 使用 Bevy 原生测试能力，运行 headless 无头模式
- 通过 `World::new()` 手动构造最小测试环境，只加载被测插件
- 重点测「架构落地」，不测「业务规则」——业务规则已经在领域层测完

#### 典型示例

```rust
// tests/integration/battle_plugin_test.rs
use bevy::prelude::*;

#[test]
fn battle_plugin_initializes_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(BattlePlugin);
    
    // 验证必要 Resource 被初始化
    assert!(app.world().contains_resource::<TurnOrder>());
    assert!(app.world().contains_resource::<BattleRecord>());
}

#[test]
fn damage_event_triggers_combat_log() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(BattlePlugin);
    
    // 发送伤害事件
    app.world_mut().write_message(DamageApplied {
        source: UnitId::new(1),
        target: UnitId::new(2),
        damage: 50,
        is_critical: false,
    });
    
    // 运行一帧
    app.update();
    
    // 验证战斗日志被更新
    let record = app.world().resource::<BattleRecord>();
    assert_eq!(record.entries().len(), 1);
}
```

---

### 2.4 Layer 4：端到端回归测试（全链路黑盒）

**存放位置**：项目根目录 `tests/`

**核心定位**：验证完整游戏的可用性与稳定性，作为 CI 门禁的最后一道防线。

#### 测试分类

| 分类 | 测试内容 | 执行频率 |
|------|---------|----------|
| 主流程测试 | 进入战斗 → 执行操作 → 推进回合 → 胜负判定 完整流程 | 每次 PR |
| 存档兼容测试 | 各版本存档能否正常加载、数据不丢失 | 发布前 |
| MOD 兼容测试 | 标准 MOD 能否正常加载、不破坏基础游戏逻辑 | 发布前 |
| 错误路径测试 | 异常输入、非法配置下游戏是否可控报错，不崩溃 | 每周回归 |

#### 强制约束

- 🟩 以黑盒方式调用游戏公共接口，不访问任何内部私有结构
- 🟩 用例只覆盖核心主干，不追求覆盖细节，细节由下层测试保障
- 🟩 执行耗时控制在分钟级，避免成为 CI 瓶颈

#### 典型示例

```rust
// tests/battle_flow/complete_battle.rs
#[test]
fn player_can_win_battle_through_combat() {
    let mut game = TestGame::new();
    
    // 1. 进入关卡
    game.start_stage("test_stage_01");
    assert_eq!(game.state(), AppState::InGame);
    
    // 2. 选择单位
    game.select_unit(UnitId::new(1));
    
    // 3. 移动到敌人附近
    game.move_unit(UnitId::new(1), IVec2::new(3, 5));
    
    // 4. 攻击敌人
    game.attack(UnitId::new(2));
    
    // 5. 推进回合
    game.end_turn();
    
    // 6. 验证敌人死亡
    assert!(game.is_unit_dead(UnitId::new(2)));
    
    // 7. 验证胜利
    assert_eq!(game.state(), AppState::GameOver);
    assert!(game.is_victory());
}
```

---

### 2.5 Layer 5：Testbeds 测试沙盒（开发调试核心资产）

**存放位置**：项目根目录 `testbeds/`

**核心定位**：可运行、可视化、可交互的调试沙盒，是 SRPG 项目长期开发的效率杠杆。

#### 核心沙盒

##### 1. battle_simulator 战斗模拟器

- 可加载任意地图、任意单位配置，快速启动一场完整战斗
- 支持单步执行、回退、状态快照，用于调试战斗逻辑
- 录制的战斗流程可直接导出为回放文件，自动转为回归测试用例

##### 2. skill_playground 技能调试台

- 单独加载技能系统，实时调整技能参数，立即看到结算结果
- 用于技能设计、数值调试，不需要启动完整游戏
- 支持批量跑技能用例，快速验证数值平衡性

##### 3. ai_debug_arena AI 调试场

- 可视化展示 AI 决策树、权重计算、最终选择
- 固定场景下反复验证 AI 行为一致性，排查决策异常
- 对应 `ai_rules.md`，是 AI 规则落地的核心调试工具

##### 4. balance_workbench 数值工作台

- 批量执行指定配置的战斗，统计伤害分布、胜率、回合时长
- 用于平衡性验证，支持多版本配置对比
- 对接 content 数据，配置修改后一键跑基准测试

##### 5. replay_validator 回放验证器

> **宪法 §18.2.2 SRPG 测试优先**：复杂 SRPG 逻辑必须优先使用 Battle Replay 测试，而非 BDD。replay_validator 是确定性保障的核心工具。

> **宪法 §18.3.2 Golden Test 对齐**：战斗回放、配置导出等稳定输出，必须使用金文件对比测试。版本升级后输出变化必须显式确认。

- 批量重放历史回放文件，校验同一输入是否得到完全相同的结果
- 是确定性保障的核心工具，每次引擎升级、架构重构后必跑
- 自动对比状态哈希，输出不一致的帧位置与差异点

#### Testbeds 技术要求

- 🟩 每个 Testbed 必须可以独立运行（`cargo run --testbed battle_simulator`）
- 🟩 每个 Testbed 必须有 README 说明使用方法
- 🟩 Testbeds 代码可以引用内部实现（开发工具权限更宽）
- 🟥 Testbeds 代码永不进入发布构建
- 🟥 **所有 Testbeds 必须使用 DeterministicRng，禁止依赖系统时间**（宪法 §18.4.2 禁止业务逻辑依赖系统时间）

---

## 3. 目录结构

```
project/
├── src/
│   ├── core/
│   │   ├── battle/
│   │   │   └── tests/      # 领域单元测试 + 领域集成测试
│   │   ├── skill/
│   │   │   └── tests/
│   │   ├── buff/
│   │   │   └── tests/
│   │   ├── character/
│   │   │   └── tests/
│   │   ├── equipment/
│   │   │   └── tests/
│   │   ├── inventory/
│   │   │   └── tests/
│   │   ├── map/
│   │   │   └── tests/
│   │   └── ai/
│   │       └── tests/
│   │
│   ├── infrastructure/
│   │   ├── persistence/
│   │   │   └── tests/      # 基础设施单元测试
│   │   └── ...
│   │
│   └── shared/
│       └── testing/        # 全项目公共测试工具库
│           ├── mod.rs
│           ├── fixtures.rs     # 测试夹具
│           ├── assertions.rs   # 自定义断言
│           ├── deterministic.rs # 确定性基础设施
│           └── time.rs         # 模拟时间
│
├── tests/                   # E2E 端到端回归测试
│   ├── battle_flow/
│   │   ├── complete_battle.rs
│   │   └── victory_conditions.rs
│   ├── save_load/
│   │   └── roundtrip.rs
│   └── mod_compatibility/
│       └── basic_mod.rs
│
├── testbeds/                # 可运行测试沙盒
│   ├── battle_simulator/
│   │   ├── main.rs
│   │   └── README.md
│   ├── skill_playground/
│   │   ├── main.rs
│   │   └── README.md
│   ├── ai_debug_arena/
│   │   ├── main.rs
│   │   └── README.md
│   ├── balance_workbench/
│   │   ├── main.rs
│   │   └── README.md
│   └── replay_validator/
│       ├── main.rs
│       └── README.md
│
└── content/
    └── tests/               # 内容数据校验
        ├── reference_integrity.rs
        ├── value_validity.rs
        └── schema_compliance.rs
```

---

## 4. shared/testing 公共测试库

> **宪法 §18.3.1 对齐**：核心测试必须使用 Builder 模式构造测试数据（TestCharacterBuilder、TestBattleBuilder），禁止每个测试手动构造大量实体与组件。

### 4.1 测试夹具库（Fixture）

```rust
// shared/testing/fixtures.rs

/// 创建标准测试角色
pub fn create_test_character(
    hp: i32,
    attack: i32,
    defense: i32,
) -> Character {
    Character {
        attributes: Attributes {
            max_hp: hp,
            current_hp: hp,
            attack,
            defense,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// 创建标准 1v1 战斗场景
pub fn standard_1v1_battle() -> BattleContext {
    let attacker = create_test_character(100, 50, 20);
    let defender = create_test_character(200, 30, 30);
    BattleContext::new(vec![attacker], vec![defender])
}

/// 创建标准团队战斗场景
pub fn full_party_battle() -> BattleContext {
    let party = (0..4).map(|i| {
        create_test_character(100 + i * 20, 40 + i * 5, 20 + i * 3)
    }).collect();
    let enemies = (0..3).map(|i| {
        create_test_character(150 + i * 30, 35 + i * 5, 25 + i * 3)
    }).collect();
    BattleContext::new(party, enemies)
}
```

### 4.2 确定性基础设施

> **宪法 §18.4.1 强制要求**：相同初始状态 + 相同输入序列 + 相同 RNG 种子，必须得到完全一致的战斗结果。这是 Battle Replay、Bug 复现、自动化测试的核心基础。

> **优化来源**：`docs/其他/69.md` — DeterministicRng + hash_game_state 用于回放验证的完整代码示例

```rust
// shared/testing/deterministic.rs

/// 固定种子随机数生成器
pub struct DeterministicRng {
    seed: u64,
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { seed, state: seed }
    }
    
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn fork(&self, tag: u64) -> Self {
        Self::new(self.seed ^ tag)
    }
}

/// 状态哈希工具：用于回放验证和回归测试
pub fn hash_game_state(
    units: &[(Entity, &Attributes, &GridPosition, &ActiveBuffs)],
    turn: &TurnOrder,
    phase: &BattlePhase,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    // 哈希所有单位状态
    for (entity, attrs, pos, buffs) in units {
        entity.hash(&mut hasher);
        attrs.max_hp.hash(&mut hasher);
        attrs.current_hp.hash(&mut hasher);
        attrs.attack.hash(&mut hasher);
        attrs.defense.hash(&mut hasher);
        pos.x.hash(&mut hasher);
        pos.y.hash(&mut hasher);
        buffs.0.hash(&mut hasher); // Buff 列表哈希
    }

    // 哈希回合状态
    turn.current_index().hash(&mut hasher);
    phase.hash(&mut hasher);

    hasher.finish()
}

/// 回放验证：录制 + 重放 + 哈希对比
pub struct ReplayRecorder {
    seed: u64,
    frames: Vec<FrameSnapshot>,
}

pub struct FrameSnapshot {
    frame: u64,
    state_hash: u64,
    actions: Vec<PlayerAction>,
}

impl ReplayRecorder {
    pub fn new(seed: u64) -> Self {
        Self { seed, frames: Vec::new() }
    }

    pub fn record_frame(&mut self, frame: u64, state_hash: u64, actions: Vec<PlayerAction>) {
        self.frames.push(FrameSnapshot { frame, state_hash, actions });
    }

    /// 重放录制的操作序列，验证每帧状态哈希是否一致
    pub fn verify_replay(&self) -> Result<(), ReplayError> {
        let mut rng = DeterministicRng::new(self.seed);
        let mut world = create_test_world(&mut rng);

        for recorded in &self.frames {
            // 执行录制的操作
            for action in &recorded.actions {
                execute_action(&mut world, action, &mut rng);
            }

            // 推进一帧
            world.advance_frame();

            // 计算当前状态哈希
            let current_hash = hash_game_state(
                &world.units(),
                &world.turn_order(),
                &world.battle_phase(),
            );

            // 对比哈希
            if current_hash != recorded.state_hash {
                return Err(ReplayError::HashMismatch {
                    frame: recorded.frame,
                    expected: recorded.state_hash,
                    actual: current_hash,
                });
            }
        }

        Ok(())
    }
}
```

### 4.3 自定义断言

```rust
// shared/testing/assertions.rs

/// 数值容差断言
pub fn assert_approx_eq(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "Assertion failed: {} ≈ {} (tolerance: {})",
        actual, expected, tolerance
    );
}

/// 状态不变量断言
pub fn assert_game_state_valid(world: &World) {
    let results = validate_game_state(world);
    for result in results {
        match result {
            ValidationResult::Valid => {}
            ValidationResult::Reject { field, reason } => {
                panic!("Game state invalid: {} — {}", field, reason);
            }
            _ => {}
        }
    }
}

/// 错误类型断言
pub fn assert_error_type<T: std::fmt::Debug>(
    result: Result<(), T>,
    expected_error: &str,
) {
    match result {
        Ok(()) => panic!("Expected error '{}' but got Ok(())", expected_error),
        Err(e) => {
            let error_str = format!("{:?}", e);
            assert!(
                error_str.contains(expected_error),
                "Expected error '{}' but got '{}'",
                expected_error, error_str
            );
        }
    }
}
```

---

## 5. 内容数据测试

**存放位置**：`content/tests/`

> **优化来源**：`docs/其他/69.md` — Content 测试独立成层的完整流程（RON 引用完整性/数值合法性/格式合规性 CI 集成）

### 5.1 测试范围

- **引用完整性**：技能引用的 Buff ID、特效 ID 是否真实存在
- **数值合法性**：伤害是否为正、冷却时间是否越界、属性加成比例是否超限
- **格式合规性**：所有配置文件是否符合 schema、必填字段是否缺失
- **规则一致性**：配置数值是否符合领域规则约束（如堆叠数量上限）

### 5.2 完整测试流程

```
RON 文件变更（git diff）
    ↓
CI 触发 content/tests/
    ↓
┌─────────────────────────────────────────────────┐
│ Step 1: 引用完整性检查                            │
│   扫描所有 SkillDef → 验证引用的 BuffId 存在       │
│   扫描所有 BuffDef → 验证引用的 EffectDef 存在     │
│   扫描所有 MapDef → 验证引用的 UnitTemplate 存在   │
├─────────────────────────────────────────────────┤
│ Step 2: 数值合法性检查                            │
│   伤害值 > 0，冷却时间 ∈ [0, 99]                  │
│   属性加成比例 ∈ [-100%, +500%]                   │
│   Buff 叠层数 ∈ [1, 99]                          │
├─────────────────────────────────────────────────┤
│ Step 3: 格式合规性检查                            │
│   所有必填字段存在                                 │
│   枚举值在合法范围内                               │
│   文件名与内部 id 一致                             │
├─────────────────────────────────────────────────┤
│ Step 4: 规则一致性检查                            │
│   技能消耗不超过属性上限                           │
│   Buff 持续时间不为 0                              │
│   公式引用的属性存在                               │
└─────────────────────────────────────────────────┘
    ↓
通过 → CI 继续
失败 → 阻塞合并，输出详细错误报告
```

### 5.3 典型测试

```rust
// content/tests/reference_integrity.rs
#[test]
fn all_skill_buff_references_exist() {
    let skills = load_all_skills();
    let buff_registry = load_buff_registry();
    
    for skill in &skills {
        for buff_ref in &skill.buff_effects {
            assert!(
                buff_registry.contains(buff_ref.buff_id),
                "Skill '{}' references non-existent buff '{}'",
                skill.id, buff_ref.buff_id
            );
        }
    }
}

// content/tests/value_validity.rs
#[test]
fn all_damage_values_positive() {
    let skills = load_all_skills();
    for skill in &skills {
        for effect in &skill.effects {
            if let EffectDef::Damage(amount) = effect {
                assert!(
                    *amount > 0,
                    "Skill '{}' has non-positive damage: {}",
                    skill.id, amount
                );
            }
        }
    }
}

// content/tests/schema_compliance.rs
#[test]
fn all_skills_have_required_fields() {
    let skills = load_all_skills();
    for skill in &skills {
        assert!(!skill.id.is_empty(), "Skill missing id");
        assert!(!skill.name.is_empty(), "Skill '{}' missing name", skill.id);
        assert!(skill.cooldown >= 0, "Skill '{}' has negative cooldown", skill.id);
    }
}
```

### 5.4 CI 集成

```yaml
# .github/workflows/content-validation.yml
- name: Content Validation
  run: |
    # 仅在 content/ 目录有变更时运行
    if git diff --name-only HEAD~1 | grep -q "^content/"; then
      cargo test --test content -- --nocapture
    else
      echo "No content changes, skipping validation"
    fi
```

---

## 6. CI 集成

### 6.1 执行频率

| 测试类型 | 执行频率 | 触发条件 |
|---------|---------|---------|
| 领域单元测试 | 每次提交 | `cargo test` |
| 领域集成测试 | 每次提交 | `cargo test` |
| 系统集成测试 | 每次 PR | CI 流水线 |
| E2E 回归测试 | 每次 PR | CI 流水线 |
| 内容数据测试 | 每次 PR | CI 流水线 |
| 性能基准测试 | 每次 PR | CI 流水线（不阻塞合并） |

### 6.2 CI 流水线

```
代码提交
  ↓
编译检查（cargo check）
  ↓
Lint 检查（cargo clippy）
  ↓
领域单元测试（cargo test --lib）
  ↓
领域集成测试（cargo test --test integration）
  ↓
系统集成测试（cargo test --test system）
  ↓
内容数据测试（cargo test --test content）
  ↓
E2E 回归测试（cargo test --test e2e）
  ↓
性能基准测试（cargo bench，不阻塞）
  ↓
报告结果
```

### 6.3 测试失败处理

- 🟥 单元测试失败：阻塞合并
- 🟥 集成测试失败：阻塞合并
- 🟥 E2E 测试失败：阻塞合并
- 🟩 性能基准回归 >10%：标记警告，不阻塞合并

### 6.4 测试失败快照机制

> **宪法 §18.1.2 Bug 修复流程对齐**：发现 Bug 后必须先编写重现测试，再修复 Bug。失败快照机制确保 Bug 可复现，自动转化为永久测试用例。

> **优化来源**：`docs/其他/69.md` — 测试失败快照机制（自动保存失败时的战斗状态快照便于复现）

当集成测试或 E2E 测试失败时，自动保存当前游戏状态快照，便于本地复现和调试：

```rust
// shared/testing/snapshot.rs

use std::path::PathBuf;

/// 测试失败时自动保存状态快照
pub struct FailureSnapshot {
    output_dir: PathBuf,
}

impl FailureSnapshot {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self { output_dir: output_dir.into() }
    }

    /// 保存战斗状态快照
    pub fn save_battle_snapshot(
        &self,
        test_name: &str,
        world: &World,
        frame: u64,
    ) -> PathBuf {
        let snapshot_dir = self.output_dir.join("failure_snapshots");
        std::fs::create_dir_all(&snapshot_dir).unwrap();

        let filename = format!("{}_frame_{}.ron", test_name, frame);
        let path = snapshot_dir.join(&filename);

        // 序列化关键状态到 RON
        let snapshot = BattleSnapshot {
            frame,
            timestamp: chrono::Utc::now(),
            units: serialize_units(world),
            turn_order: serialize_turn_order(world),
            buffs: serialize_buffs(world),
            map_state: serialize_map(world),
        };

        let ron_string = ron::to_string(&snapshot).unwrap();
        std::fs::write(&path, ron_string).unwrap();

        eprintln!("📸 Failure snapshot saved: {}", path.display());
        path
    }
}

// 在测试中使用
#[test]
fn complex_buff_interaction() {
    let mut app = setup_test_app();
    let snapshot = FailureSnapshot::new("test_output");

    // ... 执行测试逻辑 ...

    if let Err(e) = assertion_result {
        // 测试失败时自动保存快照
        snapshot.save_battle_snapshot(
            "complex_buff_interaction",
            app.world(),
            current_frame,
        );
        panic!("Test failed: {}", e);
    }
}
```

快照输出格式：
```
test_output/failure_snapshots/
├── complex_buff_interaction_frame_42.ron    # 失败时的状态快照
├── complex_buff_interaction_frame_42.meta   # 元数据（测试名、时间、RNG 种子）
└── ...
```

开发者可以：
1. 查看快照文件，了解失败时的完整游戏状态
2. 用 `battle_simulator` 沙盒加载快照，交互式调试
3. 🟥 将快照转换为回归测试用例，防止同类问题再次发生（宪法 §18.4.5 所有修复的 Bug 最终都必须沉淀为测试资产）

---

## 7. 测试编写规范

### 7.1 命名规范

- 🟩 测试函数名描述预期行为：`physical_damage_respects_armor_reduction`
- 🟩 测试文件名描述测试范围：`damage_calculation.rs`
- 🟩 模块内测试使用 `#[cfg(test)] mod tests`

### 7.2 测试组织

- 🟩 每个测试函数只验证一个行为
- 🟩 测试之间相互独立，不共享状态
- 🟩 使用 `setup` 函数准备测试环境
- 🟩 使用 `teardown` 函数清理测试环境

### 7.3 测试质量

- 🟩 覆盖正常路径、边界条件、异常输入
- 🟩 不依赖特定 RNG 输出（使用种子控制）
- 🟩 不依赖特定执行顺序（测试独立）
- 🟩 不依赖外部文件或网络

---

## 8. 禁止事项

- 🟥 **领域层测试引入 Bevy 依赖**（领域测试必须是纯 Rust）
- 🟥 **测试依赖特定 RNG 输出而不控制种子**
- 🟥 **测试之间共享可变状态**
- 🟥 **通过修改业务逻辑让测试通过**
- 🟥 **通过修改测试适配错误逻辑**
- 🟥 **删除测试来消除失败**
- 🟥 **测试函数验证多个不相关的行为**
- 🟥 **测试中硬编码业务数值（应该使用 fixtures）**
- 🟥 **E2E 测试访问内部私有结构**
- 🟥 **测试无法在并行环境中运行**

---

## 9. 测试维护生命周期

> **优化来源**：`docs/其他/69.md` — 测试维护生命周期（Test Deprecation Protocol 标记废弃测试）

### 9.1 测试生命周期阶段

```
新建 → 活跃 → 过时 → 废弃 → 删除
  │      │       │       │       │
  │      │       │       │       └─ 从代码库移除
  │      │       │       └─ 标记 #[deprecated]，保留 1 个版本
  │      │       └─ 标记 #[ignore]，在 CI 中跳过
  │      └─ CI 每次运行，定期审查覆盖有效性
  └─ 新增测试，必须包含在 CI 中
```

### 9.2 Test Deprecation Protocol（废弃协议）

当业务规则变更导致测试不再适用时，禁止直接删除测试，必须遵循以下流程：

```rust
// ❌ 禁止：直接删除过时测试
// #[test]
// fn old_damage_test() { ... }  // 删除后无法追溯

// ✅ 正确：标记废弃，保留原因
#[test]
#[deprecated(since = "v0.3.0", note = "伤害公式已重构为 Modifier 链，此测试不再适用")]
#[ignore] // CI 中跳过
fn old_damage_test_without_modifiers() {
    // 此测试验证的是旧版伤害公式（无 Modifier 链）
    // 新版伤害计算已迁移到 damage_with_modifiers_test.rs
    // 废弃原因：PR #142 重构了伤害计算管线
    let result = calculate_damage_v2(100, 30);
    assert_eq!(result, 70);
}
```

### 9.3 测试审查检查清单

每月审查一次测试健康度：

| 检查项 | 方法 | 处理方式 |
|--------|------|---------|
| 测试是否仍然覆盖有效规则 | 对照领域规则文档 | 无对应规则 → 标记废弃 |
| 测试是否在 CI 中运行 | 检查 `#[ignore]` 标记 | 被忽略 > 3 个月 → 标记废弃 |
| 测试是否与其他测试重复 | 代码审查 | 重复 → 合并或删除 |
| 测试是否硬编码了过时数值 | 对照 RON 配置 | 数值过时 → 更新 fixtures |
| 测试执行时间是否异常 | CI 报告分析 | > 10s → 优化或拆分 |

### 9.4 覆盖率指标

```yaml
# CI 中的覆盖率报告
coverage:
  domain_rules: 100%     # 每条领域规则必须有对应测试
  core_systems: ">=90%"  # 伤害/Buff/回合系统
  e2e_main_flow: 100%    # 进入战斗→操作→胜利/失败
  
  # 覆盖率下降告警
  fail_on_decrease: true  # 任何层覆盖率下降 → CI 失败
```

---

## 9. 落地优先级

| 优先级 | 任务 | 理由 |
|--------|------|------|
| 🔴 第一优先 | 搭建 `shared/testing` + 核心领域单元测试 | 后续所有扩展的基础 |
| 🔴 第一优先 | 搭建 `battle_simulator` 沙盒 | 开发调试效率倍增器 |
| 🟡 第二优先 | 补充内容数据校验 + 核心 E2E 测试 | 开始大量加内容前的闸门 |
| 🟡 第二优先 | CI 接入核心流程回归测试 | 自动化质量保障 |
| 🟢 第三优先 | 完善其余沙盒工具 + 全量回归用例 | 随项目扩张逐步补齐 |

---

## 10. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `architecture.md` | 本文档是"Testing"章节的详细补充 |
| `layer-contracts.md` | 测试层级与架构层级一一对应 |
| `testing_spec.md` | 本文档覆盖更完整的测试体系 |
| `infrastructure-design.md` | 回放测试利用 audit 模块 |
| `docs/02-domain/replay_rules.md` | replay_validator 验证回放确定性 |
