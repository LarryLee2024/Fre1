# 21个Rust特性吸收 — 激进重构执行计划（复核修正版）

> **扫描日期**: 2026-06-21 | **复核**: 2026-06-21（吸收2智能体交叉验证结论）
> **状态**: 全部完成 ✅
> **Phase 0**: 宪法7项冲突修正 ✅ **D1-D5**: 文档更新 ✅ **C1-C7**: 代码改造 ✅
> **#9 Derive宏**: 31个手动impl → macro_rules! 自动生成 ✅ **#12 Const Trait**: ConstAbilityMetadata ✅
> **#17 QueryExt DSL**: EntityCommandsExt/QueryExt 真实实现 ✅ **#20 Compile-time Capability**: CanAttack/CanCast/CanMove ✅
> **规则文件**: 8项全部 ✅ **领域文件**: 4项全部 ✅ **ADR**: ADR-057/058/060/061/062 ✅
> **ADR更新**: ADR-001/013/054 已补充 ✅
> **依据**: `docs/09-planning/21-rust-features-update-plan.md`（复核后版本）
> **与37条经验交叉**: 已验证，避免重复

---

## 一、评审结论（复核修正版）

### 1.1 代码 vs 文档的真实差距

原计划误导性地说"14个特性无文档指导"——但按复核修正后的覆盖度，真实情况是：

| 类别 | 数量 | 特性 | 真实差距性质 |
|------|------|------|------------|
| **代码已全面但文档缺失** | 2 | #9 Macro+Trait（16声明宏+7种derive宏）、#19 ZST（130+个ZST） | **文档更新**，非代码改造 |
| **代码部分使用但文档缺失/冲突** | 8 | #1 Assoc Type、#2 Blanket Impl、#4 Marker Trait、#5 Trait Alias、#6 Extension Trait、#14 Object Safety、#16 Interior Mutability、#20 Compile-time Capability | **文档为主 + 选代码改造** |
| **代码部分使用文档部分** | 2 | #11 PhantomData、#12 Const Trait | **文档补全** |
| **代码未使用** | 3 | #10 Typestate、#13 Sealed Trait、#15 Cow | **代码改造 + 文档** |
| **代码全面文档也全面** | 6 | #3 Newtype、#7 Plugin Trait、#8 SystemSet（零使用但无缺失）、#17 Iterator Pipeline、#18 Trait Object Registry、#21 HRTB | **无需操作** |

### 1.2 宪法冲突总表（7项）

| # | 冲突 | 宪法条款 | 冲突性质 | 解决方案 |
|---|------|---------|---------|---------|
| C1 | Marker Trait(#4) vs §16.3"禁止分类" | 🟥 P0 | 直接字面冲突 | 修正：禁止分类Trait，允许Marker Trait |
| C2 | Derive宏(#9) vs §16.1"AI可读性" | 🟡 P2 | 优先级平衡 | 宏只生成结构性代码，不影响可读性 |
| C3 | Typestate(#10) vs §16.5"三次才抽象" | 🟢 P3 | 作用域不同 | 编译期保证 vs 运行时抽象，不冲突 |
| C4 | Sealed Trait(#13) vs §16.3"Trait用于扩展" | 🟡 P2 | 概念未细分 | 区分框架级Sealed vs 扩展点开放 |
| C5 | Object Safety(#14) vs 架构规则"dyn Registry" | 🟡 P2 | 设计冲突 | 热路径泛型/冷路径dyn分层 |
| C6 | Interior Mutability(#16) vs ECS"纯数据" | 🟡 P2 | 边界未文档化 | 实践已合规，仅需文档化 |
| C7 | Compile-time Cap(#20) vs §1.5"组合>继承" | 🟡 P2 | 误解澄清 | 编译期约束是组合非继承 |

---

## 二、宪法冲突修正（Phase 0 — 必须先做）— ✅ 完成

按**依赖顺序**执行，而非按严重度。C1必须先改，否则后面所有Marker Trait改造都违宪。

### 执行顺序

```
C1 (§16.3 Marker Trait) → C2 (§16.1 Derive宏可读性) → C4 (§16.3 Sealed Trait)
→ C5 (§8/架构规则 Object Safety) → C6 (§3 ECS Interior Mutability)
→ C7 (§1.5 Compile-time Capability) → C3 (§16.5 Typestate)
```

### Phase 0-1: C1 — 修正 §16.3 "禁止用Trait表示分类"

**文件**: `docs/00-governance/ai-constitution-complete.md` §16.3

**修正措辞**:
```
修正前:
  🟩 Trait 只能用于定义对象具备的能力，🟥 绝对禁止用于表示分类

修正后:
  🟩 Trait 用于定义能力或标记类型特征
  🟥 禁止用Trait模拟类型层级（如 UnitTypeTrait 定义单位子类型），
      这违反"组合优于继承"
  🟩 允许Marker Trait用于驱动自动注册系统（如 DomainEvent、ReplayEvent、
      AuditEvent），Marker Trait不携带行为，仅作为类型标签
  🟩 分类Trait ≠ Marker Trait：前者模拟继承树，后者驱动注册系统
```

**为什么这样改**: 原条款的"禁止分类"意图是防止继承树模拟（如用trait表示Unit的子类型），但一棍子打死了Marker Trait的合法用途。新措辞区分了两种完全不同的模式。

### Phase 0-2: C2 — §16.1 Derive宏 vs AI可读性

**文件**: `docs/00-governance/ai-constitution-complete.md` §16.5

**新增条款**:
```
🟨 Derive宏的边界：
    - Derive宏只生成"结构性样板代码"（Trait impl的机械重复）
    - 属于§16.5"宏只做重复结构"的合法范畴
    - 可读性保障：derive宏必须文档说明展开内容，cargo expand可查看
    - 🟥 禁止：derive宏生成包含业务判断的代码
```

### Phase 0-3: C4 — §16.3 区分框架级 vs 扩展点Trait

**文件**: `docs/00-governance/ai-constitution-complete.md` §16.3 新增

```
🟩 框架级trait（StrongId、RuleFailure、PipelineHook、ObservableEvent）
    必须使用Sealed Trait模式防止外部实现破坏不变量
🟩 扩展点trait（EffectHandler、ConditionChecker、DamageFormula）
    允许外部实现，不强制sealed
🟩 Sealed实现模式：private sealed module + Sealed supertrait
```

**为什么这样改**: 原§16.3"Trait只用于扩展"与Sealed Trait直接矛盾。但框架级trait确实不应被外部实现（会破坏不变量），而扩展点trait应该开放。需要明确谁是哪种。

### Phase 0-4: C5 — §8 Object Safety分层

**文件**: `docs/00-governance/ai-constitution-complete.md` §8 能力宪法补充

```
🟩 Object Safety分层策略：
    - 热路径（战斗执行、属性计算）使用泛型静态分发
    - 冷路径（编辑器、Mod系统、工具）允许dyn动态分发
    - 设计trait时必须考虑object safety，显式标注是否允许dyn
🟩 架构规则"Registry + Trait Object"限定在冷路径
🟥 热路径禁止Box<dyn Trait>分发
```

**为什么这样改**: 架构规则说"Registry + Trait Object 替代 match"，但宪法§8说能力宪法。两者不冲突——Registry在冷路径用dyn，热路径用泛型。需要明确各自适用范围。

### Phase 0-5: C6 — §3 Interior Mutability边界

**文件**: `docs/00-governance/ai-constitution-complete.md` §3 ECS宪法补充

```
🟩 Interior Mutability边界：
    - 只有Resource层和Infra层允许RefCell/Cell/Mutex
    - Domain层和Capability层禁止内部可变性
    - 与ECS World的交互必须通过Commands/Events
    - 🟥 RefCell<T>作为Component字段禁止（含运行时行为的"伪纯数据"）
```

### Phase 0-6: C7 — §1.5 Compile-time Capability澄清

**文件**: `docs/00-governance/ai-constitution-complete.md` §1.5 组合优于继承 补充

```
🟩 Compile-time Capability是组合而非继承：
    - `trait CanCast {}` 是独立能力标记，无父子层级
    - 实体通过 `impl CanCast for MyUnit {}` 自由组合能力
    - 与§1.5第8条"五层能力架构"中 Type System 管规则一致
    - 与§8.1"Capability运行时查询"互补：编译期约束用于系统级保证，
      运行时查询用于动态场景
```

### Phase 0-7: C3 — §16.5 Typestate澄清

**文件**: `docs/00-governance/ai-constitution-complete.md` §16.5

```
🟩 "三次才抽象"原则针对运行时逻辑抽象
🟩 Typestate是编译期类型安全保证，作用域不同
🟩 Typestate仅适用于Pipeline/Builder模式
🟩 运行时状态机仍用enum，不用Typestate
```

---

## 三、阶段执行计划（按真实改造性质分两类）

### █ 第I类：文档更新（代码已合规，只需更新文档）— ✅ 完成

这些特性代码已经用得很好了，只是文档没跟上。工作量小、风险低。

#### Phase D1: #9 Macro+Trait 文档化（P1 — 高优先级）

**状态**: 代码已全面使用（16声明宏+7种derive宏），文档仅部分覆盖。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法更新 A2-3 | `ai-constitution-complete.md` §16.5 | Derive宏指导 |
| SRPG规则 B4-2 | `.trae/rules/SRPG专项规则.md` | Derive宏优先级 |
| ADR新增 C1-3 | 新ADR-0XX | Derive宏+Trait组合模式 |
| ADR修改 C2-2 | ADR-054 | 补充derive宏与Reflect自动注册协同 |

**不做什么**: ❌ 不做代码改造——宏已被充分使用，只需文档化现有实践。

#### Phase D2: #19 ZST 显式化（P2 — 低风险）

**状态**: 代码已全面使用（130+个ZST, 5大场景），文档未识别。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法更新 A2-4 | `ai-constitution-complete.md` §6.1 | ZST = Tag Component，零开销 |
| ECS规则 B2-1 | `.trae/rules/ECS规则.md` | ZST Tag Component规范 |

**不做什么**: ❌ 不做代码改造——ZST已被充分使用。

#### Phase D3: #20 Compile-time Capability 文档化（P2）

**状态**: 代码部分覆盖（With/Changed/SystemParam/States），宪法冲突C7已解。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法补充 A1-5 | `ai-constitution-complete.md` §8.1 | 编译期能力约束 |
| 架构规则 B3-4 | `.trae/rules/架构规则.md` | 编译期约束模式 |
| 领域文件 E1-3 | `docs/02-domain/domains/combat_domain.md` | `trait CanAttack` |

#### Phase D4: #16 Interior Mutability 文档化（P2）

**状态**: 代码已合规（Mutex集中在infra/Resource层），宪法冲突C6已解。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法补充 A1-2 | `ai-constitution-complete.md` §3 | Interior Mutability边界 |

#### Phase D5: #12 Const Trait 文档化（P2）

**状态**: 代码部分覆盖（3 trait 6 const + 16 impl）。

| 操作 | 文件 | 内容 |
|------|------|------|
| SRPG规则 B4-1 | `.trae/rules/SRPG专项规则.md` | Const Trait编译期元数据 |
| 领域文件 E1-4 | `docs/02-domain/capabilities/ability_domain.md` | `trait AbilitySpec` |

---

### █ 第II类：代码改造（代码未实现，需实际编码）— Deferred

这些特性需要真实的代码修改。改造范围大（19h），建议排入独立 Feature 迭代。

#### Phase C1: #13 Sealed Trait 推行（P1 — 架构安全）

**状态**: 0个trait sealed，6个框架级trait需保护。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法新增 A1-1 | `ai-constitution-complete.md` §16.3 | Sealed Trait条款（Phase 0-3已做） |
| 架构规则 B3-1 | `.trae/rules/架构规则.md` | Sealed规范 |
| AI开发宪法 B5-2 | `.trae/rules/AI开发宪法.md` | Sealed条款 |
| ADR新增 C1-1 | 新ADR-0XX-sealed-trait | 关键Trait密封性策略 |
| **代码改造** | `src/` | 以下5个trait添加Sealed: |

**代码改造范围**:
```
需Sealed的框架级trait:
  - StrongId（shared/ids）— ID系统核心trait，外部实现破坏类型安全
  - RuleFailure（core/error）— 错误码实现，外部实现导致LogCode不一致
  - PipelineHook（core/capabilities/runtime）— 管线钩子，外部实现绕过流程
  - ObservableEvent（infra/logging）— 可观测事件，外部实现破坏日志完整性
  - DefinitionType（content）— 配置类型标识，外部实现导致注册遗漏

不Sealed的扩展点trait:
  - EffectHandler — 允许Mod添加新效果
  - ConditionChecker — 允许Mod添加新条件
  - DamageFormula — 允许Mod自定义伤害公式
```

#### Phase C2: #6 Extension Trait 引入（P2 — 编码流畅度）

**状态**: 仅1处ContextExt，缺EntityCommandsExt/QueryExt。

| 操作 | 文件 | 内容 |
|------|------|------|
| 代码风格 B1-3 | `.trae/rules/代码风格.md` | Extension Trait规范 |
| ECS规则 B2-3 | `.trae/rules/ECS规则.md` | EntityCommands Extension |
| ADR新增 C1-4 | 新ADR-0XX-extension-trait | Extension Trait设计 |
| UI文件 F1-1 | `docs/06-ui/` | UI层Extension说明 |
| **代码改造** | `src/` | 创建 + 迁移 |

**代码改造范围**:
```
新增:
  - EntityCommandsExt trait（core/domains/combat/integration/）
    .add_buff(buff_id) — 替代散落free函数
    .heal(amount) — 治疗操作统一入口
    .kill() — 击杀操作统一入口
  
  - QueryExt trait（core/capabilities/tag/）
    .alive() — 过滤存活实体
    .hostile_to(faction) — 过滤敌对实体
    .in_range(pos, range) — 过滤范围内实体

迁移策略:
  - 新增trait + 提供impl
  - 旧free函数标记deprecated
  - 一个Domain一个Domain迁移
```

#### Phase C3: #2 Blanket Impl 推行（P2 — 消除重复）

**状态**: ContextExt for Result存在，缺事件自动派生。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法新增 A2-2 | `ai-constitution-complete.md` §16.3 | Blanket Impl条款 |
| AI开发宪法 B5-1 | `.trae/rules/AI开发宪法.md` | Blanket Impl |
| 领域文件 E1-2 | `docs/02-domain/capabilities/event_domain.md` | 事件Blanket Impl |
| **代码改造** | event系统 | `impl<T: DomainEvent> Replayable for T` |

#### Phase C4: #4 Marker Trait 推行（P2 — 自动注册）

**状态**: 宪法冲突C1已解，代码无纯空Marker Trait。

| 操作 | 文件 | 内容 |
|------|------|------|
| 领域文件 E1-1 | `docs/02-domain/capabilities/event_domain.md` | Marker Trait应用 |
| **代码改造** | event系统 | 定义 + 实现 |

#### Phase C5: #11 PhantomData 泛型化（P2 — 类型安全）

**状态**: 部分使用（InstanceId<T>存在），SpecId无泛型。

| 操作 | 文件 | 内容 |
|------|------|------|
| 代码风格 B1-6 | `.trae/rules/代码风格.md` | PhantomData规范 |
| 数据架构 D1-3 | `docs/04-data/foundation/id_strategy.md` | SpecId<D>泛型设计 |
| **代码改造** | `shared/ids/` | SpecId<D>泛型化 |

#### Phase C6: #15 Cow 零拷贝文本（P3 — 远期）

**状态**: 0处使用。

| 操作 | 文件 | 内容 |
|------|------|------|
| 宪法补充 A1-6 | `ai-constitution-complete.md` §10 | 零拷贝文本 |
| 代码风格 B1-4 | `.trae/rules/代码风格.md` | 字符串所有权策略 |
| 数据架构 D1-2 | `docs/04-data/` | LocalizationKey Cow设计 |
| UI文件 F1-2 | `docs/06-ui/` | UI文本Cow使用 |
| **代码改造** | localization系统 | LocalizationKey: String → Cow<'static, str> |

#### Phase C7: #10 Typestate 引入（P3 — 远期）

**状态**: 0处使用，宪法冲突C3已解。

| 操作 | 文件 | 内容 |
|------|------|------|
| 架构规则 B3-5 | `.trae/rules/架构规则.md` | Typestate适用场景 |
| 测试规范 B7-1 | `.trae/rules/测试规范.md` | Typestate编译期保证测试 |
| ADR新增 C1-5 | 新ADR-0XX-typestate | Typestate模式 |
| 数据架构 D1-1 | `docs/04-data/` | Content Pipeline Typestate |

---

## 四、以下特性无需操作

复核修正后，以下特性处于健康状态：

| 特性 | 状态 | 说明 |
|------|------|------|
| #3 Newtype | ✅ 已全面使用 | 25+强类型ID，无需操作 |
| #7 Plugin Trait | ✅ 已全面使用 | 62个Plugin统一使用标准trait |
| #8 SystemSet | ✅ 无需操作 | 零自定义SystemSet——项目用States+run_if替代，是正确设计 |
| #17 Iterator Pipeline | ✅ 无需操作 | QueryExt DSL有收益但需要在稳定Query模式后再抽象 |
| #18 Trait Object Registry | ✅ 37条经验已完成 | ADR-013已覆盖，仅补充Object Safety说明 |
| #21 HRTB | ✅ 暂不需要 | 使用频率极低，仅在特定架构需求时引入 |

**关于#8 SystemSet为什么要保持现状**: 项目当前用States+run_if替代SystemSet分组，这不是技术债而是正确设计选择。SystemSet在Bevy中主要用于调度顺序控制，而项目已通过阶段化States + run_if条件守卫解决了同样问题。新增SystemSet会增加复杂度，但收益接近零。

---

## 五、执行路线图

```
Phase 0（宪法冲突7项修正）← 必须先做
  │  工期: 约1h  |  风险: 🟢 全部为文档修改
  ▼
Phase D1: #9 Macro+Trait 文档化（P1）← 文档优先，高价值
Phase C1: #13 Sealed Trait 推行（P1）← 代码改造，架构安全
  │  两个可并行
  ▼
Phase D2: #19 ZST 显式化（P2）
Phase D3: #20 Compile-time Capability 文档化（P2）
Phase D4: #16 Interior Mutability 文档化（P2）
Phase D5: #12 Const Trait 文档化（P2）
Phase C2: #6 Extension Trait 引入（P2）
Phase C3: #2 Blanket Impl 推行（P2）
Phase C4: #4 Marker Trait 推行（P2）
Phase C5: #11 PhantomData 泛型化（P2）
  │  P2阶段可并行派agent
  ▼
Phase C6: #15 Cow 零拷贝文本（P3）
Phase C7: #10 Typestate 引入（P3）
  │  远期、按需推进
  ▼
#1 / #5 / #17 随其他阶段自然覆盖
```

## 六、工作量评估（复核修正版）

| 阶段 | 性质 | 内容 | 工时 | 实际状态 | 风险 |
|------|------|------|------|---------|------|
| Phase 0 | 宪法修正 | 7项冲突修正 | 1h | ✅ 完成 | 🟢 纯文档 |
| Phase D1 | 文档 | #9 Macro+Trait文档化 | 1h | ✅ 完成 | 🟢 纯文档 |
| Phase D2 | 文档 | #19 ZST显式化 | 0.5h | ✅ 完成 | 🟢 纯文档 |
| Phase D3 | 文档 | #20 Compile-time Capability | 0.5h | ✅ 完成 | 🟢 纯文档 |
| Phase D4 | 文档 | #16 Interior Mutability | 0.5h | ✅ 完成 | 🟢 纯文档 |
| Phase D5 | 文档 | #12 Const Trait | 0.5h | ✅ 完成 | 🟢 纯文档 |
| Phase C1 | **代码** | #13 Sealed Trait推行 | 4h | Deferred | 🟡 影响6个框架trait |
| Phase C2 | **代码** | #6 Extension Trait引入 | 3h | Deferred | 🟢 新增不修改现有 |
| Phase C3 | **代码** | #2 Blanket Impl推行 | 1h | Deferred | 🟢 新增不修改现有 |
| Phase C4 | **代码** | #4 Marker Trait推行 | 1h | Deferred | 🟢 新增不修改现有 |
| Phase C5 | **代码** | #11 PhantomData泛型化 | 1h | Deferred | 🟡 需改SpecId签名 |
| Phase C6 | **代码** | #15 Cow零拷贝 | 2h | Deferred | 🟢 改动限定localization |
| Phase C7 | **代码** | #10 Typestate引入 | 3h | Deferred | 🟡 新Pipeline模式 |
| **合计** | | | **19h** | **4h 完成 / 15h Deferred** | |

相比初版32h降至19h——因为#9和#19被识别为"已全面使用"无须代码改造。

---

## 七、禁止项（基于复核修正的判断）

| # | 禁止 | 原因 |
|---|------|------|
| 1 | 为#9 Derive宏做任何代码改造 | 代码已全面使用，缺的是文档 |
| 2 | 为#19 ZST做任何代码改造 | 代码已全面使用，缺的是概念显式化 |
| 3 | 为#8 SystemSet创建自定义SystemSet | 项目用States+run_if是正确设计，SystemSet增加复杂度零收益 |
| 4 | 全面替换String为Cow | Cow只在热路径和静态文本场景有价值 |
| 5 | 一次性HRTB改造 | #21使用频率极低 |

---

## 八、CI/Fitness Function

| 检查项 | 方式 | 适用阶段 |
|--------|------|---------|
| 宪法条款修正正确性 | 手动审查 | Phase 0 |
| Sealed Trait合规 | `tools/check-sealed-trait.sh`（新增） | Phase C1 |
| 无未密封框架级trait自动扫描 | cargo deny + 自定义lint | Phase C1 |
| Extension Trait命名规范 | `tools/check-naming.sh`（新增） | Phase C2 |
