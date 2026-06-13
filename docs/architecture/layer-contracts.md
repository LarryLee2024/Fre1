# Layer Contracts — 七层架构边界定义

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目的七层架构边界、依赖规则、禁止事项和判断标准。

---

## 七层总览

```
┌─────────────────────────────────────────────────────┐
│  Layer 1: App          游戏启动与装配                  │
├─────────────────────────────────────────────────────┤
│  Layer 2: Core          游戏规则（纯领域逻辑）            │
├─────────────────────────────────────────────────────┤
│  Layer 3: Shared        基础能力（通用工具）              │
├─────────────────────────────────────────────────────┤
│  Layer 4: Infrastructure 技术实现                       │
├─────────────────────────────────────────────────────┤
│  Layer 5: Content       内容桥接（配置 → 规则绑定）       │
├─────────────────────────────────────────────────────┤
│  Layer 6: Modding        MOD 支持                       │
├─────────────────────────────────────────────────────┤
│  Layer 7: Tools          开发工具                       │
└─────────────────────────────────────────────────────┘

表现层（跨层）：
  UI — 独立于所有层，只读 ViewModel
  Debug — 独立于所有层，只读业务数据
```

---

## 第一层：App — 游戏启动与装配

### 职责

App 层的唯一职责是**组装整个游戏**。

- 注册所有 Plugin
- 定义 AppState / TurnPhase 状态机
- 定义 Schedules 和 SystemSets
- 启动时加载必要资源
- 关闭时清理资源

### 判断标准

> 这是把游戏"启动起来"的代码吗？

如果是 → App 层。

### 目录

```
src/app/
├── app_plugin.rs       # 主 Plugin，注册所有子 Plugin
├── game_state.rs       # AppState 定义
├── schedules.rs        # Schedule 定义
├── sets.rs             # SystemSet 定义
├── startup.rs          # 启动逻辑
├── shutdown.rs          # 关闭逻辑
└── plugins.rs          # Plugin 汇集注册
```

### 依赖规则

🟩 App 层**允许依赖所有层**。

App 是唯一允许全局视野的层。它知道所有模块的存在，但只是组装它们。

### 禁止事项

- 🟥 App 层禁止包含任何业务逻辑
- 🟥 App 层禁止直接创建 Entity（应在各业务模块的 startup 中创建）
- 🟥 禁止在 App 层硬编码游戏数值

详细设计见 `docs/architecture/app-bootstrap.md` 和 `docs/architecture/plugin-design.md`。

---

## 第二层：Core — 游戏规则

### 职责

Core 层是项目的**心脏**——纯游戏规则。

所有游戏规则的逻辑都在 Core 中：
- 战斗伤害计算
- Buff 施加与结算
- 属性修饰计算
- 回合管理
- AI 决策规则
- 技能效果定义
- 装备穿脱规则
- 背包操作逻辑
- 寻路算法
- 胜负判定

### 判断标准（三问法）

> **核心问题**：如果明天把 Bevy 删了，换成 Godot / Unity / UE / 服务器模拟器，这个逻辑还存在吗？

如果**存在** → Core 层。

### 必须在 Core 的

```
战斗规则          回合管理          伤害计算
属性系统          Buff 规则         技能规则
装备规则          背包逻辑          寻路算法
AI 决策规则       角色规则          胜负判定
任务规则          对话树逻辑         地形规则
职业成长          经济规则           掉落规则
```

### 绝对不在 Core 的

```
Bevy 组件声明      资源加载           日志框架
存档序列化         网络通信           UI 渲染
音频播放           输入处理           文件读取
热重载             调试面板           Shader 编译
```

### 依赖规则

```
Core → Shared    ✅ 允许
Core → Content   ❌ 禁止
Core → Infra     ❌ 禁止
Core → UI        ❌ 禁止
Core → App       ❌ 禁止
Core → Modding   ❌ 禁止
Core → Tools     ❌ 禁止
```

🟥 Core 层**只能依赖 Shared 层**。

**发现 core 模块 use 了 infrastructure、ui、modding 等层时**：

必须停止。必须输出：

```
ARCHITECTURE VIOLATION: core 模块依赖外部层 [模块名]，违反"核心层无外部依赖"原则。
```

### Core 内部模块边界

Core 内部模块之间通过 Message 通信，禁止直接访问内部组件。

```
battle → skill       只能通过 CombatIntent / DamageApplied Message
skill → buff         只能通过 SkillCastStarted / SkillCastFinished Message
equipment → buff     只能通过 EquipItem / ItemEquipped Message
character → battle  只能通过 CharacterDied / UnitMoved Message
```

### 关键约束

- 🟥 Core 模块禁止 use 其他业务模块的内部实现
- 🟥 Core 不得包含任何 Bevy 特定依赖（如 `Res<AssetServer>`）
- 🟩 Core 通过 Shared 层的 trait 和事件进行模块间通信
- 🟩 Core 错误类型放在各业务模块内部（`core/skill/domain/skill_error.rs`）

---

## 第三层：Shared — 基础能力

### 职责

Shared 层提供**所有模块都能复用的基础工具**。

它既不是游戏规则，也不是技术实现，而是"如果没有任何游戏逻辑，这个模块依然有意义"的东西。

### 判断标准

> **核心问题**：这个东西既不是游戏规则，也不是技术实现，而是所有地方都会用到的基础工具吗？

如果是 → Shared 层。

### 必须在 Shared 的

```
强类型 ID（UnitId, SkillId, BuffId, ...）
共享事件定义
审计轨迹基础设施
确定性随机数
数学工具（距离计算等）
时间工具
通用集合类型
校验工具
全局常量
核心 trait（DamageSource, Healable 等）
测试工具（spawn_test_battle 等）
版本管理工具
GameResult<T> 类型别名
错误转换 trait
```

### 绝对不在 Shared 的

```
skill_utils        ❌ 这是 Skill 领域逻辑，回 skill/
buff_utils         ❌ 这是 Buff 领域逻辑，回 buff/
battle_utils       ❌ 这是 Battle 领域逻辑，回 battle/
quest_utils        ❌ 这是 Quest 领域逻辑，回 quest/
game_manager       ❌ 这是业务逻辑，不是工具
anyhow::Error      ❌ 全局错误类型是反模式
```

### 目录

```
src/shared/
├── ids/             # 强类型 ID
├── error/           # GameResult<T> + 错误转换 trait
├── events/          # 跨模块领域事件
├── audit/           # 审计轨迹基础设施
├── random/          # 确定性随机数
├── math/            # 数学工具
├── time/            # 时间工具
├── collections/     # 通用集合
├── validation/      # 校验工具
├── constants/       # 全局常量
├── traits/          # 核心 trait
├── macros/          # 过程宏
├── testing/         # 测试工具
└── versioning/      # 版本管理
```

### 依赖规则

```
Shared → 无依赖     ✅ Shared 不依赖任何其他层
其他层 → Shared     ✅ 所有层都可以依赖 Shared
```

🟥 **Shared 层禁止依赖任何其他层**。

Shared 是依赖图的**叶子节点**——它只提供工具，不消费任何业务逻辑。

发现 `shared/` 模块 use 了 `core/`、`infrastructure/`、`ui/` 等时：

必须停止。必须输出：

```
ARCHITECTURE VIOLATION: shared 模块依赖业务层 [模块名]，违反"基础能力层无外部依赖"原则。
```

### 严格管控规则

🟥 **Shared 层必须严格限制新增模块**。

每次想在 Shared 中添加新模块时，必须通过以下检查：

1. 这个东西对所有模块都有用吗？（不是 → 回到业务模块）
2. 这个东西不包含任何业务逻辑吗？（包含 → 回到业务模块）
3. 这个东西不依赖任何业务类型吗？（依赖 → 回到业务模块）

只有三个答案都是"是"才能加入 Shared。

---

## 第四层：Infrastructure — 技术实现

### 职责

Infrastructure 层提供**技术实现**——游戏规则不变，但实现方式可以替换的东西。

### 判断标准

> **核心问题**：如果游戏规则不变，能不能换一种实现方式？

如果能 → Infrastructure 层。

### 必须在 Infrastructure 的

```
资源加载（AssetServer 封装）
存档保存/加载/迁移
日志框架
多语言/本地化
战斗回放
数据分析/遥测
配置管理
数据导入（JSON/CSV/YAML/Excel）
数据导出
网络通信
Steam 集成
云存档
热重载
诊断工具
性能分析
崩溃报告
脚本运行时（MOD 支持）
```

### 绝对不在 Infrastructure 的

```
SkillError       ❌ 这是领域错误，回 core/skill/domain/
BattleError      ❌ 这是领域错误，回 core/battle/
BuffError        ❌ 这是领域错误，回 core/buff/
伤害计算         ❌ 这是游戏规则，回 core/battle/
属性公式         ❌ 这是游戏规则，回 core/attribute/
AI 决策规则      ❌ 这是游戏规则，回 core/ai/
```

### 依赖规则

```
Infra → Core      ✅ 允许（但只通过 shared 事件，不直接依赖内部）
Infra → Shared    ✅ 允许
Infra → Content   ✅ 允许（加载数据需要）
Infra → UI        ❌ 禁止
Infra → App       ❌ 禁止
Infra → Modding   ❌ 禁止
Infra → Tools     ❌ 禁止
```

### Infrastructure 层错误归属

```
infrastructure/ persistence/  → save_error.rs, load_error.rs, migration_error.rs
infrastructure/ assets/       → asset_error.rs
infrastructure/ networking/   → network_error.rs
```

Infrastructure 错误在 Infrastructure 内部定义，不放入 Shared。

详细设计见 `docs/architecture/infrastructure-design.md`。

---

## 第五层：Content — 内容桥接

### 职责

Content 层是**连接外部数据和内部规则的桥梁**。

它负责：
- 将 RON 配置文件加载为内部的 `XxxData` / `XxxDef` 类型
- 校验配置数据的完整性和一致性
- 注册到对应的 `XxxRegistry`

**核心区分**：Skill（规则）在 Core，Fireball（内容）在 Content。

### 判断标准

> 这是"用规则解释一段数据"的代码吗？

如果是 → Content 层。

### 目录

```
src/content/
├── content_plugin.rs    # 内容加载总 Plugin
├── skills/              # 加载 RON → SkillData → SkillRegistry
├── buffs/               # 加载 RON → BuffData → BuffRegistry
├── classes/             # 加载 RON → ClassData
├── characters/          # 加载 RON → UnitTemplate
├── ...
```

### 依赖规则

```
Content → Core      ✅ 允许（Content 调用 Core 的 Registry 注册）
Content → Shared    ✅ 允许
Content → Infra     ✅ 允许（使用 AssetServer 加载）
Content → UI        ❌ 禁止
Content → App       ❌ 禁止
Content → Modding   ❌ 禁止
```

### 关键约束

- 🟥 Content 层禁止包含任何游戏规则逻辑
- 🟩 Content 层只做"加载 → 校验 → 注册"三件事
- 🟩 Content 层的测试可以用 Mock Registry 替代真实 Registry

### 双类型模式（Core + Content 协作）

```
core/skill/skill_def.rs       # SkillDef: RON 反序列化用，TagName 字符串
core/skill/skill_data.rs      # SkillData: 运行时用，GameplayTag 位掩码
content/skills/skill_content.rs # RON → SkillDef → SkillData → Registry
```

Content 加载 RON 文件为 `SkillDef`，然后 `impl From<SkillDef> for SkillData` 转换为运行时类型，注册到 `SkillRegistry`。

---

## 第六层：Modding — MOD 支持

### 职责

Modding 层提供**安全的 MOD 扩展能力**。

它只暴露稳定的 API，让 MOD 作者可以：
- 添加新技能、新 Buff、新装备
- 修改数值平衡
- 添加新地图
- 添加新剧情对话

同时确保 MOD 不能：
- 绕过 Effect Pipeline
- 直接修改核心游戏规则
- 破坏其他 MOD 的数据

### 依赖规则

```
Modding → Core      ✅ 允许（通过稳定 API 接口）
Modding → Shared    ✅ 允许
Modding → Content   ✅ 允许（MOD 也是一种内容加载）
Modding → Infra     ✅ 允许（MOD 加载需要基础设施）
Modding → UI        ❌ 禁止
Modding → App       ❌ 禁止
```

详细设计见 `docs/architecture/modding-design.md`。

---

## 第七层：Tools — 开发工具

### 职责

Tools 层提供**开发期间的工具链**。

- 内容编辑器
- 地图编辑器
- 数据校验器
- 数值平衡器
- 回放检查器
- 性能基准

### 关键约束

- 🟥 Tools 层永不进入发布构建
- 🟥 Tools 层禁止包含业务逻辑
- 🟩 Tools 层可以直接操作 Registry 和 World（开发期间权限更宽）

---

## 表现层：UI 和 Debug

### UI 层

UI 是一个**跨层模块**，但它有严格的边界规则。

#### 依赖规则

```
UI → ViewModel    ✅ 只读 ViewModel
UI → UiCommand    ✅ 只输出 UiCommand
UI → Core         ❌ 绝对禁止直接查询 ECS 组件
UI → Infra        ❌ 绝对禁止
```

详见 `docs/architecture.md` 中"UI 架构"部分。

### Debug 层

Debug 也是一个**跨层模块**，但只读。

#### 依赖规则

```
Debug → Core      ✅ 只读业务数据
Debug → Infra      ✅ 只读日志/遥测
Debug → UI         ❌ 禁止
```

---

## 依赖方向总图

```
                    ┌──────┐
                    │ App  │  ← 唯一的全局组装者
                    └──┬───┘
                       │（注册所有 Plugin）
            ┌──────────┼──────────┐
            │          │          │
         ┌──▼──┐  ┌────▼──┐  ┌───▼────┐
         │ UI  │  │ Core  │  │Modding │
         └──┬──┘  └───┬───┘  └───┬────┘
            │         │          │
            │    ┌────▼────┐     │
            │    │ Shared  │◄────┘
            │    └────┬────┘
            │         │
         ┌──▼──┐  ┌───▼────┐
         │Debug│  │ Infra  │
         └─────┘  └───┬────┘
                      │
                ┌─────▼──────┐
                │  Content   │
                └────────────┘

        ┌────────────────────────┐
        │        Tools           │  ← 开发期间，永不发布
        └────────────────────────┘
```

### 简化依赖规则

```
App      → 任意层           ✅（仅注册，不含逻辑）
Core     → Shared           ✅（唯一允许的外部依赖）
Shared   → 无               ✅（叶子节点，零外部依赖）
Infra    → Core, Shared     ✅（技术实现依赖规则）
Content  → Core, Infra, Shared  ✅（桥接层，连接数据与规则）
UI       → ViewModel only   ✅（只读状态层）
Debug    → Core only（只读） ✅
Modding  → Core, Shared, Infra, Content  ✅（扩展层）
Tools    → Core, Shared      ✅（开发工具）
```

### 严格禁止的依赖方向

```
Core → Infra         🟥 禁止（规则不依赖技术实现）
Core → Content       🟥 禁止（规则不依赖数据加载）
Core → UI            🟥 禁止（规则不依赖表现层）
Core → Modding       🟥 禁止（规则不依赖 MOD）
Shared → Core        🟥 禁止（基础不依赖规则）
Shared → Infra       🟥 禁止（基础不依赖技术实现）
Shared → UI          🟥 禁止（基础不依赖表现层）
Infra → UI           🟥 禁止（技术不依赖表现层）
```

---

## 三个垃圾桶警告

在大型项目中，以下三个目录最容易退化为垃圾桶：

### 1. shared/ 垃圾桶

**症状**：所有不知道放哪里的东西都往 shared 塞。

**防治**：每次往 shared 添加模块时，回答三个问题：
1. 如果删掉所有游戏逻辑，这个模块还有用吗？
2. 这个模块包含任何业务逻辑吗？
3. 这个模块依赖任何业务类型吗？

如果 1=否 或 2=是 或 3=是 → **不放 Shared**。

### 2. common/ 垃圾桶

**症状**：和 shared 一样，只是换了个名字。

**防治**：不创建 common/ 目录。如果已经存在，合并到 shared/ 或拆分到业务模块。

### 3. utils/ 垃圾桶

**症状**：把小函数全部塞进 utils.rs。

**防治**：
- 🟥 禁止创建 `utils.rs`、`helpers.rs` 垃圾桶文件
- 🟩 通用工具按功能拆分到 shared/ 对应模块
- 🟩 领域工具放在对应业务模块内部

---

## 错误归属规范（补充）

参照 `docs/architecture/error-architecture.md` 完整规范。此处简述：

### 领域错误 → core/xxx/domain/

```rust
// core/skill/domain/skill_error.rs
pub enum SkillError {
    SkillNotFound { skill_id: SkillId },
    InvalidTarget { caster: UnitId, target: UnitId },
    InsufficientResource { skill_id: SkillId, cost: i32 },
    CooldownNotExpired { skill_id: SkillId, remaining: u32 },
    RequirementNotMet { skill_id: SkillId, reason: String },
}
```

### 基础设施错误 → infrastructure/xxx/

```rust
// infrastructure/persistence/save/save_error.rs
pub enum SaveError {
    FileNotFound { path: String },
    SerializeFailed { reason: String },
    DiskFull,
}
```

### 共享错误工具 → shared/error/

```rust
// shared/error/result.rs
pub type GameResult<T> = Result<T, InfrastructureError>;

// shared/error/extensions.rs
pub trait ErrorExt {
    fn with_context(self, context: &str) -> Self;
}
```

🟥 **绝对禁止**：
- 全局统一 `AppError` 大枚举
- `anyhow::Error`、`Box<dyn Error>` 作为业务层返回类型
- 领域错误放在 `infrastructure/` 或 `shared/`
- 基础设施错误包含领域语义

---

## 跨层通信规范

### Core ↔ Core

通过 **Message** 广播。

```
core/battle → core/skill     只能通过 DamageApplied / HealApplied Message
core/skill  → core/buff      只能通过 SkillCastStarted / SkillCastFinished Message
core/equipment → core/buff   只能通过 EquipItem / ItemEquipped Message
```

### Core → UI

通过 **ViewModel** 间接通信。

```
Core（业务逻辑）→ 修改 ECS Component
                  → ViewModel（只读查询）→ UI Panel/Widget
```

### UI → Core

通过 **UiCommand** Message。

```
UI Panel → UiCommand → CommandHandler → Core System
```

### Core → Infrastructure

通过 **Shared 事件**间接通信。

```
Core 发布领域事件 → Infrastructure 的 Observer 监听 → 执行技术操作
```

### Content → Core

通过 **Registry 注册**。

```
Content 加载 RON → XxxDef → XxxData → Registry.insert()
```

### Modding → Core

通过 **稳定 API**。

```
MOD 添加新技能 → mod/api → SkillRegistry.insert_mod_skill()
```

---

## 迁移检查清单

每次添加新模块或新文件时，必须回答：

- [ ] 这个模块/文件属于哪一层？
- [ ] 它是否依赖了不允许的层？（对照依赖规则图）
- [ ] 它是否包含不应该在当前层的逻辑？（对照判断标准）
- [ ] 它的错误类型是否放在了正确的位置？
- [ ] 它的 ID 类型是否放在了 shared/ids/？
- [ ] 它是否创建了 xxx_utils 垃圾桶？

如果任何一项为"是"，必须重构。