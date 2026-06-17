
```bash
D:\Code\Bevy\Fre\

├── Cargo.toml                    # Rust 项目配置，依赖声明（Bevy 0.18.1, bevy-inspector-egui, serde, ron）
├── Cargo.lock                    # 依赖版本锁定文件
├── .gitignore                    # Git 忽略规则
│
├── assets/                       # 游戏资源目录（数据驱动配置）
│   ├── ai/                       # AI 行为配置（RON 格式）
│   │   ├── aggressive.ron        #   激进型 AI 行为定义
│   │   ├── cautious.ron          #   谨慎型 AI 行为定义
│   │   └── default.ron           #   默认 AI 行为定义
│   ├── buffs/                    # Buff 定义（RON 格式）
│   │   ├── attack_down.ron       #   攻击力降低
│   │   ├── attack_up.ron         #   攻击力提升
│   │   ├── burn.ron              #   燃烧（DoT）
│   │   ├── defense_down.ron      #   防御力降低
│   │   ├── defense_up.ron        #   防御力提升
│   │   ├── poison.ron            #   中毒（DoT）
│   │   ├── regen.ron             #   回复（HoT）
│   │   └── stun.ron              #   眩晕
│   ├── config/                   # 全局配置（预留）
│   ├── definitions/              # 核心定义配置
│   │   ├── attributes.ron        #   属性类型定义（Primary/Derived Stat）
│   │   └── tags.ron              #   GameplayTag 枚举定义
│   ├── fonts/                    # 字体资源
│   │   └── Arial Unicode.ttf     #   支持 CJK 的字体
│   ├── maps/                     # 地图配置
│   │   └── tutorial.ron          #   教程关卡地图
│   ├── rules/                    # 游戏规则配置
│   │   └── element_interactions.ron  # 元素交互规则
│   ├── skills/                   # 技能定义（RON 格式）
│   │   ├── basic_attack.ron      #   普通攻击
│   │   ├── charge.ron            #   冲锋
│   │   ├── cleanse_skill.ron     #   净化
│   │   ├── fireball.ron          #   火球术
│   │   ├── heal.ron              #   治疗
│   │   └── pierce.ron            #   穿刺
│   ├── terrains/                 # 地形定义（RON 格式）
│   │   ├── forest.ron            #   森林（移动消耗 2）
│   │   ├── mountain.ron          #   山地（移动消耗 3）
│   │   ├── plain.ron             #   平原（移动消耗 1）
│   │   └── water.ron             #   水域（需游泳/飞行）
│   ├── traits/                   # Trait 定义（RON 格式）
│   │   ├── archer_mastery.ron    #   弓箭手精通
│   │   ├── fire_affinity.ron     #   火焰亲和
│   │   ├── heavy_armor.ron       #   重甲精通
│   │   ├── mage_mastery.ron      #   法师精通
│   │   └── warrior_mastery.ron   #   战士精通
│   └── units/                    # 单位模板定义（RON 格式）
│       ├── enemy_dark_knight.ron #   敌方：暗黑骑士
│       ├── enemy_goblin.ron      #   敌方：哥布林
│       ├── player_archer.ron     #   我方：弓箭手
│       ├── player_mage.ron       #   我方：法师
│       └── player_warrior.ron    #   我方：战士
│
├── src/                          # 源代码目录（按业务 Feature 划分）
│   ├── main.rs                   # 程序入口，插件注册顺序：核心层→数据层→逻辑层→表现层
│   ├── lib.rs                    # 库入口，导出所有模块
│   ├── assets.rs                 # 资源加载插件（AssetPlugin）
│   ├── input.rs                  # 输入处理插件（InputPlugin）
│   │
│   ├── core/                     # 核心层：属性系统、标签系统、效果管线、修饰规则（无外部依赖）
│   │   ├── mod.rs                #   模块导出
│   │   ├── attribute_def.rs      #   属性类型定义（AttributeKind, Primary/Derived Stat）
│   │   ├── attribute/            #   属性系统
│   │   │   ├── mod.rs            #     模块导出
│   │   │   └── types.rs          #     Attributes, ModifierSource, ModifierStack
│   │   ├── effect/               #   效果管线（Generate→Modify→Execute）
│   │   │   ├── mod.rs            #     模块导出
│   │   │   ├── types.rs          #     EffectDef, PendingEffect
│   │   │   └── handler.rs        #     EffectHandler trait, EffectHandlerRegistry
│   │   ├── modifier_rule.rs      #   修饰规则（标签匹配 + Calculator trait 分发）
│   │   ├── tag.rs                #   GameplayTag 位掩码实现
│   │   ├── tag_def.rs            #   标签类型定义（TagName 枚举）
│   │   ├── registry_loader.rs    #   注册表加载器 trait（RON 文件加载）
│   │   └── snapshot.rs           #   场景快照（World 状态保存/恢复）
│   │
│   ├── character/                # 角色模块：单位生成、组件、Trait 扩展、移动动画
│   │   ├── mod.rs                #   模块导出
│   │   ├── components.rs         #   Unit, Faction, GridPosition, UnitName, UnitId 组件
│   │   ├── marker.rs             #   Tag Component（Dead, Selected, Acted 等）
│   │   ├── movement.rs           #   移动动画系统
│   │   ├── plugin.rs             #   CharacterPlugin 注册
│   │   ├── spawn.rs              #   单位生成（从 UnitTemplate spawn）
│   │   ├── template.rs           #   UnitTemplate 定义（Race + Job + Stats）
│   │   └── traits/               #   Trait 扩展体系
│   │       ├── mod.rs            #     模块导出
│   │       ├── types.rs          #     TraitCollection, TraitTrigger, TraitEffect
│   │       └── handlers.rs       #     TraitEffectHandler trait 分发
│   │
│   ├── battle/                   # 战斗模块：效果管线、伤害计算、战斗记录
│   │   ├── mod.rs                #   模块导出
│   │   ├── combat.rs             #   战斗逻辑（CombatIntent 处理）
│   │   ├── events.rs             #   战斗事件（EntityEvent 机制）
│   │   ├── log.rs                #   战斗日志
│   │   ├── plugin.rs             #   BattlePlugin 注册
│   │   ├── record.rs             #   BattleRecord, DamageBreakdown（可观测性）
│   │   └── pipeline/             #   效果管线（三步：Generate→Modify→Execute）
│   │       ├── mod.rs            #     管线调度
│   │       ├── generate.rs       #     效果生成（CombatIntent → PendingEffect）
│   │       ├── modify.rs         #     效果修饰（ModifierRule 标签匹配）
│   │       ├── execute.rs        #     效果执行（EffectHandler 分发）
│   │       ├── intent.rs         #     CombatIntent 组件定义
│   │       └── trait_trigger.rs  #     Trait 触发时机处理
│   │
│   ├── buff/                     # Buff 模块：定义、实例、穿戴、结算
│   │   ├── mod.rs                #   模块导出
│   │   ├── domain.rs             #   BuffDef, BuffData, BuffRegistry（定义层）
│   │   ├── instance.rs           #   BuffInstance, ActiveBuffs（实例层）
│   │   ├── apply.rs              #   Buff 穿戴/移除逻辑
│   │   ├── resolve.rs            #   Buff 持续效果结算（DoT/HoT/Stun/tick）
│   │   └── plugin.rs             #   BuffPlugin 注册
│   │
│   ├── skill/                    # 技能模块：定义、槽位、冷却、预览
│   │   ├── mod.rs                #   模块导出
│   │   ├── domain/               #   技能定义层
│   │   │   ├── mod.rs            #     模块导出
│   │   │   ├── types.rs          #     SkillDef, SkillData, SkillRegistry
│   │   │   └── defaults.rs       #     默认技能配置
│   │   ├── slots.rs              #   SkillSlots, SkillCooldowns（运行时）
│   │   ├── preview.rs            #   技能效果预览
│   │   └── plugin.rs             #   SkillPlugin 注册
│   │
│   ├── equipment/                # 装备模块：定义、实例、槽位、穿脱、需求检查
│   │   ├── mod.rs                #   模块导出
│   │   ├── definition.rs         #   EquipmentDef, EquipmentSlot, Rarity, EquipmentRegistry
│   │   ├── instance.rs           #   EquipmentInstance, Inventory（背包组件）
│   │   ├── slots.rs              #   EquipmentSlots（槽位容器）
│   │   ├── equip.rs              #   穿脱逻辑（EquipItem/UnequipItem Message）
│   │   ├── requirements.rs       #   需求检查（RequireTag, AttributeMin）
│   │   └── plugin.rs             #   EquipmentPlugin 注册
│   │
│   ├── inventory/                # 背包模块：物品定义、容器、转移、使用
│   │   ├── mod.rs                #   模块导出
│   │   ├── definition.rs         #   ItemDef, ItemRegistry（物品定义）
│   │   ├── instance.rs           #   ItemInstance, ItemStack（物品实例）
│   │   ├── container.rs          #   Container（容器：Slot + Stack + Weight）
│   │   ├── battle_bag.rs         #   BattleBag（战斗背包）
│   │   ├── transfer.rs           #   物品转移逻辑
│   │   ├── use_item.rs           #   物品使用逻辑
│   │   └── resources.rs          #   背包相关 Resource
│   │
│   ├── map/                      # 地图模块：地形、占用、寻路、坐标转换
│   │   ├── mod.rs                #   模块导出
│   │   ├── data.rs               #   TerrainDef, TerrainRegistry（地形定义）
│   │   ├── grid.rs               #   GameMap（坐标转换）
│   │   ├── plugin.rs             #   MapPlugin 注册
│   │   ├── runtime/              #   运行时数据
│   │   │   ├── mod.rs            #     模块导出
│   │   │   ├── terrain_grid.rs   #     TerrainGrid（地形唯一真相源）
│   │   │   └── occupancy_grid.rs #     OccupancyGrid（单位占用独立存在）
│   │   └── pathfinding/          #   寻路系统
│   │       ├── mod.rs            #     模块导出
│   │       ├── algorithms.rs     #     BFS 寻路算法
│   │       └── cost.rs           #     TerrainCostCalculator trait（地形消耗）
│   │
│   ├── turn/                     # 回合模块：状态机、行动队列、回合消息
│   │   ├── mod.rs                #   模块导出
│   │   ├── state.rs              #   AppState, TurnPhase（状态层次）
│   │   └── order.rs              #   TurnOrder（Initiative 降序行动队列）
│   │
│   ├── ai/                       # AI 模块：行为定义、策略选择、决策系统
│   │   ├── mod.rs                #   模块导出
│   │   ├── behavior.rs           #   AiBehavior, AiBehaviorRegistry（行为定义）
│   │   ├── strategy.rs           #   TargetSelector, MoveSelector, SkillSelector trait
│   │   ├── targeting.rs          #   目标选择策略实现
│   │   ├── movement.rs           #   移动选择策略实现
│   │   ├── skill_select.rs       #   技能选择策略实现
│   │   ├── decision.rs           #   AI 决策系统
│   │   └── plugin.rs             #   AiPlugin 注册
│   │
│   ├── ui/                       # UI 模块：ViewModel、UiCommand、面板组件
│   │   ├── mod.rs                #   模块导出
│   │   ├── plugin.rs             #   UiPlugin 注册
│   │   ├── command_handler.rs    #   UiCommand 分发器
│   │   ├── view_models.rs        #   ViewModel 层（只读状态）
│   │   ├── events.rs             #   UI 事件定义
│   │   ├── theme.rs              #   UiTheme（统一样式）
│   │   ├── focus.rs              #   输入焦点管理
│   │   ├── camera.rs             #   相机控制
│   │   ├── action_menu.rs        #   行动菜单面板
│   │   ├── combat_preview.rs     #   战斗预览面板
│   │   ├── combat_log_handler.rs #   战斗日志处理器
│   │   ├── combat_vfx_handler.rs #   战斗特效处理器
│   │   ├── highlight.rs          #   高亮显示
│   │   ├── tile_info.rs          #   地块信息显示
│   │   ├── vfx.rs                #   视觉特效
│   │   ├── settings.rs           #   设置面板
│   │   ├── panels/               #   面板组件
│   │   │   ├── mod.rs            #     模块导出
│   │   │   ├── action_hint.rs    #     行动提示
│   │   │   ├── combat_log_panel.rs  # 战斗日志面板
│   │   │   ├── inventory_panel.rs   # 背包面板
│   │   │   ├── turn_indicator.rs    # 回合指示器
│   │   │   └── unit_info.rs         # 单位信息面板
│   │   └── widgets/              #   通用 Widget
│   │       ├── mod.rs            #     模块导出
│   │       ├── layout.rs         #     布局组件
│   │       ├── popup.rs          #     弹窗组件
│   │       └── resource_bar.rs   #     资源条组件（HP/MP）
│   │
│   └── debug/                    # 调试模块：调试面板、可观测性
│       ├── mod.rs                #   模块导出
│       ├── gizmos_viz.rs         #   Gizmos 可视化
│       ├── overlay.rs            #   调试叠加层
│       ├── stepping_control.rs   #   单步调试控制
│       └── viewers/              #   调试查看器
│           ├── mod.rs            #     模块导出
│           ├── ai_viewer.rs      #     AI 行为查看器
│           ├── attribute_viewer.rs  # 属性查看器
│           ├── battle_debugger.rs   # 战斗调试器
│           ├── buff_viewer.rs       # Buff 查看器
│           ├── damage_viewer.rs     # 伤害查看器
│           ├── equipment_viewer.rs  # 装备查看器
│           ├── grid_viewer.rs       # 网格查看器
│           ├── settings_viewer.rs   # 设置查看器
│           └── turn_queue_viewer.rs # 回合队列查看器
│
├── tests/                        # 测试目录（按测试类型划分）
│   ├── common/                   #   公共测试工具
│   ├── rule/                     #   规则测试（验证核心规则：伤害、Buff、属性、寻路）
│   ├── feature/                  #   功能测试（验证完整 Feature：装备、背包、战斗）
│   ├── scenario/                 #   场景测试（验证玩家流程：回合、技能、胜负）
│   ├── golden/                   #   黄金测试（快照对比）
│   ├── system/                   #   系统测试（ECS 系统集成测试）
│   ├── legacy/                   #   历史测试（遗留测试用例）
│   ├── feature.rs                #   功能测试入口
│   ├── golden.rs                 #   黄金测试入口
│   ├── rule.rs                   #   规则测试入口
│   ├── scenario.rs               #   场景测试入口
│   ├── system.rs                 #   系统测试入口
│   ├── legacy_buff.rs            #   历史 Buff 测试
│   ├── legacy_combat.rs          #   历史战斗测试
│   ├── legacy_edge.rs            #   历史边界测试
│   └── legacy_turn.rs            #   历史回合测试
│
├── docs/                         # 文档目录
│   ├── architecture.md           #   架构文档 v3.0（最高优先级）
│   ├── AI开发宪法.md              #   AI 开发宪法 v1.1（约束 AI 生成代码）
│   ├── architecture_changelog_v3.md  # 架构变更日志
│   ├── test_spec.md              #   测试规范
│   ├── 1.md                      #   临时文档
│   ├── domain/                   #   领域规则文档
│   ├── rules/                    #   业务规则文档
│   ├── testing/                  #   测试相关文档
│   ├── refactor/                 #   重构文档
│   ├── reviews/                  #   代码评审文档
│   ├── adr/                      #   架构决策记录
│   └── 其他/                      #   其他文档
│
├── .omo/                         # OpenCode 配置目录
│   └── run-continuation/         #   运行 continuation 配置
│
└── target/                       # Rust 编译输出目录（自动生成）
```
