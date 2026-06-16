---
id: 01-architecture.project-structure
title: Project Structure
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Project Structure Specification

Version: 1.1
Status: Proposed
Source: `docs/其他/30.md` 架构提炼
> **宪法依据**：`docs/AI开发宪法完整版.md` v1.6 — 第1.1节模块划分原则（Feature First）、第4.0节代码组织宪法
> ⚠️ **文件大小警告**（宪法 4.0.3）：本文件超过1000行，建议在后续迭代中考虑拆分。当前保持单一文件是因为项目结构规范需要全局视角，内聚性优先于行数指标。

本文档定义 SRPG 项目的完整目录结构规范。
适用于几十万行代码级别、支持 MOD、AI 协作、外包美术、长期更新的大型项目。

---

## 核心原则

> **宪法依据**：〔宪法 1.1.1 Feature First〕按业务领域拆分模块，绝对禁止按技术类型拆分全局目录

### 三棵树分离

> **宪法依据**：〔宪法 1.1.2 定义与实例分离〕、〔宪法 1.1.3 规则与内容分离〕

项目必须分离为三棵独立的树：

```
项目源码树 (src/)     → 游戏逻辑，Rust 代码
项目内容树 (content/) → 游戏数据，RON 配置
项目资产树 (assets/)  → 美术音频，二进制资源
```

🟥 **绝对禁止**：将配置数据、美术资源、开发脚本混入同一目录。

### 判断标准

每个文件/目录归属，使用三个问题判断：

1. **Core 问题**：如果明天把 Bevy 删了，换成 Godot/Unity/UE/服务器模拟器，这个东西还存在吗？
   - 存在 → `core/`
2. **Infrastructure 问题**：如果游戏规则不变，能不能换一种实现方式？
   - 能 → `infrastructure/`
3. **Shared 问题**：这个东西既不是游戏规则，也不是技术实现，而是所有地方都会用到的基础工具吗？
   - 是 → `shared/`

一句话总结：
```
Core           = 为什么（业务规则 = 游戏规则）
Infrastructure = 怎么做（技术实现）
Shared         = 通用工具（基础能力）
Content        = 是什么（游戏内容 = 配置数据）
```

---

## 顶层结构

```
project/
├── src/                  # Rust 源码（游戏逻辑）
├── assets/               # 运行时资源（美术音频）
├── content/              # 游戏内容（RON 配置数据）
├── mods/                 # MOD 扩展
├── tools/                # 开发工具（独立二进制）
├── scripts/              # 自动化脚本
├── tests/                # 集成测试 & 回放测试
├── benchmarks/           # 性能基准
├── docs/                 # 文档
└── build/                # 构建输出（gitignore）
```

---

## 一、src/ — Rust 源码树

源码树按七层架构组织：

```
src/
├── app/                      # 第一层：游戏启动与装配
│   ├── app_plugin.rs         #   主 Plugin 注册
│   ├── game_state.rs         #   AppState 定义
│   ├── schedules.rs         #   Schedule 定义
│   ├── sets.rs               #   SystemSet 定义
│   ├── startup.rs            #   启动逻辑
│   ├── shutdown.rs           #   关闭逻辑
│   └── plugins.rs            #   所有 Plugin 汇集注册
│
├── core/                     # 第二层：游戏规则（纯领域逻辑）
│   ├── mod.rs
│   │
│   ├── battle/               #   战斗效果管线
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── combat.rs
│   │   ├── events.rs
│   │   ├── record.rs
│   │   ├── log.rs
│   │   └── pipeline/
│   │       ├── mod.rs
│   │       ├── intent.rs
│   │       ├── generate.rs
│   │       ├── modify.rs
│   │       ├── execute.rs
│   │       └── trait_trigger.rs
│   │
│   ├── character/            #   角色与 Trait 扩展体系
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── unit.rs
│   │   ├── faction.rs
│   │   ├── race.rs
│   │   ├── job.rs
│   │   ├── progression.rs
│   │   └── traits/
│   │       ├── mod.rs
│   │       ├── trait_collection.rs
│   │       ├── trait_trigger.rs
│   │       └── trait_effect.rs
│   │
│   ├── skill/                #   技能系统
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── skill_def.rs
│   │   ├── skill_slots.rs
│   │   ├── skill_cooldowns.rs
│   │   ├── skill_preview.rs
│   │   └── domain/
│   │       ├── mod.rs
│   │       └── skill_error.rs
│   │
│   ├── buff/                 #   Buff/Debuff 系统
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── buff_def.rs
│   │   ├── buff_instance.rs
│   │   ├── buff_apply.rs
│   │   ├── buff_tick.rs
│   │   └── domain/
│   │       ├── mod.rs
│   │       └── buff_error.rs
│   │
│   ├── turn/                 #   回合状态机与行动顺序
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── turn_phase.rs
│   │   ├── turn_order.rs
│   │   └── victory.rs
│   │
│   ├── action/               #   行动系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── command/              #   命令模式
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── movement/             #   移动系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── target/               #   目标选择
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── condition/            #   条件判定
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── requirement/          #   需求检查
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── stat/                 #   属性体系（Primary Stat）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── attribute/            #   属性计算（Derived + Modifier）
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── plugin.rs
│   │
│   ├── modifier/             #   修饰规则（ModifierRule + Calculator）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── formula/              #   公式引擎
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── resource/             #   战斗资源（HP/MP/Stamina/AP）
│   │   ├── mod.rs
│   │   ├── hp.rs
│   │   ├── mp.rs
│   │   ├── stamina.rs
│   │   └── ap.rs
│   │
│   ├── equipment/            #   装备系统
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── equipment_def.rs
│   │   ├── equipment_slots.rs
│   │   ├── equipment_instance.rs
│   │   └── equip.rs
│   │
│   ├── weapon/               #   武器子类型
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── armor/                #   护甲子类型
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── accessory/            #   饰品子类型
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── inventory/            #   背包系统
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── item_def.rs
│   │   ├── item_instance.rs
│   │   ├── container.rs
│   │   └── transfer.rs
│   │
│   ├── item/                 #   物品系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── consumable/           #   消耗品
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── crafting/             #   制作系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── summon/               #   召唤系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── faction/              #   阵营系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── relationship/          #   关系系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── ai/                   #   AI 行为系统
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── ai_behavior.rs
│   │   ├── ai_behavior_def.rs
│   │   ├── target_selector.rs
│   │   ├── move_selector.rs
│   │   └── skill_selector.rs
│   │
│   ├── quest/                #   任务系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── dialogue/             #   对话树系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── cutscene/             #   剧情演出系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── loot/                 #   掉落系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── economy/              #   经济系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── achievement/           #   成就系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── reputation/           #   声望系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── stage/                #   关卡系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── chapter/              #   章节系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── campaign/             #   战役系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── world_map/            #   世界地图系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── terrain/              #   地形系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── map/                  #   地图与寻路
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── terrain_grid.rs
│   │   ├── occupancy_grid.rs
│   │   ├── game_map.rs
│   │   └── pathfinding/
│   │       ├── mod.rs
│   │       └── bfs.rs
│   │
│   ├── victory/              #   胜利条件
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── defeat/               #   失败条件
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── cooldown/             #   冷却系统
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── status/               #   状态效果
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── save_domain/          #   存档领域模型
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── effect/               #   效果定义与分发（现有）
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── handler.rs
│   │
│   ├── tag.rs                #   标签系统（GameplayTag 位掩码）
│   ├── tag_def.rs            #   标签定义
│   │
│   ├── attribute_def.rs      #   属性定义
│   │
│   ├── registry_loader.rs    #   注册表加载器
│   ├── snapshot.rs           #   场景快照
│   │
│   └── id/                   #   [移除] → 迁移到 shared/ids/
│       └── ...                #   当前临时位置，待迁移
│
├── shared/                   # 第三层：基础能力（所有模块通用）
│   ├── mod.rs
│   ├── ids/                  #   强类型 ID
│   │   ├── mod.rs
│   │   ├── unit_id.rs
│   │   ├── skill_id.rs
│   │   ├── buff_id.rs
│   │   ├── item_id.rs
│   │   ├── quest_id.rs
│   │   ├── stage_id.rs
│   │   ├── faction_id.rs
│   │   └── equipment_id.rs
│   │
│   ├── error/                #   共享错误工具
│   │   ├── mod.rs
│   │   ├── result.rs         #   GameResult<T> 类型别名
│   │   ├── context.rs         #   错误上下文工具
│   │   └── extensions.rs     #   错误转换 trait
│   │
│   ├── events/               #   跨模块领域事件（公共基础设施）
│   │   ├── mod.rs
│   │   └── event_whitelist.rs
│   │
│   ├── audit/                #   审计轨迹
│   │   ├── mod.rs
│   │   └── whitelist.rs
│   │
│   ├── random/               #   确定性随机数
│   │   └── mod.rs
│   │
│   ├── math/                 #   游戏数学工具
│   │   └── mod.rs
│   │
│   ├── time/                 #   游戏时间工具
│   │   └── mod.rs
│   │
│   ├── collections/          #   通用集合类型
│   │   └── mod.rs
│   │
│   ├── validation/           #   通用校验工具
│   │   └── mod.rs
│   │
│   ├── constants/            #   全局常量
│   │   └── mod.rs
│   │
│   ├── traits/               #   核心 trait 定义（DamageSource, Healable 等）
│   │   └── mod.rs
│   │
│   ├── macros/               #   过程宏
│   │   └── mod.rs
│   │
│   ├── testing/              #   测试工具
│   │   ├── mod.rs
│   │   ├── spawns.rs         #   spawn_test_battle() 等
│   │   ├── assertions.rs     #   领域断言
│   │   └── fixtures.rs       #   测试固件
│   │
│   └── versioning/           #   版本管理工具
│       └── mod.rs
│
├── infrastructure/           # 第四层：技术实现层
│   ├── mod.rs
│   │
│   ├── assets/               #   资源加载与管理
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── asset_error.rs
│   │   └── loaders/
│   │       ├── mod.rs
│   │       ├── ron_loader.rs
│   │       └── manifest_loader.rs
│   │
│   ├── persistence/          #   存档与持久化
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── save/
│   │   │   ├── mod.rs
│   │   │   └── save_error.rs
│   │   ├── load/
│   │   │   ├── mod.rs
│   │   │   └── load_error.rs
│   │   └── migration/
│   │       ├── mod.rs
│   │       └── migration_error.rs
│   │
│   ├── logging/              #   日志基础设施
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── localization/         #   多语言支持
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── replay/               #   战斗回放
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── analytics/            #   数据分析
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── telemetry/            #   遥测
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── config/               #   配置管理
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── importer/             #   数据导入
│   │   ├── mod.rs
│   │   ├── json/
│   │   ├── csv/
│   │   ├── yaml/
│   │   └── excel/
│   │
│   ├── exporter/             #   数据导出
│   │   ├── mod.rs
│   │   └── ...
│   │
│   ├── networking/           #   网络层（未来）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── steam/                #   Steam 集成（未来）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── cloud_save/           #   云存档（未来）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── hot_reload/           #   热重载
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── diagnostics/          #   诊断工具
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── profiler/             #   性能分析
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── crash_report/         #   崩溃报告
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   ├── scripting_runtime/    #   脚本运行时（MOD 支持）
│   │   ├── mod.rs
│   │   └── plugin.rs
│   │
│   └── audit/                #   审计轨迹基础设施
│       ├── mod.rs
│       └── whitelist.rs
│
├── content/                  # 第五层：内容桥接（连接 content/ 数据到 core/ 规则）
│   ├── mod.rs
│   │
│   ├── content_plugin.rs     #   内容加载总 Plugin
│   │
│   ├── skills/               #   技能内容加载
│   │   ├── mod.rs
│   │   └── skill_content.rs
│   │
│   ├── buffs/                #   Buff 内容加载
│   │   ├── mod.rs
│   │   └── buff_content.rs
│   │
│   ├── classes/              #   职业内容加载
│   │   ├── mod.rs
│   │   └── class_content.rs
│   │
│   ├── characters/           #   角色内容加载
│   │   ├── mod.rs
│   │   └── character_content.rs
│   │
│   ├── enemies/              #   敌人内容加载
│   │   ├── mod.rs
│   │   └── enemy_content.rs
│   │
│   ├── items/                #   物品内容加载
│   │   ├── mod.rs
│   │   └── item_content.rs
│   │
│   ├── equipments/           #   装备内容加载
│   │   ├── mod.rs
│   │   └── equipment_content.rs
│   │
│   ├── maps/                 #   地图内容加载
│   │   ├── mod.rs
│   │   └── map_content.rs
│   │
│   ├── stages/               #   关卡内容加载
│   │   ├── mod.rs
│   │   └── stage_content.rs
│   │
│   ├── terrains/             #   地形内容加载
│   │   ├── mod.rs
│   │   └── terrain_content.rs
│   │
│   ├── ai_behaviors/         #   AI 行为内容加载
│   │   ├── mod.rs
│   │   └── ai_behavior_content.rs
│   │
│   ├── quests/               #   任务内容加载
│   │   ├── mod.rs
│   │   └── quest_content.rs
│   │
│   ├── dialogues/            #   对话内容加载
│   │   ├── mod.rs
│   │   └── dialogue_content.rs
│   │
│   ├── factions/             #   阵营内容加载
│   │   ├── mod.rs
│   │   └── faction_content.rs
│   │
│   ├── loot_tables/          #   掉落表内容加载
│   │   ├── mod.rs
│   │   └── loot_content.rs
│   │
│   ├── shops/                #   商店内容加载
│   │   ├── mod.rs
│   │   └── shop_content.rs
│   │
│   └── achievements/         #   成就内容加载
│       ├── mod.rs
│       └── achievement_content.rs
│
├── modding/                  # 第六层：MOD 支持
│   ├── mod.rs
│   │
│   ├── api/                  #   MOD API 暴露
│   │   ├── mod.rs
│   │   ├── mod_api.rs
│   │   └── safe_exports.rs
│   │
│   ├── registry/             #   MOD 注册表
│   │   ├── mod.rs
│   │   ├── mod_registry.rs
│   │   └── dependency_resolver.rs
│   │
│   ├── loaders/              #   MOD 加载器
│   │   ├── mod.rs
│   │   └── mod_loader.rs
│   │
│   ├── validators/           #   MOD 校验器
│   │   ├── mod.rs
│   │   ├── schema_validator.rs
│   │   └── conflict_detector.rs
│   │
│   ├── sandbox/              #   MOD 沙箱环境
│   │   ├── mod.rs
│   │   └── sandbox_runner.rs
│   │
│   └── compatibility/         #   MOD 兼容性
│       ├── mod.rs
│       └── version_checker.rs
│
├── debug/                    # 调试工具（独立于业务）
│   ├── mod.rs
│   ├── plugin.rs
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── battle_debugger.rs
│   │   ├── buff_viewer.rs
│   │   ├── attribute_viewer.rs
│   │   └── turn_viewer.rs
│   └── viewers/
│       ├── mod.rs
│       └── ...
│
├── ui/                       # UI 表现层（独立于业务逻辑）
│   ├── mod.rs
│   ├── plugin.rs
│   ├── view_models.rs
│   ├── focus.rs
│   ├── settings.rs
│   ├── camera.rs
│   ├── highlight.rs
│   ├── combat_vfx_handler.rs
│   ├── combat_log_handler.rs
│   ├── combat_preview.rs
│   ├── action_menu.rs
│   ├── command_handler.rs
│   ├── events.rs
│   ├── theme.rs
│   ├── screens/
│   ├── panels/
│   └── widgets/
│
├── lib.rs                    # 库入口
└── main.rs                   # 主入口
```

### 源码树关键约束

> **宪法依据**：〔宪法 1.3.2 依赖方向铁则〕、〔宪法 3.0.1 接口最小化原则〕、〔宪法 3.0.7 通用代码规范〕

1. 🟥 **core/ 禁止依赖任何业务模块** — core 模块的 `use` 语句不得出现 `battle/`, `skill/`, `buff/` 等 〔宪法 1.3.2〕
2. 🟥 **shared/ 禁止包含任何业务逻辑** — shared 只放通用工具，不放 `skill_utils`、`buff_utils` 等 〔宪法 3.0.7〕
3. 🟥 **infrastructure/ 禁止包含领域错误** — `SkillError` 放在 `core/skill/`，不在 `infrastructure/` 〔宪法 13.9.1〕
4. 🟩 **content/ 是唯一的内容桥接层** — 连接外部 RON 数据到内部 Registry 〔宪法 1.1.3〕
5. 🟩 **modding/ 只暴露稳定 API** — MOD 作者只需要了解 `modding/api/` 暴露的接口 〔宪法 3.0.1〕

---

## 二、assets/ — 运行时资源树

```
assets/
├── art/
│   ├── characters/           # 角色美术
│   │   ├── knight/
│   │   ├── mage/
│   │   ├── priest/
│   │   └── assassin/
│   │       ├── sprite/
│   │       ├── animation/
│   │       ├── portrait/
│   │       ├── avatar/
│   │       └── vfx/
│   │
│   ├── enemies/              # 敌人美术
│   ├── npc/                  # NPC 美术
│   ├── summons/              # 召唤物美术
│   ├── bosses/               # Boss 美术
│   │
│   ├── portraits/            # 头像资源
│   ├── faces/                # 表情资源
│   │
│   ├── skills/               # 技能特效图标
│   ├── buffs/                # Buff 图标
│   ├── status/               # 状态图标
│   │
│   ├── weapons/              # 武器美术
│   ├── armors/               # 护甲美术
│   ├── accessories/          # 饰品美术
│   ├── items/                # 物品美术
│   │
│   ├── maps/                 # 地图美术
│   │   ├── battle_maps/
│   │   ├── town_maps/
│   │   ├── dungeon_maps/
│   │   ├── world_maps/
│   │   └── templates/
│   │
│   ├── terrains/             # 地形瓦片
│   │   ├── grass/
│   │   ├── sand/
│   │   ├── snow/
│   │   ├── lava/
│   │   ├── water/
│   │   ├── city/
│   │   └── dungeon/
│   │
│   ├── props/                # 地图物件
│   ├── buildings/            # 建筑
│   ├── decorations/          # 装饰
│   ├── world_map/            # 世界地图美术
│   │
│   ├── effects/              # 特效
│   │   ├── hit/
│   │   ├── slash/
│   │   ├── fire/
│   │   ├── ice/
│   │   ├── lightning/
│   │   ├── holy/
│   │   ├── dark/
│   │   └── poison/
│   │
│   ├── projectiles/          # 投射物
│   ├── backgrounds/          # 背景图
│   ├── illustrations/        # 插画
│   ├── cg/                   # CG
│   └── marketing/            # 宣传素材
│
├── audio/
│   ├── bgm/                  # 背景音乐
│   ├── battle_bgm/
│   ├── town_bgm/
│   ├── dungeon_bgm/
│   ├── boss_bgm/
│   ├── sfx/                  # 音效
│   │   ├── battle/
│   │   ├── skills/
│   │   ├── buffs/
│   │   ├── ui/
│   │   ├── items/
│   │   ├── footsteps/
│   │   └── environment/
│   ├── voice/                # 语音
│   │   ├── characters/
│   │   ├── npcs/
│   │   ├── enemies/
│   │   ├── system/
│   │   └── battle/
│   ├── ambience/             # 环境音
│   └── music_stems/          # 音乐分轨
│
├── ui/
│   ├── atlas/                # UI 图集
│   ├── buttons/              # 按钮
│   ├── panels/               # 面板
│   ├── windows/               # 窗口
│   ├── cursors/              # 光标
│   ├── frames/               # 边框
│   ├── icons/                # UI 图标
│   │   ├── skills/
│   │   ├── buffs/
│   │   ├── items/
│   │   ├── equipment/
│   │   ├── classes/
│   │   ├── status/
│   │   └── currencies/
│   ├── portraits/            # UI 用头像
│   └── themes/               # 主题
│
├── shaders/
│   ├── characters/
│   ├── terrain/
│   ├── water/
│   ├── lighting/
│   ├── ui/
│   ├── postprocess/
│   └── vfx/
│
├── fonts/
│   ├── latin/
│   ├── chinese/
│   ├── japanese/
│   └── korean/
│
├── localization/
│   ├── en/
│   ├── zh_cn/
│   ├── zh_tw/
│   ├── ja/
│   └── ko/
│
├── particles/
│   ├── fire/
│   ├── ice/
│   ├── lightning/
│   ├── poison/
│   ├── healing/
│   └── environment/
│
├── cinematics/               # 过场动画资源
│
└── definitions/              # 游戏定义配置（现有）
    ├── attributes.ron
    └── tags.ron
```

### 资产树关键约束

> **宪法依据**：〔宪法 1.1.2 定义与实例分离〕、〔宪法 12.1.4 资源管理〕

1. 🟥 **assets/ 只存放二进制资源** — 美术、音频、字体、Shader、粒子等 〔宪法 1.1.2〕
2. 🟥 **assets/ 禁止存放游戏配置数据** — 配置数据放 `content/` 〔宪法 1.1.3〕
3. 🟩 **按类型组织，不按功能** — `audio/bgm/` 不放 `audio/battle/bgm/` 中，按类型分
4. 🟩 **角色资源按角色目录组织** — 每个 NPC/角色 有独立目录（利于外包）

---

## 三、content/ — 游戏内容数据树

```
content/
├── skills/                   # 技能定义（RON）
│   ├── fireball.ron
│   ├── heal.ron
│   ├── bash.ron
│   └── ...
│
├── buffs/                    # Buff 定义（RON）
│   ├── poison.ron
│   ├── strength_boost.ron
│   └── ...
│
├── effects/                  # 效果定义（RON）
│   ├── direct_damage.ron
│   ├── heal_effect.ron
│   └── ...
│
├── formulas/                 # 公式定义（RON）
│   ├── damage_formula.ron
│   ├── healing_formula.ron
│   └── ...
│
├── classes/                  # 职业定义（RON）
│   ├── knight.ron
│   ├── mage.ron
│   └── ...
│
├── characters/               # 角色模板（RON）
│   ├── hero_knight.ron
│   ├── goblin_warrior.ron
│   └── ...
│
├── enemies/                  # 敌人模板（RON）
│   └── ...
│
├── summons/                  # 召唤物模板（RON）
│   └── ...
│
├── items/                    # 物品定义（RON）
│   └── ...
│
├── equipments/               # 装备定义（RON）
│   └── ...
│
├── quests/                   # 任务定义（RON）
│   └── ...
│
├── dialogues/                # 对话树定义（RON）
│   └── ...
│
├── cutscenes/                # 剧情定义（RON）
│   └── ...
│
├── factions/                 # 阵营定义（RON）
│   └── ...
│
├── loot_tables/              # 掉落表（RON）
│   └── ...
│
├── shops/                    # 商店定义（RON）
│   └── ...
│
├── recipes/                  # 配方定义（RON）
│   └── ...
│
├── campaigns/               # 战役定义（RON）
│   └── ...
│
├── chapters/                 # 章节定义（RON）
│   └── ...
│
├── stages/                  # 关卡定义（RON）
│   └── ...
│
├── achievements/             # 成就定义（RON）
│   └── ...
│
├── tutorials/                # 教程定义（RON）
│   └── ...
│
└── terrains/                 # 地形定义（RON）
    └── ...
```

### 内容树关键约束

> **宪法依据**：〔宪法 1.1.3 规则与内容分离〕、〔宪法 12.1.1 职责划分〕

1. 🟥 **content/ 只存放 RON 配置文件** — 禁止放 Rust 代码 〔宪法 1.1.3〕
2. 🟥 **新增内容 = 新增 RON 文件** — 禁止修改逻辑代码 〔宪法 1.1.3〕
3. 🟩 **Skill 是 Core，Fireball 是 Content** — 规则在 src/core/skill/，数据在 content/skills/ 〔宪法 1.1.3〕
4. 🟩 **content/ 目录结构对齐 src/core/ 模块结构** — 每个 core 模块对应一个 content 子目录

---

## 四、mods/ — MOD 扩展目录

```
mods/
├── official/                 # 官方 MOD 示例
│   └── example_mod/
│       ├── manifest.ron      #   MOD 清单
│       ├── content/          #   MOD 内容
│       └── compatibility/    #   兼容性声明
│
├── community/                # 社区 MOD（gitignore）
│
└── dev/                      # 开发中 MOD
    └── test_mod/
        ├── manifest.ron
        └── content/
```

---

## 五、scripts/ — 开发脚本目录

```
scripts/
├── build/                    # 构建脚本
├── release/                  # 发布脚本
├── packaging/                # 打包脚本
├── validation/                # 校验脚本
├── asset_pipeline/            # 资源管线脚本
├── sprite_pipeline/           # Sprite 管线
├── audio_pipeline/            # 音频管线
├── localization/              # 本地化脚本
├── data_generation/           # 数据生成脚本
├── balance/                   # 数值平衡脚本
├── ci/                        # CI 脚本
└── migration/                 # 数据迁移脚本
```

---

## 六、tests/ — 测试目录

```
tests/
├── unit/                     # 单元测试（各模块内部）
├── integration/              # 集成测试（跨模块）
├── scenario/                 # 场景测试（完整流程）
├── replay/                   # 回放测试（战斗录像）
├── golden/                   # 黄金测试（快照比对）
├── rule/                     # 规则测试（领域不变量）
└── fixtures/                 # 测试固件（共享测试数据）
    ├── units/
    ├── skills/
    ├── buffs/
    └── maps/
```

---

## 七、tools/ — 开发工具目录

```
tools/
├── content_editor/           # 内容编辑器
├── map_editor/               # 地图编辑器
├── dialogue_editor/          # 对话编辑器
├── data_validator/            # 数据校验器
├── balance_checker/          # 数值检查器
├── content_linter/            # 内容规范检查
├── migration_tool/            # 数据迁移工具
├── replay_inspector/         # 回放检查器
├── save_inspector/           # 存档检查器
├── benchmark/                 # 性能基准
├── test_generator/            # 测试生成器
└── asset_pipeline/            # 资源管线工具
```

---

## 八、benchmarks/ — 性能基准目录

```
benchmarks/
├── battle/
├── pathfinding/
├── attribute/
└── memory/
```

---

## 九、docs/ — 文档目录

```
docs/
├── architecture.md           # 架构规范（最高优先级）
├── architecture/              # 架构详细文档
│   ├── project-structure.md
│   ├── layer-contracts.md
│   ├── error-architecture.md
│   ├── content-pipeline.md
│   ├── modding-design.md
│   ├── asset-organization.md
│   ├── collaboration-model.md
│   ├── migration-roadmap.md
│   ├── infrastructure-design.md
│   ├── app-bootstrap.md
│   ├── skill-buff-abstraction.md
│   ├── plugin-design.md
│   ├── i18n_design.md
│   ├── tools_architecture.md
│   ├── content_migration_design.md
│   ├── asset_namespace_design.md
│   └── feature_flag_design.md
├── adr/                       # 架构决策记录
├── domain/                    # 领域规则
├── testing/                   # 测试规范
├── reviews/                   # 代码审查记录
├── refactor/                  # 重构记录
├── planning/                  # 计划文档
├── coding_rules.md            # 编码规范
├── AI开发宪法.md              # AI 开发宪法
└── 其他/                      # 参考材料
```

---

## 三棵树物理分离（再强调）

> **优化来源**: `docs/其他/64.md`
> **宪法依据**：〔宪法 1.1.1 Feature First〕、〔宪法 1.1.2 定义与实例分离〕、〔宪法 1.1.3 规则与内容分离〕

三棵树不仅是逻辑分离，更是**物理分离**——禁止交叉引用、禁止混放。

```
项目根目录/
├── src/                          # 树1：源码（Rust 代码）
│   ├── app/                      #   启动装配
│   ├── core/                     #   游戏规则
│   ├── shared/                   #   基础能力
│   ├── infrastructure/           #   技术实现
│   ├── content/                  #   内容桥接
│   ├── modding/                  #   MOD 支持
│   ├── debug/                    #   调试工具
│   └── ui/                       #   UI 表现
│
├── content/                      # 树2：内容（RON 配置）
│   ├── skills/                   #   技能定义
│   ├── buffs/                    #   Buff 定义
│   ├── characters/               #   角色模板
│   ├── stages/                   #   关卡定义
│   ├── terrains/                 #   地形定义
│   ├── items/                    #   物品定义
│   ├── equipments/               #   装备定义
│   └── ...
│
└── assets/                       # 树3：资产（二进制资源）
    ├── art/                      #   美术
    ├── audio/                    #   音频
    ├── ui/                       #   UI 资源
    ├── shaders/                  #   Shader
    ├── fonts/                    #   字体
    ├── localization/             #   多语言
    └── ...
```

### 禁止交叉规则

> **宪法依据**：〔宪法 1.1.2 定义与实例分离〕、〔宪法 1.1.3 规则与内容分离〕

| 禁止行为 | 理由 | 替代方案 | 宪法条款 |
|---------|------|---------|----------|
| 🟥 `src/` 中引入 `content/` 路径硬编码 | 内容路径变化影响编译 | 通过 `AssetServer` 动态加载 | 1.1.3 |
| 🟥 `content/` 中放入 Rust 代码 | 混淆数据与逻辑 | 代码放 `src/` | 1.1.3 |
| 🟥 `assets/` 中放入 RON 配置 | 混淆资产与配置 | 配置放 `content/` | 1.1.2 |
| 🟥 `mods/` 中直接修改 `src/` | MOD 不应修改游戏源码 | 通过 `modding/api/` 扩展 | 3.0.1 |

---

## RON 内容文件 → Registry 映射路径表

> **优化来源**: `docs/其他/64.md`

| content/ 路径 | RON 文件类型 | 加载模块（src/content/） | 注册目标（src/core/） | 交叉引用 |
|--------------|-------------|------------------------|---------------------|---------|
| `content/skills/*.ron` | `SkillDef` | `content/skills/skill_content.rs` | `SkillRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/buffs/*.ron` | `BuffDef` | `content/buffs/buff_content.rs` | `BuffRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/characters/*.ron` | `UnitTemplate` | `content/characters/character_content.rs` | `UnitRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/stages/*.ron` | `StageDef` | `content/stages/stage_content.rs` | `StageRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/terrains/*.ron` | `TerrainDef` | `content/terrains/terrain_content.rs` | `TerrainRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/items/*.ron` | `ItemDef` | `content/items/item_content.rs` | `ItemRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/equipments/*.ron` | `EquipmentDef` | `content/equipments/equipment_content.rs` | `EquipmentRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/classes/*.ron` | `ClassDef` | `content/classes/class_content.rs` | `ClassRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |
| `content/ai_behaviors/*.ron` | `AiBehaviorDef` | `content/ai_behaviors/ai_behavior_content.rs` | `AiBehaviorRegistry` | `docs/01-architecture/03-data-config-asset/content-pipeline.md` |

### 规则

> **宪法依据**：〔宪法 12.1.1 职责划分〕配置定义内容，代码解释配置

- 🟥 **每种 RON 文件必须有明确的加载模块和注册目标** — 禁止无 Registry 的 RON 〔宪法 12.2.1〕
- 🟩 **加载模块遵循 `加载 → 校验 → 注册` 三步模式** — 详见 `docs/01-architecture/03-data-config-asset/content-pipeline.md` 〔宪法 12.1.1〕
- 🟩 **RON 文件路径与 Registry 名称保持一致** — `content/skills/` → `SkillRegistry`

---

## MOD 沙箱标准布局

> **优化来源**: `docs/其他/64.md`

MOD 目录遵循标准布局，确保安全隔离：

```
mods/
├── official/                     # 官方 MOD 示例
│   └── example_mod/
│       ├── manifest.ron          #   MOD 清单（名称、版本、依赖）
│       ├── content/              #   MOD 内容（RON 配置）
│       │   ├── skills/           #     新增技能
│       │   ├── buffs/            #     新增 Buff
│       │   ├── characters/       #     新增角色
│       │   └── ...
│       ├── localization/         #   MOD 本地化
│       │   ├── en/               #     英文
│       │   └── zh_cn/            #     中文
│       └── assets/               #   MOD 资产（可选）
│           ├── art/              #     自定义美术
│           └── audio/            #     自定义音频
│
├── community/                    # 社区 MOD（gitignore）
│   └── <mod_name>/
│       ├── manifest.ron
│       ├── content/
│       ├── localization/
│       └── assets/
│
└── dev/                          # 开发中 MOD
    └── test_mod/
        ├── manifest.ron
        ├── content/
        └── localization/
```

### manifest.ron 标准格式

```rust
pub struct ModManifest {
    pub name: String,              // MOD 名称
    pub version: String,           // 语义化版本
    pub author: String,            // 作者
    pub description: String,       // 描述
    pub dependencies: Vec<String>, // 依赖的其他 MOD
    pub compatibility: ModCompat,  // 兼容性声明
    pub capabilities: Vec<ModCapability>, // 声明的能力
}

pub enum ModCapability {
    AddSkills,       // 可以添加新技能
    AddBuffs,        // 可以添加新 Buff
    AddCharacters,   // 可以添加新角色
    ModifyBalance,   // 可以修改数值平衡
    AddMaps,         // 可以添加新地图
}
```

### 规则

> **宪法依据**：〔宪法 17.2.1 Mod支持预留〕核心系统设计时预留轻量扩展点

- 🟥 **MOD 内容只能放在 `mods/<name>/content/`** — 禁止修改项目根 `content/` 〔宪法 1.1.3〕
- 🟥 **MOD 资产只能放在 `mods/<name>/assets/`** — 禁止修改项目根 `assets/`
- 🟩 **MOD 必须声明 `manifest.ron`** — 包含版本、依赖、能力声明 〔宪法 12.6.1〕
- 🟩 **MOD 沙箱必须限制执行权限** — 详见 `src/modding/sandbox/sandbox_runner.rs`

---

## 层级模块索引表

> **优化来源**: `docs/其他/64.md`

快速跳转：想找 X → 看 Y → 调用 Z

| 我想找... | 看这个文件/目录 | 调用这个 API/类型 |
|-----------|---------------|------------------|
| 战斗伤害计算 | `src/core/battle/combat.rs` | `calculate_damage()` |
| 技能定义 | `content/skills/*.ron` | `SkillDef` → `SkillRegistry` |
| Buff 施加逻辑 | `src/core/buff/buff_apply.rs` | `apply_buff()` |
| 回合状态机 | `src/core/turn/turn_phase.rs` | `TurnPhase` |
| 角色属性 | `src/core/character/unit.rs` | `Unit` Component |
| 装备穿脱 | `src/core/equipment/equip.rs` | `equip_item()` / `unequip_item()` |
| 寻路算法 | `src/core/map/pathfinding/bfs.rs` | `bfs_pathfinding()` |
| 存档保存/加载 | `src/infrastructure/persistence/` | `SavePlugin` / `LoadPlugin` |
| 内容加载 | `src/content/content_plugin.rs` | `ContentPlugin` |
| UI 面板 | `src/ui/panels/` | `UiPlugin` |
| 调试面板 | `src/debug/panels/` | `DebugPlugin` |
| MOD API | `src/modding/api/mod_api.rs` | `ModApi` |
| 错误类型 | `src/core/xxx/domain/xxx_error.rs` | `XxxError` |
| 共享 ID | `src/shared/ids/` | `UnitId`, `SkillId`, ... |
| 测试工具 | `src/shared/testing/` | `spawn_test_battle()` |
| 游戏常量 | `src/shared/constants/` | `MAX_LEVEL`, ... |
| 校验工具 | `src/shared/validation/` | `validate_*()` |

### 规则

> **宪法依据**：〔宪法 3.0.1 接口最小化原则〕、〔宪法 3.0.4 跨模块交互规范〕

- 🟩 **每个模块必须有明确的入口文件** — `plugin.rs` 或 `mod.rs` 〔宪法 3.0.2〕
- 🟩 **模块间通过 Message 通信** — 禁止直接 `use` 其他模块内部类型 〔宪法 3.0.4〕
- 🟩 **新增功能先查索引表** — 避免重复造轮子

---

## 迁移映射：当前 → 目标

| 当前位置 | 目标位置 | 说明 |
|----------|---------|------|
| `src/core/id/` | `src/shared/ids/` | 强类型 ID 是通用基础能力 |
| `src/core/error/` | 各 `core/xxx/domain/` + `src/shared/error/` | 领域错误回归领域，共享工具归 shared |
| `src/infrastructure/audit/` | `src/infrastructure/audit/` + `src/shared/audit/` | 审计基础设施在 infra，白名单在 shared |
| `src/battle/` | `src/core/battle/` | 战斗是核心游戏规则 |
| `src/buff/` | `src/core/buff/` | Buff 是核心游戏规则 |
| `src/skill/` | `src/core/skill/` | 技能是核心游戏规则 |
| `src/character/` | `src/core/character/` | 角色是核心游戏规则 |
| `src/equipment/` | `src/core/equipment/` | 装备是核心游戏规则 |
| `src/inventory/` | `src/core/inventory/` | 背包是核心游戏规则 |
| `src/ai/` | `src/core/ai/` | AI 行为是核心游戏规则 |
| `src/map/` | `src/core/map/` | 地图是核心游戏规则 |
| `src/turn/` | `src/core/turn/` | 回合是核心游戏规则 |
| `src/campaign/` | `src/core/campaign/` | 战役是核心游戏规则 |
| `src/ui/` | `src/ui/` (保持顶层) | UI 是表现层，不入 core |
| `src/debug/` | `src/debug/` (保持顶层) | 调试工具不入 core |
| `src/input.rs` | `src/ui/input.rs` | 输入归 UI 层 |
| `src/assets.rs` | `src/infrastructure/assets/` | 资源管理是基础设施 |
| `assets/units/*.ron` | `content/characters/*.ron` | 游戏数据分离到 content/ |
| `assets/skills/*.ron` | `content/skills/*.ron` | 游戏数据分离到 content/ |
| `assets/buffs/*.ron` | `content/buffs/*.ron` | 游戏数据分离到 content/ |
| `assets/ai/*.ron` | `content/ai_behaviors/*.ron` | AI 数据分离到 content/ |
| `assets/maps/*.ron` | `content/stages/*.ron` | 关卡数据分离到 content/ |
| `assets/terrains/*.ron` | `content/terrains/*.ron` | 地形数据分离到 content/ |
| `assets/traits/*.ron` | `content/classes/*.ron` (部分) | 特质数据分离到 content/ |

---

## 附录：各目录职责速查

| 目录 | 职责 | 变化频率 | 外包友好 |
|------|------|---------|---------|
| `core/` | 游戏规则 | 低 | 否（需程序员） |
| `shared/` | 基础能力 | 低 | 否 |
| `infrastructure/` | 技术实现 | 中 | 部分可外包 |
| `app/` | 游戏装配 | 低 | 否 |
| `content/` (src) | 内容桥接 | 中 | 否 |
| `content/` (项目根) | 游戏数据 | 高 | 是（策划） |
| `assets/` | 美术音频 | 高 | 是（美术+音频） |
| `mods/` | MOD 扩展 | 高 | 是（MOD 作者） |
| `tools/` | 开发工具 | 中 | 部分 |
| `scripts/` | 自动化 | 低 | 否 |
| `tests/` | 测试 | 高 | 否 |