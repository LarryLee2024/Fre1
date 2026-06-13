# Tactical RPG

基于 Bevy 0.18.1 的回合制战棋游戏，采用 ECS 架构与数据驱动设计。

## 核心功能

- **回合制战斗**：基于回合阶段状态机（选择 → 行动 → 结算），支持移动、攻击、技能、道具等操作
- **数据驱动配置**：单位、技能、Buff、地形、特质、AI 行为、关卡均通过 RON 文件配置，无需修改代码即可扩展内容
- **关卡与胜负条件**：关卡配置包含可组合的胜利/失败条件（全灭、存活回合、击败 Boss、超时等），支持多条件 OR 组合
- **Effect Pipeline**：战斗效果通过生成 → 修饰 → 执行的管线处理，支持 Trait/Modifier 组合扩展
- **AI 系统**：数据驱动的 AI 行为配置，支持多种策略模板
- **调试面板**：基于 egui 的运行时调试工具，支持 World Inspector 和状态查看

## 安装指南

### 环境要求

- Rust 1.96+（edition 2024）
- Cargo

### 构建与运行

```bash
# 克隆项目
git clone <repository-url>
cd a1

# 编译运行
cargo run

# 开发模式（启用文件热重载和调试工具）
cargo run --features dev

# 运行测试
cargo test
```

## 项目结构

```
src/
  ai/          # AI 行为系统
  battle/      # 战斗效果管线（generate → modify → execute）
  buff/        # Buff/Debuff 系统
  character/   # 单位组件与 Trait 扩展体系
  core/        # 属性系统、效果管线、修饰规则、标签系统
  debug/       # 调试面板与查看器
  equipment/   # 装备系统
  inventory/   # 背包系统
  map/         # 地图、寻路、关卡配置加载
  skill/       # 技能系统
  turn/        # 回合状态机、行动顺序、胜负条件检查
  ui/          # 用户界面面板与组件
  input/       # 输入处理

assets/        # RON 配置文件
  units/       # 单位模板
  skills/      # 技能定义
  buffs/       # Buff 定义
  terrains/    # 地形类型
  traits/      # 角色特质
  ai/          # AI 行为模板
  maps/        # 关卡配置
  definitions/ # 属性与标签定义
  rules/       # 游戏规则（元素交互等）

docs/
  architecture.md    # 架构规范（最高优先级）
  domain/            # 领域规则文档
  adr/               # 架构决策记录
  testing/           # 测试计划
  reviews/           # 代码审查记录

tests/         # 集成测试、场景测试、快照测试
```

## 架构原则

项目遵循以下核心架构原则（详见 `docs/architecture.md`）：

1. **Definition / Instance 分离**：配置数据（如 UnitTemplate）不可变，运行时实例（如 Unit）可变
2. **Rule / Content 分离**：检查逻辑是规则，RON 配置是内容
3. **Logic / Presentation 分离**：业务逻辑在 System 中，UI 层只读取状态
4. **数据驱动**：游戏内容通过 RON 文件配置，禁止硬编码

## AI 辅助开发

项目配备 6 个专用 AI Agent（详见 `AGENTS.md`），遵循严格的协作流程：

```
需求 → @domain-designer → @architect → @feature-developer → @test-guardian → @code-reviewer
```

## 注意事项

- 资产路径使用编译时绝对路径（`CARGO_MANIFEST_DIR`），发布构建时需确保 assets 目录与可执行文件相对位置正确
- 关卡配置中 `victory_condition` 为 `Option` 类型，`None` 时回退到默认的全灭胜利条件
- 胜负条件检查仅在 TurnEnd 阶段执行，全灭玩家即失败为绝对不变量（不可被配置覆盖）
- 胜负同时满足时优先判定失败（失败优先原则）
