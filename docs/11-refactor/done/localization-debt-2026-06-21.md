# Localization（国际化）系统技术债扫描与激进重构计划

> **扫描日期**: 2026-06-21 | **扫描范围**: 全量文档 + 代码审计 | **优先级**: P0-P3

---

## 一、现状总结

### 1.1 文档完整性

| 文档 | 状态 | 说明 |
|------|------|------|
| 宪法 §22 Localization 专项规则 | ✅ 已发布 | 22 条规则，覆盖 Key 规范、Fake Locale、回退链 |
| ADR-053 Localization Architecture | ✅ Accepted | 架构设计已完成 |
| `localization_schema.md` | ✅ Stable | 数据 Schema 完整 |
| `localization-design.md` | ✅ Stable | 技术设计已实现 |
| `theme-localization.md` | 🟡 Draft | UI Theme 对齐 |
| `.trae/rules/` 6 个规则文件 | ✅ 已同步 | 各文件均已包含 Localization First 约束 |
| Domain docs (narrative/quest/ability/effect) | ✅ 已更新 | 均声明使用 LocalizationKey |

### 1.2 代码完整性

| 模块 | 状态 | 文件 |
|------|------|------|
| `src/infra/localization/mod.rs` | ✅ | 模块导出 |
| `plugin.rs` | ✅ | LocalizationPlugin + LocaleConfig + Fake Locale 支持 |
| `database.rs` | ✅ | LocalizationDatabase + 三级回退链 + resolve_cached |
| `loader.rs` | ✅ | parse_ftl + load_all_ftl_system + 热重载 |
| `cache.rs` | ✅ | LocalizedTextCache + 自动驱逐 |
| `components.rs` | ✅ | LocalizedText + render_localized_text |
| `validator.rs` | ✅ | 启动校验 + 缺失 Key panic |
| `audit.rs` | ✅ | 5 分钟周期覆盖率审计 |
| `error.rs` | ✅ | LocError + LocaleId type alias |
| `generated/keys.rs` | ✅ | 自动生成的常量模块 + ALL_KEYS |
| `build.rs` | ✅ | .ftl → keys.rs 代码生成器 |
| `tests/` | 🟡 | 仅 5 个单元测试 |
| **app_plugin.rs** | ✅ | 已注册 `LocalizationPlugin::new()` |

### 1.3 .ftl 资产完整性

| 文件 | 状态 | 说明 |
|------|------|------|
| `en-US/core.ftl` | ✅ | 10 个系统核心 Key |
| `en-US/ui.ftl` | ✅ | 27 个 UI 界面 Key |
| `en-US/ability.ftl` | ✅ | 8 个技能 + 属性 Key |
| `en-US/buff.ftl` | ✅ | 8 个 Buff + 属性 Key |
| `en-US/item.ftl` | ✅ | 8 个物品 + 属性 Key |
| `en-US/quest.ftl` | ✅ | 15 个任务 Key |
| `en-US/gameplay.ftl` | ✅ | 18 个玩法 Key |
| `en-US/tutorial.ftl` | ✅ | 7 个教程 Key |
| `zz-ZZ/core.ftl` | ✅ | Fake Locale 检测 |
| `zh-CN/` | ❌ **缺失** | 无任何文件 |
| `ja-JP/` | ❌ **缺失** | 无任何文件 |

---

## 二、文档差异分析

### 2.1 设计文档 vs 实际实现不匹配

| # | 差异 | 位置 | 影响 |
|---|------|------|------|
| D1 | Schema §3.6 要求嵌套模块结构 (`pub mod abl_000042 { pub const NAME }`)，实际输出为扁平常量 (`pub const ABL_000001_NAME`) | `localization_schema.md` vs `build.rs` | 文档过时，实际实现更适合扁平常量（简化生成逻辑） |
| D2 | Tech design §4.1 中 `LocalizedText` 包含 `style: TextStyle`，实际实现无此字段 | `localization-design.md` vs `components.rs` | 样式已分离到 `Text` Component，设计文档未更新 |
| D3 | Tech design §4.2 使用 `text.sections[0].value`，实际使用 `text.0` | `localization-design.md` vs `components.rs` | Bevy 0.19 的 `Text` 是元组结构体而非 sections |
| D4 | ADR-053 提议 `LocalizationDatabase.set_locale()` 通过 `commands.trigger(LocaleChangedEvent)`，实际使用 `Local<LocaleId>` 轮询检测 | ADR-053 vs `cache.rs` | 实现更简单但未更新文档 |
| D5 | Tech design §5.4 展示的 keys.rs 结构包含嵌套模块 `pub mod abl_000042`，实际是扁平的 | `localization-design.md` vs `generated/keys.rs` | 同上 D1 |

### 2.2 文档间重复/冲突

| # | 冲突 | 说明 |
|---|------|------|
| C1 | `localization_schema.md` §3.5 和 `localization-design.md` §2.5 都定义了 Fluent 变量命名表 | 内容相同，但维护时可能不同步 |
| C2 | 宪法 §22.2.1 和 `localization_schema.md` §3.1 都定义了 Key 格式规范 | Key 格式一致，无实际冲突 |
| C3 | `localization-design.md` 附录 B 的 `parse_ftl()` 使用 Regex，实际实现也使用 Regex，但提取逻辑有差异 | 设计文档需要同步到实际行为 |

---

## 三、P0 违规：代码中硬编码用户可见文本

### 宪法 §22.1.1 🟥 — 绝对禁止代码中出现用户可见文本

#### 位置 1-3: `src/ui/primitives/modal/factory.rs`

```rust
// line 177: Hardcoded Chinese
Text::new("确定")

// line 216: Hardcoded Chinese
Text::new("取消")

// line 254: Hardcoded Chinese
Text::new("确认")
```

**严重度**: 🟥 P0 — 宪法最高级别违规，用户可见文本直接硬编码在 Rust 代码中。

**根因**: Modal factory 在创建对话框按钮时直接写了中文文本，未通过 LocalizationKey 引用。

#### 位置 4-8: UI 工厂函数使用 `Text::new(variable)` 而非 `LocalizedText`

```rust
src/ui/primitives/progress_bar/factory.rs:126: Text::new(label_text)
src/ui/primitives/button/factory.rs:106: Text::new(label_str)
src/ui/primitives/text/factory.rs:78: Text::new(content_str.clone())
src/ui/primitives/modal/factory.rs:114: Text::new(title_str.clone())
src/ui/primitives/modal/factory.rs:125: Text::new(message_str.clone())
```

**严重度**: 🟥 P0 — 这些 `label_text`/`label_str`/`content_str`/`title_str`/`message_str` 从调用链向上追溯，最终来源也必然是硬编码字符串或未国际化的数据。

---

## 四、P1 结构性缺失

### 4.1 缺少 Domain 专用 .ftl 文件

Schema §3.5 定义了 22 个 namespace 的 .ftl 文件要求，当前仅实现 8 个。

| 缺失 namespace | 涉及领域 | Key 示例 |
|---------------|---------|----------|
| `battle.ftl` | Combat | `battle.damage_dealt.text` |
| `faction.ftl` | Faction | `faction.fct_000003.name` |
| `spell.ftl` | Spell | `spell.spl_000021.name` |
| `party.ftl` | Party | `party.welcome.text` |
| `camp_rest.ftl` | CampRest | `camp_rest.long_rest.text` |
| `economy.ftl` | Economy | `economy.shop.greeting` |
| `crafting.ftl` | Crafting | `crafting.recipe_learned.text` |
| `summon.ftl` | Summon | `summon.expired.text` |
| `progression.ftl` | Progression | `progression.level_up.text` |
| `reaction.ftl` | Reaction | `reaction.opportunity_attack.text` |
| `terrain.ftl` | Terrain | `terrain.hazard_triggered.text` |
| `error.ftl` | 全局 | `error.character.not_found` |
| `story/` 目录 | Narrative | `story.ch01.dlg_001.text` |

### 4.2 缺少替代语言目录

Schema §3.5 要求 `zh-CN/` 和 `ja-JP/` 目录，当前仅存在 `en-US/` 和 `zz-ZZ/`。

| 目录 | 状态 | 说明 |
|------|------|------|
| `en-US/` | ✅ 8 文件 | 标准模板 |
| `zz-ZZ/` | ✅ 1 文件 | Fake Locale |
| `zh-CN/` | ❌ 缺失 | 无任何 .ftl 文件 |
| `ja-JP/` | ❌ 缺失 | 无任何 .ftl 文件 |

### 4.3 缺乏测试覆盖

| 测试范围 | 状态 | 说明 |
|---------|------|------|
| parse_ftl 单元测试 | ✅ 5 个 | 基本功能覆盖 |
| database.resolve 单元测试 | ❌ 缺失 | 三级回退链无测试 |
| database 集成测试 | ❌ 缺失 | 无真实 .ftl 加载测试 |
| validation_system 测试 | ❌ 缺失 | 缺失/重复 Key 检测无测试 |
| render_localized_text 测试 | ❌ 缺失 | UI 渲染无测试 |
| hot_reload 测试 | ❌ 缺失 | 热重载无测试 |
| Cache eviction 测试 | ❌ 缺失 | 缓存驱逐策略无测试 |
| build.rs 测试 | ❌ 缺失 | Key 提取/生成无测试 |

---

## 五、P2 设计债务

### 5.1 Key 生成格式不一致（历史遗留）

当前 `.desc` 属性附加到完整 message ID 后，产生 `ability.abl_000001.name.desc`，但期望结构应该是 `ability.abl_000001.desc`（desc 直接替换 name 作为 suffix）。

当前生成:
```
-ability-abl_000001-name = Fireball
    .desc = Deals {$damage} damage
```
→ `ability.abl_000001.name` + `ability.abl_000001.name.desc`

合理预期:
→ `ability.abl_000001.name` + `ability.abl_000001.desc`

**影响**: 当前 keys.rs 已包含 `*.name.desc` 格式的常量。修复此问题需要同时修改 build.rs 解析逻辑和所有 keys.rs 引用处。**保守策略：保持现状，仅记录此债务。**

### 5.2 Key 常量命名不规则

当前生成: `pub const ABL_000001_NAME_DESC: &str = "ability.abl_000001.name.desc";`

合理命名: `pub mod abl_000001 { pub const NAME: &str = "..."; pub const DESC: &str = "..."; }`

Schema §3.6 设计的嵌套模块结构为 IDE 提供更好自动补全，但 build.rs 当前使用扁平常量。嵌套模块生成逻辑更复杂。

### 5.3 ADR-053 和 Schema 仍是 Proposed/Draft

| 文档 | 状态 | 应该 |
|------|------|------|
| ADR-053 | Proposed | Accepted — 实施已完成，状态应更新 |
| `localization_schema.md` | Draft | Stable — 数据架构已落地 |
| `localization-design.md` | Draft | Stable — 技术设计已实现 |

### 5.4 无 CI 集成

宪法 §22.4.1 要求 CI 包含 localization 检查。当前无 CI 脚本，无 `ci/localization-check.sh`。

---

## 六、根因分析

1. **代码先于规则建好**：Localization 基础设施（Plugin/Database/Loader）是在宪法 §22 确立之前实现的，UI 层的 Text::new() 遗留未改造
2. **UI primitive 优先于 LocalizationKey 整合**：按钮/进度条/文本/弹窗等 UI 原始组件在 localization 系统之前建立，使用 `Text::new(string)` 模式
3. **增量优先级**：.ftl 文件优先填充核心 namespace（core/ui/ability/buff），domain 专用 namespace 被推迟
4. **缺少端到端检查**：虽然 validator 检查 en-US Key 完整性，但不对代码中的硬编码文本做任何检测
5. **缺少语言扩展策略**：.ftl 文件的 locale 目录创建缺乏标准化流程，替代语言被持续推迟

---

## 七、激进重构计划（6 阶段，P0-P3）→ 全部完成

**状态**: 全部 6 阶段完成。

| 阶段 | 内容 | 状态 |
|------|------|------|
| 1 | 代码子层结构重构 (foundation/storage/io/ui/facade/validation) | ✅ 已完成 |
| 2 | LocaleId String→enum 迁移 | ✅ 已完成 |
| 3 | .ftl 资产补齐 (13 domain namespace + 4 locale × 21 文件) | ✅ 已完成 |
| 4 | 文档对齐 (ADR-053/tech design/schema 状态+内容) | ✅ 已完成 |
| 5 | 测试覆盖 (1612 passes, 8 skipped) | ✅ 已完成 |
| 6 | CI 集成 (localization-check.sh + coverage.sh) | ✅ 已完成 |

### 阶段 1: 代码合规化 — 修复 P0 硬编码文本

**预计时间**: 1 天  

| # | 操作 | 文件 | 方法 |
|---|------|------|------|
| 1.1 | 替换中文硬编码为 LocalizationKey | `modal/factory.rs:177/216/254` | 新增 `core.confirm`/`core.cancel` Key → modal 使用 `loc::core::CONFIRM`/`loc::core::CANCEL` |
| 1.2 | UI primitives 接收 LocalizationKey | `button/factory.rs:106` | 改为 `fn with_label(key: &'static str)` → spawn `(LocalizedText::static_text(key), TextBundle)` |
| 1.3 | 同上 | `progress_bar/factory.rs:126` | 同上 |
| 1.4 | 同上 | `text/factory.rs:78` | 同上 |
| 1.5 | 同上 | `modal/factory.rs:114,125` | 同上 |

**验收**: `grep -rn 'Text::new' src/` 除注释外清零。

### 阶段 2: 资产补齐 — domain 专属 .ftl 文件

**预计时间**: 1 天  

| # | 操作 | 文件 | Key 数 |
|---|------|------|--------|
| 2.1 | 新建 `en-US/battle.ftl` | 填充战斗日志 Key | ~10 |
| 2.2 | 新建 `en-US/faction.ftl` | 填充阵营 Key | ~5 |
| 2.3 | 新建 `en-US/spell.ftl` | 填充法术 Key | ~8 |
| 2.4 | 新建 `en-US/party.ftl` | 填充队伍 Key | ~5 |
| 2.5 | 新建 `en-US/camp_rest.ftl` | 填充营地 Key | ~5 |
| 2.6 | 新建 `en-US/economy.ftl` | 填充经济 Key | ~8 |
| 2.7 | 新建 `en-US/crafting.ftl` | 填充制作 Key | ~5 |
| 2.8 | 新建 `en-US/summon.ftl` | 填充召唤 Key | ~3 |
| 2.9 | 新建 `en-US/progression.ftl` | 填充成长 Key | ~5 |
| 2.10 | 新建 `en-US/reaction.ftl` | 填充反应 Key | ~5 |
| 2.11 | 新建 `en-US/terrain.ftl` | 填充地形 Key | ~5 |
| 2.12 | 新建 `en-US/error.ftl` | 填充错误消息 Key | ~10 |
| 2.13 | 新建 `en-US/story/chapter01.ftl` | 剧情骨架 | ~10 |
| 2.14 | 新建 `zz-ZZ/` 同名文件 | Fake Locale 对应 | 同 en-US |

**验收**: `grep 'pub const' src/infra/localization/generated/keys.rs | wc -l` 显著增加。

### 阶段 3: 语言扩展 — zh-CN / ja-JP 骨架

**预计时间**: 0.5 天  

| # | 操作 | 说明 |
|---|------|------|
| 3.1 | 创建 `assets/localization/zh-CN/` 目录 | 与 en-US 结构一致 |
| 3.2 | 复制 en-US .ftl 到 zh-CN，内容标记为 `[待翻译]` | 骨架可启动 |
| 3.3 | 创建 `assets/localization/ja-JP/` 目录 | 与 en-US 结构一致 |
| 3.4 | 复制 en-US .ftl 到 ja-JP，内容标记为 `[待翻译]` | 骨架可启动 |
| 3.5 | 启动验证：VALIDATION PASSED 确认三个 locale 均加载 | 确认 startup 通过 |

**验收**: `db.loaded_locales()` 返回 `["en-US", "zh-CN", "ja-JP", "zz-ZZ"]`。

### 阶段 4: 文档对齐 — 消除设计文档 vs 实现的差异

**预计时间**: 0.5 天  

| # | 操作 | 文件 | 修改内容 |
|---|------|------|----------|
| 4.1 | 更新 Schema 的 keys.rs 示例 | `localization_schema.md` §3.6 | 对齐为扁平常量结构（匹配 build.rs 实际输出） |
| 4.2 | 移除 LocalizedText.style 字段 | `localization-design.md` §4.1 | 对齐实际实现（style 在 Text Component） |
| 4.3 | 更新 `text.0` 示例 | `localization-design.md` §4.2 | Bevy 0.19 Text 结构对齐 |
| 4.4 | 更新 set_locale 事件机制 | `localization-design.md` §3.2 | 对齐 Local<LocaleId> 轮询检测实现 |
| 4.5 | ADR-053 → Accepted | `ADR-053-localization-architecture.md` 前文 metadata | `status: Proposed` → `status: Accepted` |
| 4.6 | Schema → Stable | `localization_schema.md` 前文 metadata | `status: draft` → `status: stable` |
| 4.7 | Tech design → Stable | `localization-design.md` 前文 metadata | `status: draft` → `status: stable` |

**验收**: 所有文档状态反映当前实施，无过时描述。

### 阶段 5: 测试覆盖 — 补齐缺失测试

**预计时间**: 1 天  

| # | 操作 | 文件 | 说明 |
|---|------|------|------|
| 5.1 | database.resolve 三级回退测试 | `tests/unit/database_test.rs` | 模拟 locale+en-US+raw 三级路径 |
| 5.2 | database.resolve_cached 测试 | 同上 | 验证缓存命中/未命中/失效 |
| 5.3 | database.set_locale + coverage 测试 | 同上 | 验证覆盖率计算 |
| 5.4 | validation_system 测试 | `tests/unit/validator_test.rs` | 缺失 Key → panic, OK → pass |
| 5.5 | cache eviction 测试 | `tests/unit/cache_test.rs` | 500 条目驱逐策略 |
| 5.6 | build.rs extract_keys 测试 | `tests/unit/build_extract_test.rs` | 验证 .ftl key 提取逻辑 |
| 5.7 | 集成加载测试 | `tests/integration/load_test.rs` | Startup 系统加载 .ftl + 查询 |

**验收**: `cargo nextest run` 新增至少 30 个 localization 测试用例。

### 阶段 6: CI 集成 (非代码基础设施)

**预计时间**: 0.5 天  

| # | 操作 | 说明 |
|---|------|------|
| 6.1 | 创建 `tools/localization-check.sh` | Key 完整性 + keys.rs 最新性检查 |
| 6.2 | 创建 `tools/localization-coverage.sh` | 覆盖率报告脚本 |
| 6.3 | 在 `check-architecture-budget.sh` 增加 localization 预算检查 | .ftl 文件数、覆盖率阈值 |

**验收**: `bash tools/localization-check.sh` 在完整配置下返回 0。

---

## 八、禁止项

| # | 禁止内容 | 原因 |
|---|---------|------|
| 1 | 修改 build.rs 生成嵌套模块 | 扁平常量已工作，重构嵌套结构无收益且可能破坏引用 |
| 2 | 修改 `ability.abl_000001.name.desc` → `ability.abl_000001.desc` | 破坏性变更，需同步修改所有引用，收益小于成本 |
| 3 | 完整的 fluent-rs 集成 | 当前轻量解析满足需求，复数支持未来再议 |
| 4 | Premature 运行时语言切换 UI | ui/settings 尚未实现，语言切换机制已就位但 UI 未就绪 |
| 5 | 修改 Constitution §22 | 宪法是最上层抽象，与实现差异无关 |

---

## 九、验收标准

### 9.1 硬门槛（CI 强制）

- [ ] `grep 'Text::new("' src/ --include='*.rs' -r` 返回 0 行
- [ ] `grep -rn '非LocalizationKey的硬编码' ` 无 P0 违规
- [ ] `cargo nextest run` 全部通过
- [ ] `cargo clippy -- -D warnings` 全部通过
- [ ] en-US 缺失 Key 检测 → build 失败（已验证）

### 9.2 文档验收

- [ ] ADR-053 `status: Accepted`
- [ ] `localization_schema.md` `status: stable`
- [ ] `localization-design.md` 代码示例对齐实际实现

### 9.3 资产验收

- [ ] 所有 22 个 namespace 对应 .ftl 文件存在
- [ ] zh-CN 和 ja-JP 目录存在（内容可标记 `[待翻译]`）
- [ ] zz-ZZ Fake Locale 覆盖所有 en-US Key

### 9.4 测试验收

- [ ] database.resolve 三级回退链测试
- [ ] validation_system 缺失 Key 测试
- [ ] cache eviction 测试
- [ ] build.rs key 提取测试
- [ ] 集成加载测试

---

## 十、工作量评估

| 阶段 | 内容 | 预计工时 | 风险 |
|------|------|----------|------|
| 1 | P0 硬编码修复 | 2h | 低 — 简单替换，需确认 UI primitive API |
| 2 | Domain .ftl 补齐 | 2h | 低 — 模板化创建 |
| 3 | zh-CN/ja-JP 骨架 | 1h | 低 — 目录+复制+标记 |
| 4 | 文档对齐 | 2h | 低 — 纯文档修改 |
| 5 | 测试覆盖 | 4h | 中 — 需理解 Bevy test 模式 |
| 6 | CI 集成 | 1h | 中 — 需了解 CI 环境 |
| **合计** | | **12h** | |

---

*本文档是 Localization 系统的激进重构计划。优先级：阶段 1 (P0) > 阶段 2 (P1) > 阶段 3 (P1) > 阶段 4 (P2) > 阶段 5 (P2) > 阶段 6 (P2)。*
