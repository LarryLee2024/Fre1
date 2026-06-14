---
id: 00-governance.coding-rules
title: Coding Rules
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - governance
  - coding
---

# Bevy SRPG Coding Constitution v1.0
**工业级AI专属 | 永久架构稳定版 | 与《测试宪法v3.1》配套执行**

Version: 1.0
Status: Active
Applies To:
* All Rust code
* All Bevy plugins
* All ECS systems
* All gameplay features
* All AI-generated code

## 优先级
architecture.md > domain_rules.md > coding_rules.md

---

# 1. Core Principles
## 1.1 实现设计，不重新设计
AI的唯一职责：**严格实现已有设计**。

禁止：
* 修改架构边界
* 发明新的领域模型
* 引入新的核心模式

发现设计问题时：
立即停止实现，输出：
```
ARCHITECTURE QUESTION:
当前实现与设计存在冲突，需要架构决策。
冲突点：XXX
建议方案：XXX
```

## 1.2 最简单方案优先
抽象优先级（从上到下，优先选择最上面的）：
```
纯函数
↓
Struct
↓
Component
↓
System
↓
Trait
↓
泛型
↓
Macro
```

**绝对禁止**：为了"代码优雅"或"未来扩展"增加不必要的抽象层。

## 1.3 Rule / Content 分离
新增以下内容必须优先修改配置，**绝对禁止**修改核心规则代码：
* 职业
* 技能
* Buff
* 装备
* AI行为
* 天赋
* 种族

## 1.4 Definition / Instance 分离
* Definition（配置）：加载后不可变，全局唯一
* Instance（运行时状态）：每个实体独立，不写回Definition

**绝对禁止**：修改任何Definition对象。

---

# 2. Feature First
## 必须
按业务领域组织代码结构：
```
battle/
character/
inventory/
skill/
quest/
map/
ui/
```

## 禁止
以下目录绝对不能作为顶层业务结构：
```
components/
systems/
events/
managers/
utils/
helpers/
common/
```

## Feature内部结构
一个Feature至少包含：
```
mod.rs
plugin.rs
```

必要时可增加：
```
components.rs
messages.rs
systems.rs
```
以上均为Feature内部实现，不得对外暴露。

---

# 3. ECS Rules
## Entity 只是 ID
Entity仅用于引用实体。

禁止：
* 任何形式的EntityManager
* 面向对象实体模型
* 在Entity上调用任何方法

## Component 只存数据
Component只能存储纯数据状态。

禁止：
* 在Component impl块中实现复杂业务逻辑
* Component包含函数指针或闭包

## System 只存行为
所有业务逻辑必须放在System中。

## Tag 绝对优于 bool
所有布尔状态必须使用空Tag Component实现。

正确：`Dead`、`Stunned`、`Selected`
错误：`is_dead: bool`、`is_selected: bool`

## Resource 不是垃圾桶
Resource只能存储真正的全局唯一状态。

禁止：
```rust
struct GameManager;
struct BattleManager;
struct EverythingManager;
```

---

# 4. Bevy Native First
优先使用Bevy原生能力表达所有概念：
* 依赖关系：Required Components
* 组件生命周期：Component Hooks
* 局部响应：Observers
* 跨模块通信：Messages
* 流程控制：States
* 资源管理：Assets

**绝对禁止**：重复实现Bevy引擎已提供的基础设施。

---

# 5. Message / Observer / Hook
## Message
用于：跨Feature广播通知。

示例：`TurnEnded`、`QuestCompleted`、`BattleFinished`

## Observer
用于：实体生命周期事件。

示例：Dead组件添加、Buff组件添加、装备穿戴

## Hook
用于：组件的固有行为。

示例：Dead自动移除MoveTarget组件

## Observer Boundary
**绝对禁止**：在以下场景使用Observer：
* 高频计算（每帧执行10次以上）
* 位置同步
* 寻路更新
* 属性刷新
* 行动力变化

以上场景必须直接使用System处理。

---

# 6. Effect Pipeline Protection
**领域不变量，优先级高于所有其他编码规则**

所有战斗效果必须通过统一Effect Pipeline执行：
```
CombatIntent
↓
Generate
↓
Modify
↓
Execute
```

**绝对禁止**：
* 直接修改单位生命值：`entity.hp -= damage`
* 直接添加/移除Buff：`entity.insert(Buff)`
* 直接执行死亡判定
* 直接应用任何战斗效果

---

# 7. Modifier Pipeline Protection
**领域不变量，优先级高于所有其他编码规则**

所有属性修改必须通过统一Modifier Pipeline执行：
```
Modifier
↓
Attribute Resolver
↓
Final Stat
```

**绝对禁止**：
* 直接修改最终属性值：`attack += 10`
* 绕过Attribute Resolver计算属性
* 在任何地方硬编码属性计算公式

---

# 8. Trait Rules
## Trait 只表示扩展点
允许：
```rust
trait DamageFormula;
trait TargetSelector;
trait EffectExecutor;
```

## Trait 不表示分类
**绝对禁止**：使用Trait模拟继承树。

禁止：
```rust
trait CharacterTrait;
trait EnemyTrait;
trait BossTrait;
```

## 三次原则
代码真实重复出现三次以上，才允许抽象为Trait。

## 单实现原则
如果Trait只有一个实现，**必须**改为普通函数。

---

# 9. Function Rules
## 单一职责
一个函数只能有一个主要目的。

## 长度限制
* 理想：20~50行
* 警觉：超过100行必须重构

## 嵌套限制
超过3层嵌套必须重构。

## Early Return 优先
正确：
```rust
if !is_valid(target) {
    return Err(InvalidTarget);
}
```

## 命名规范
函数名描述意图，不描述过程。

正确：`apply_damage()`
错误：`calculate_and_apply_damage_and_notify_everything()`

---

# 10. File Rules
## 一个文件一个主题
正确：`equipment.rs`
错误：`equipment_buff_skill_upgrade.rs`

## 文件大小限制
* 理想：300~500行
* 警觉：超过1000行必须拆分

## 禁止垃圾桶文件
**绝对禁止**：创建无限增长的`utils.rs`、`helpers.rs`、`common.rs`文件。

通用工具代码必须按功能拆分到独立文件。

---

# 11. Data Driven Rules
## 所有内容必须配置化
职业、技能、Buff、装备、AI行为、天赋、种族必须100%配置化。

## 配置不可变
所有Definition对象加载后必须保持不可变。

## 运行时独立
运行时状态绝对不能写回Definition。

---

# 12. Reflect Boundary
Reflect **仅属于工具链层**，不属于游戏逻辑层。

允许用于：
* 编辑器
* Inspector
* 配置工具
* 调试面板

**绝对禁止**：在以下核心运行时逻辑中使用Reflect：
* 战斗计算
* AI决策
* 寻路算法
* 属性计算
* Buff结算
* 回合管理

---

# 13. Error Handling
## Result 优先
所有可失败的逻辑必须返回Result。

## 禁止 unwrap/expect
**绝对禁止**：在业务代码中使用`unwrap()`或`expect()`。

仅允许在以下场景使用：
* 测试代码
* 工具代码
* 绝对不可能失败的情况（必须加注释说明）

## 错误必须包含上下文
正确：`format!("skill_id={} not found", skill_id)`
错误：`"failed"`

---

# 14. Logging
* 统一使用`tracing`库
* **绝对禁止**：使用`println!`或`dbg!`
* 所有日志必须是结构化的

正确：
```rust
info!(unit=?entity, damage=amount, "damage applied");
```

---

# 15. Testing
* 新增业务逻辑必须同时新增测试
* 修复Bug必须先写失败测试，再修复代码
* 测试验证行为，不验证实现细节

---

# 16. AI Modification Rules
## 允许
* 新增Feature
* 新增配置
* 新增测试
* 修复Bug
* 重构重复逻辑

## 禁止（未经授权）
* 修改架构边界
* 删除领域规则
* 绕过Effect Pipeline
* 绕过Modifier Pipeline
* 绕过Message体系
* 绕过测试

## 修改前必须检查
1. 是否违反architecture.md
2. 是否违反domain_rules.md
3. 是否破坏现有测试
4. 是否增加架构复杂度
5. 是否存在更简单的方案

## 修改后必须输出
```
### Changes
- 修改内容1
- 修改内容2

### Architecture Check
✅ 符合架构要求 / ❌ 违反条款：XXX

### Domain Check
✅ 符合领域规则 / ❌ 违反条款：XXX

### Pipeline Check
✅ 未绕过统一管道 / ❌ 绕过管道：XXX

### Tests Check
✅ 新增对应测试 / ❌ 未新增测试

### Complexity Check
复杂度变化：无/低/中/高

### Risk Check
潜在风险：XXX
```

---

# 17. CODE_EXEMPT
任何违反本规则的代码必须标注：
```rust
// CODE_EXEMPT
//
// Rule: [章节号.条款号]
//
// Reason: [详细技术理由]
//
// Expire: [YYYY-MM-DD]
```

**绝对禁止**：永久豁免。

所有豁免代码必须在每3个月的架构复盘时重新评估。

---

# 18. Refactoring Rules
重构优先级：
```
删除无用代码
>
合并重复代码
>
重命名
>
抽象
```

**绝对禁止**：为了"更优雅"增加层级。

重构后必须保证：
* 外部行为完全一致
* 所有测试通过
* 领域规则不变

---

# 19. AI Self Check
AI生成任何代码后，必须自动完成以下检查并在文件开头标注结果：
```rust
// ================================================
// AI Self Check
// ================================================
// Feature First: PASS/FAIL
// Definition/Instance: PASS/FAIL
// Rule/Content: PASS/FAIL
// Effect Pipeline: PASS/FAIL
// Modifier Pipeline: PASS/FAIL
// Architecture Violation: NONE/XXX
// Complexity Increase: NONE/LOW/MEDIUM/HIGH
// Tests Added: YES/NO
// ================================================
// CODE_EXEMPT: NONE/[规则编号]
// ================================================
```

---

## 版本说明
本版本是专门为**Bevy 0.18+ SRPG项目 + AI辅助开发**设计的工业级编码宪法。核心设计原则：
1. **架构永久稳定**：所有规则不依赖具体Bevy API版本，未来版本升级无需修改
2. **领域优先**：将SRPG最核心的两个管道（Effect/Modifier）提升为最高优先级的领域不变量
3. **AI友好**：所有规则都是可执行的命令，没有模糊表述
4. **边界清晰**：明确划分了AI的权限范围，防止AI自作聪明破坏核心架构
5. **可验证**：所有规则都可以通过工具自动检查

本规范已覆盖99% AI可能犯的错误，同时保持了足够的灵活性。新增规则的边际收益已低于维护成本。

需要我把这个v1.0版本转换成**纯提示词版**（去掉所有格式和示例，只保留AI需要执行的核心规则，压缩到700字以内）吗？
