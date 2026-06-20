# 21个Rust特性吸收 — 项目文档全面更新计划

> 来源：`docs/ai_ignore_this_dir/15rust特性.md`
> 日期：2026-06-21
> 状态：待执行

---

## 一、覆盖度总览

| 覆盖状态 | 数量 | 特性编号 |
|---------|------|---------|
| **已全面使用** | 3 | #1 Trait+Associated Type, #3 Newtype, #7 Plugin Trait封装 |
| **部分使用** | 11 | #2 Blanket Impl, #4 Marker Trait, #5 Trait Alias, #6 Extension Trait, #8 SystemSet泛型注册, #9 Macro+Trait, #11 PhantomData, #12 Const Trait, #14 Object Safety, #16 Interior Mutability, #17 Iterator Pipeline, #18 Trait Object Registry, #19 ZST |
| **未使用** | 7 | #10 Typestate, #13 Sealed Trait, #15 Cow<'a,T>, #20 Compile-time Capability, #21 HRTB |

### 代码实现 vs 文档指导对比

| 特性 | 代码实现 | 文档指导 | 差距 |
|------|---------|---------|------|
| #1 Trait+Associated Type | 全面 | 部分 | 业务层自定义trait缺关联类型模式指导 |
| #2 Blanket Impl | 部分 | 无 | 缺使用边界规范 |
| #3 Newtype | 全面 | 有 | 非ID场景缺规范 |
| #4 Marker Trait | 部分 | 无 | 宪法禁止"Trait分类"但未区分Marker Trait |
| #5 Trait Alias | 部分 | 无 | 缺supertrait组合模式指导 |
| #6 Extension Trait | 部分 | 无 | 完全缺失 |
| #7 Plugin Trait | 全面 | 部分 | 缺自定义DomainPlugin trait设计 |
| #8 SystemSet泛型注册 | 部分 | 无 | 完全缺失 |
| #9 Macro+Trait | 部分 | 部分 | 缺derive宏+Trait组合模式指导 |
| #10 Typestate | 未使用 | 无 | 完全缺失 |
| #11 PhantomData | 部分 | 部分 | 缺使用规范 |
| #12 Const Trait | 部分 | 无 | 完全缺失 |
| #13 Sealed Trait | 未使用 | 无 | 完全缺失 |
| #14 Object Safety | 部分 | 无 | 完全缺失 |
| #15 Cow<'a,T> | 未使用 | 无 | 完全缺失 |
| #16 Interior Mutability | 部分 | 无 | 实践合规但未文档化 |
| #17 Iterator Pipeline | 部分 | 无 | 缺QueryExt DSL指导 |
| #18 Trait Object Registry | 部分 | 有 | ADR-013覆盖最完善 |
| #19 ZST | 部分 | 部分 | Tag Component即ZST但概念未显式化 |
| #20 Compile-time Capability | 未使用 | 无 | 完全缺失 |
| #21 HRTB | 未使用 | 无 | 完全缺失 |

---

## 二、专项计划 A：宪法文件更新

### 目标文件
- `docs/00-governance/ai-constitution-complete.md`

### A1. 需要新增的内容

| 编号 | 特性 | 新增位置 | 新增内容 |
|------|------|---------|---------|
| A1-1 | #13 Sealed Trait | §16.3 Trait宪法补充 | 新增"关键Trait密封性"条款：框架级trait（StrongId、RuleFailure、PipelineHook、ObservableEvent）必须使用Sealed Trait模式防止外部实现破坏不变量；只有设计为扩展点的trait才允许外部实现 |
| A1-2 | #16 Interior Mutability | §3 ECS宪法补充 | 新增"Interior Mutability边界"条款：只有Resource层和Infra层允许使用RefCell/Cell/Mutex；Domain层和Capability层禁止内部可变性；与ECS World的交互必须通过Commands/Events |
| A1-3 | #14 Object Safety | §8 能力宪法补充 | 新增"Object Safety分层"条款：热路径（战斗执行、属性计算）必须使用泛型静态分发；冷路径（编辑器、Mod系统、工具）允许使用dyn动态分发；设计trait时必须考虑object safety，显式标注是否允许dyn |
| A1-4 | #4 Marker Trait | §16.3 Trait宪法修正 | 修正"禁止用Trait表示分类"条款：区分"分类Trait"（禁止，如UnitTypeTrait）和"Marker Trait"（允许，如DomainEvent/ReplayEvent/AuditEvent）；Marker Trait用于驱动自动注册系统，不携带行为 |
| A1-5 | #20 Compile-time Capability | §8.1 角色系统补充 | 增加"编译期能力约束"条款：关键能力接口（如技能执行、物品使用）应通过trait bound在编译期约束，而非仅运行时检查；`fn execute<T: CanCast>(...)` 优于 `if unit.can_cast()` |
| A1-6 | #15 Cow | §10 Content宪法补充 | 增加"零拷贝文本"条款：静态文本（LocalizationKey常量、Def中的name_key/desc_key）必须使用 `Cow<'static, str>` 而非 `String`，避免静态内容的堆分配 |

### A2. 需要修改的内容

| 编号 | 特性 | 修改位置 | 修改方案 |
|------|------|---------|---------|
| A2-1 | #1 Associated Type | §16.3 Trait宪法补充 | 增加"关联类型优先"原则：当trait的实现类型决定返回/错误/上下文类型时，必须使用关联类型（`type Error; type Output;`）而非泛型参数，避免类型爆炸 |
| A2-2 | #2 Blanket Impl | §16.3 Trait宪法补充 | 增加"Blanket Impl自动派生"条款：当一个能力可由另一个能力自动推导时，必须使用blanket impl（`impl<T: Observable> Replayable for T {}`），禁止手动为每个类型重复实现 |
| A2-3 | #9 Macro+Trait | §16.5 宏使用规范补充 | 扩展derive宏指导：当同一trait需要为10+类型手动实现且逻辑高度相似时，必须创建derive宏自动生成；优先级：`#[derive(DomainEvent)]` > `#[derive(RuleFailure)]` > `#[derive(ObservableEvent)]` |
| A2-4 | #19 ZST | §6.1 Tag Component补充 | 显式化ZST概念：Tag Component就是零大小类型（ZST），编译期零开销；ZST不仅用于实体标记，还用于泛型分类标记（如 `struct DamageTag;` 用于 `Effect<DamageTag>`） |

### A3. 需要删除的内容

| 编号 | 特性 | 删除位置 | 删除方案 |
|------|------|---------|---------|
| A3-1 | #4 Marker Trait | §16.3 "禁止用Trait表示分类" | 修正为"禁止用Trait模拟类型层级（如UnitTypeTrait），但允许Marker Trait用于驱动自动注册系统" |

---

## 三、专项计划 B：规则文件更新

### B1. 代码风格（`.trae/rules/代码风格.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B1-1 | #1 Associated Type | 新增 | "关联类型 vs 泛型参数"选择指南：当返回/错误/上下文类型由实现决定时用关联类型；当需要同一类型多种实现时用泛型参数 |
| B1-2 | #5 Trait Alias | 新增 | "Trait Alias模拟"模式：通过supertrait链模拟trait alias（`pub trait DomainEvent: Event + Debug + Clone + Send + Sync {}`），统一领域事件约束 |
| B1-3 | #6 Extension Trait | 新增 | "Extension Trait"规范：为Bevy类型（EntityCommands/Query）添加扩展方法时使用Extension Trait；命名约定 `*Ext`；禁止为核心库类型添加与项目无关的扩展 |
| B1-4 | #15 Cow | 新增 | "字符串所有权策略"：静态文本用 `Cow<'static, str>`；动态文本用 `String`；跨层数据传递优先 `Cow`；禁止在热路径clone String |
| B1-5 | #17 Iterator Pipeline | 新增 | "迭代器管道 vs for循环"取舍：简单遍历用for；过滤/映射/聚合用迭代器管道；自定义QueryExt DSL用于高频ECS查询模式 |
| B1-6 | #11 PhantomData | 新增 | "PhantomData使用规范"：当需要编译期类型区分但运行时不需要数据时使用PhantomData；与Newtype组合实现类型安全ID（`InstanceId<T>`）；Reflect场景下用 `#[reflect(ignore)]` 标注 |

### B2. ECS规则（`.trae/rules/ECS规则.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B2-1 | #19 ZST | 新增 | "ZST Tag Component规范"：显式化ZST概念，Tag Component = ZST；ZST不仅用于实体标记，还用于泛型分类标记；`struct DamageTag;` 用于 `Effect<DamageTag>` 编译期分类 |
| B2-2 | #8 SystemSet泛型注册 | 新增 | "SystemSet分组规范"：每个Domain必须定义自己的SystemSet；通过 `register_domain_types!` 宏统一注册Reflect类型；未来考虑 `DomainSystems` trait 统一系统注册 |
| B2-3 | #6 Extension Trait | 新增 | "EntityCommands Extension"规范：为EntityCommands添加领域操作扩展方法（如 `.add_buff()`, `.heal()`, `.kill()`），替代散落的free函数 |

### B3. 架构规则（`.trae/rules/架构规则.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B3-1 | #13 Sealed Trait | 新增 | "Sealed Trait使用规范"：框架级trait必须sealed；扩展点trait允许外部实现；sealed实现模式（private module + Sealed supertrait） |
| B3-2 | #14 Object Safety | 新增 | "Object Safety分层策略"：热路径泛型/冷路径dyn；设计trait时必须考虑object safety；显式标注trait是否允许dyn |
| B3-3 | #16 Interior Mutability | 新增 | "Interior Mutability边界"：只有Resource层和Infra层允许RefCell/Cell/Mutex；Domain层和Capability层禁止；与ECS World交互必须通过Commands/Events |
| B3-4 | #20 Compile-time Capability | 新增 | "编译期能力约束"模式：关键能力接口通过trait bound编译期约束；`fn execute<T: CanCast>()` 优于 `if unit.can_cast()` |
| B3-5 | #10 Typestate | 新增 | "Typestate模式适用场景"：Pipeline/Builder模式适用（如Def加载：Unvalidated→Validated→Frozen）；运行时状态机不适用（用enum） |

### B4. SRPG专项规则（`.trae/rules/SRPG专项规则.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B4-1 | #12 Const Trait | 新增 | "编译期元数据"规范：SRPG中固定不变的配置（如技能ID、基础消耗、目标类型）可通过const trait item在编译期确定，减少运行时HashMap查找 |
| B4-2 | #9 Macro+Trait | 新增 | "Derive宏消除样板"优先级：`#[derive(DomainEvent)]` 自动生成Observable+Replayable+Auditable impl；`#[derive(RuleFailure)]` 自动生成code()实现 |

### B5. AI开发宪法（`.trae/rules/AI开发宪法.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B5-1 | #2 Blanket Impl | 新增 | "Blanket Impl自动派生"：当一个能力可由另一个能力自动推导时，必须使用blanket impl，禁止手动重复实现 |
| B5-2 | #13 Sealed Trait | 新增 | "关键Trait密封性"：框架级trait必须sealed，防止外部实现破坏不变量 |

### B6. AI架构准则（`.trae/rules/AI架构准则.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B6-1 | #1 Associated Type | 新增 | "关联类型优先"原则：避免类型爆炸（DamageAbilityError/HealAbilityError/BuffAbilityError），用 `type Error` 统一 |
| B6-2 | #14 Object Safety | 新增 | "Object Safety分层"：热路径泛型/冷路径dyn |

### B7. 测试规范（`.trae/rules/测试规范.md`）

| 编号 | 特性 | 操作 | 具体内容 |
|------|------|------|---------|
| B7-1 | #10 Typestate | 新增 | "Typestate编译期保证测试"：Typestate模式的编译期保证不需要运行时测试，但状态转换逻辑需要测试 |

---

## 四、专项计划 C：架构文件更新

### C1. 需要新增的 ADR

| 编号 | 特性 | ADR编号建议 | 标题 |
|------|------|------------|------|
| C1-1 | #13 Sealed Trait | ADR-0XX | 关键Trait密封性策略 |
| C1-2 | #14 Object Safety | ADR-0XX | Object Safety分层策略（热路径泛型/冷路径dyn） |
| C1-3 | #9 Derive宏 | ADR-0XX | Derive宏+Trait组合模式（消除样板代码） |
| C1-4 | #6 Extension Trait | ADR-0XX | Extension Trait for Bevy Types（EntityCommandsExt/QueryExt） |
| C1-5 | #10 Typestate | ADR-0XX | Typestate模式在Pipeline中的应用 |

### C2. 需要修改的 ADR

| 编号 | 特性 | ADR | 修改方案 |
|------|------|-----|---------|
| C2-1 | #18 Trait Object Registry | ADR-013 | 补充Object Safety设计原则：哪些trait保持object safe、何时允许破坏object safety |
| C2-2 | #9 Macro+Trait | ADR-054 Bevy 0.19迁移 | 补充derive宏自动注册的设计（与Reflect自动注册协同） |
| C2-3 | #7 Plugin Trait | ADR-001 Plugin组合 | 补充自定义DomainPlugin trait的设计（统一领域Plugin接口约束） |

---

## 五、专项计划 D：数据架构文件更新

| 编号 | 特性 | 文件 | 修改方案 |
|------|------|------|---------|
| D1-1 | #10 Typestate | `docs/04-data/` 相关文件 | Content Pipeline的Def加载流程可引入Typestate：`DefBuilder<Unvalidated>` → `DefBuilder<Validated>` → `DefFrozen`，编译期保证未验证的Def不能注册 |
| D1-2 | #15 Cow | `docs/04-data/` 相关文件 | LocalizationKey Schema设计应使用 `Cow<'static, str>` 而非 `String`，静态文本零分配 |
| D1-3 | #11 PhantomData | `docs/04-data/foundation/id_strategy.md` | SpecId应设计为 `SpecId<D>` 泛型形式，编译期避免BuffId传入AbilitySystem |

---

## 六、专项计划 E：领域文件更新

| 编号 | 特性 | 文件 | 修改方案 |
|------|------|------|---------|
| E1-1 | #4 Marker Trait | `docs/02-domain/capabilities/event_domain.md` | 增加 Marker Trait 在事件系统中的应用：`trait DomainEvent {}` / `trait ReplayEvent {}` / `trait AuditEvent {}` 驱动自动注册 |
| E1-2 | #2 Blanket Impl | `docs/02-domain/capabilities/event_domain.md` | 增加 Blanket Impl 自动派生：`impl<T: DomainEvent> Replayable for T {}` |
| E1-3 | #20 Compile-time Capability | `docs/02-domain/domains/combat_domain.md` | 增加编译期能力约束在战斗域的应用：`trait CanAttack {}` / `trait CanCast {}` / `trait CanMove {}` |
| E1-4 | #12 Const Trait | `docs/02-domain/capabilities/ability_domain.md` | 增加编译期元数据在技能域的应用：`trait AbilitySpec { const ID: AbilityId; const COST: u32; }` |

---

## 七、专项计划 F：UI设计文件更新

| 编号 | 特性 | 文件 | 修改方案 |
|------|------|------|---------|
| F1-1 | #6 Extension Trait | `docs/06-ui/` 相关文件 | UI层可使用EntityCommands Extension Trait简化Widget spawn代码 |
| F1-2 | #15 Cow | `docs/06-ui/` 相关文件 | UI文本展示应使用 `Cow<'static, str>` 避免静态文本堆分配 |

---

## 八、执行优先级排序

### P0 — 立即执行（影响架构安全性和代码正确性）

| 序号 | 特性 | 计划编号 | 核心行动 | 理由 |
|------|------|---------|---------|------|
| 1 | #13 Sealed Trait | A1-1, B3-1, B5-2, C1-1 | 保护关键trait不被外部实现 | 框架级trait无密封保护是架构风险 |
| 2 | #16 Interior Mutability | A1-2, B3-3 | 文档化Interior Mutability边界 | 实践合规但未文档化，新开发者可能违规 |
| 3 | #4 Marker Trait | A1-4, A3-1, E1-1 | 修正宪法"禁止Trait分类"条款 | 当前条款过于严格，阻碍了合法的Marker Trait使用 |

### P1 — 近期执行（消除大量样板代码）

| 序号 | 特性 | 计划编号 | 核心行动 | 理由 |
|------|------|---------|---------|------|
| 4 | #9 Derive宏 | A2-3, B4-2, C1-3 | 创建 `#[derive(DomainEvent)]` / `#[derive(RuleFailure)]` | RuleFailure 17个手动impl、DefinitionType 15个手动impl是最大样板代码源 |
| 5 | #2 Blanket Impl | A2-2, B5-1, E1-2 | 为ObservableEvent等添加blanket impl | `impl<T: DomainEvent> Replayable for T {}` 少写几千行 |
| 6 | #1 Associated Type | A2-1, B1-1, B6-1 | 业务层trait引入关联类型模式 | 避免Error/Context类型爆炸 |
| 7 | #14 Object Safety | A1-3, B3-2, B6-2, C1-2 | 制定热路径泛型/冷路径dyn分层策略 | 影响编译时间和运行时性能 |

### P2 — 中期执行（提升编码质量和类型安全）

| 序号 | 特性 | 计划编号 | 核心行动 | 理由 |
|------|------|---------|---------|------|
| 8 | #6 Extension Trait | B1-3, B2-3, C1-4, F1-1 | 为EntityCommands/Query添加扩展方法 | 提升ECS操作流畅度 |
| 9 | #11 PhantomData | B1-6, D1-3 | SpecId改为泛型形式，InstanceId扩展使用 | 编译期类型安全 |
| 10 | #15 Cow | A1-6, B1-4, D1-2, F1-2 | LocalizationKey使用Cow<'static,str> | 静态文本零分配 |
| 11 | #5 Trait Alias | B1-2 | 统一领域事件约束别名 | 简化约束书写 |
| 12 | #20 Compile-time Capability | A1-5, B3-4, E1-3 | 关键能力接口编译期约束 | 编译期保证优于运行时检查 |
| 13 | #19 ZST | A2-4, B2-1 | 显式化ZST概念，扩展使用 | 编译期分类零开销 |
| 14 | #8 SystemSet泛型注册 | B2-2 | 统一SystemSet分组 | 大型项目系统注册规范 |
| 15 | #12 Const Trait | B4-1, E1-4 | 编译期元数据模式 | 减少运行时HashMap查找 |

### P3 — 远期执行（高级特性，当前阶段收益有限）

| 序号 | 特性 | 计划编号 | 核心行动 | 理由 |
|------|------|---------|---------|------|
| 16 | #10 Typestate | B3-5, B7-1, C1-5, D1-1 | Pipeline/Builder引入Typestate | 编译期状态保证，但实现复杂度较高 |
| 17 | #17 QueryExt DSL | B1-5 | 自定义ECS查询DSL | 收益高但需要稳定的Query模式后再抽象 |
| 18 | #21 HRTB | — | Pipeline/Rule Engine高级场景 | 使用频率低，仅在特定架构需求时引入 |
| 19 | #7 DomainPlugin trait | C2-3 | 自定义领域Plugin接口约束 | 62个Plugin已统一使用Bevy标准Plugin trait，自定义trait收益有限 |

---

## 九、关键发现总结

### 项目优势（已成熟应用的特性）

1. **Newtype**（#3）— 25+种强类型ID，通过宏系统化生成，是最成熟的特性应用
2. **Plugin Trait封装**（#7）— 62个Plugin统一使用Bevy标准Plugin trait
3. **Trait + Associated Type**（#1）— 在框架层（DefinitionType、AssetLoader）已全面使用
4. **Trait Object Registry**（#18）— ADR-013有完整的决策标准和迁移路径

### 系统性缺失（14个特性完全无文档指导）

最关键的4个缺失（与架构安全性直接相关）：
1. **Sealed Trait**（#13）— 关键trait无密封保护，外部实现可能破坏不变量
2. **Interior Mutability边界**（#16）— 实践合规但未文档化，新开发者可能违规
3. **Object Safety分层**（#14）— 无策略指导，可能导致编译时间爆炸
4. **Compile-time Capability**（#20）— 宪法要求编译期类型隔离但无实现模式

### 最大样板代码消除机会

1. **Derive宏**（#9）— RuleFailure 17个手动impl + DefinitionType 15个手动impl = 最大样板代码源
2. **Blanket Impl**（#2）— `impl<T: DomainEvent> Replayable for T {}` 可少写几千行
3. **Extension Trait**（#6）— EntityCommands/Query扩展方法提升ECS操作流畅度

### 与37条宝贵经验的交叉

以下Rust特性直接支撑37条经验的落地：
- **#9 Derive宏** → 支撑经验#4（Reflect消灭注册代码）和#7（Macro只做重复结构）
- **#18 Trait Object Registry** → 支撑经验#3（Trait Object减少match）和#15（Registry优于枚举）
- **#13 Sealed Trait** → 支撑经验#28（SSOT唯一真相源）和#22（Domain隔离ECS）
- **#20 Compile-time Capability** → 支撑经验#13（Capability而非类型层级）
- **#6 Extension Trait** → 支撑经验#6（Query Facade）和#8（Bundle Factory）
- **#2 Blanket Impl** → 支撑经验#12（Content Pipeline > Code Architecture）
