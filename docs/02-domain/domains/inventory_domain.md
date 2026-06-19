---
id: 02-domain.inventory
title: Inventory（背包/物品）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - inventory
  - business-domain
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Inventory | 背包/库存，管理角色持有的所有物品集合 | 负责：物品的增删查改与堆叠合并，Inventory 的 LocalizationKey（name_key/desc_key）；不负责：物品的使用逻辑 |
| EquipmentSlots | 装备槽位，定义角色身上各部位的装备位 | 负责：槽位规则（装备位数量/类型/兼容性）；不负责：装备的属性修改 |
| ItemInstance | 物品实例，物品在背包中的具体存在（含附魔/耐久/自定义属性） | 负责：物品实例数据的存储；不负责：物品的模板定义 |
| Equipment | 可穿戴的装备物品，穿戴时提供 Modifier 或特殊能力 | 负责：装备定义与穿戴规则；不负责：装备对属性的影响（归 Modifier 管线） |
| Consumable | 消耗品，使用时消耗并产生一次性效果 | 负责：消耗品的定义与消耗规则；不负责：消耗效果的执行（归 Effect 领域） |
| LootTable | 战利品表，定义掉落物品的概率和数量 | 负责：掉落规则的概率模型；不负责：掉落物品的生成 |

### 装备槽位体系

```
EquipmentSlots（装备槽位，基于 D&D 5e/BG3）
 ├── 主手（Main Hand）：武器/盾牌/法器
 ├── 副手（Off Hand）：武器/盾牌/法器（双手武器同时占主手+副手）
 ├── 头盔（Helmet）：头盔/头饰/帽子
 ├── 铠甲（Armor）：胸甲/皮甲/布甲
 ├── 手套（Gloves）：手套/护腕
 ├── 靴子（Boots）：靴子/护胫
 ├── 披风（Cloak）：披风/斗篷
 ├── 戒指1（Ring 1）：戒指（左侧）
 ├── 戒指2（Ring 2）：戒指（右侧，限两个戒指）
 ├── 项链（Amulet）：项链/护符
 └── 特殊槽位（Special）：任务物品/特殊装备

 槽位规则：
 - 双手武器占用 MainHand + OffHand 两个槽位
 - 每个槽位最多一件装备
 - 同类型覆盖（新装备替换旧装备）
```

### 物品稀有度

```
物品稀有度（Rarity）
 ├── 普通（Common）：基础物品，无特殊效果
 ├── 非凡（Uncommon）：+1 装备，基础附魔
 ├── 稀有（Rare）：+2 装备，强力附魔
 ├── 史诗（Very Rare）：+3 装备，独特能力
 └── 传说（Legendary）：+3 装备，特殊剧情能力/唯一装备
```

### 已对齐项目术语

- **Attribute**：装备穿戴后通过 Modifier 影响角色的属性值
- **Modifier**：Equipment 携带的 Modifier 列表，穿戴时注册到 ModifierContainer
- **Condition**：装备穿戴前检查条件（等级需求/属性需求/职业需求/阵营需求）
- **Effect**：Consumable 使用后产生 Effect
- **Crafting**：Crafting 领域产出的装备进入 Inventory
- **Economy**：Economy 领域的商店买卖消耗/产生 Inventory 中的物品

---

## 2. 背包状态机

### 装备状态

```
InInventory（在背包中）
   │  [穿戴上对应的 EquipmentSlot]
   ▼
Equipped（已穿戴/生效中）
   │  [Modifier 已注册到角色]
   │  [卸下/替换]
   ▼
Unequipped（卸下——回到背包）
```

### 物品使用状态

```
InInventory（在背包中）
   │  [使用（消耗品）/ 装备（装备）]
   │
   ├──→ [消耗品]：使用后 → 生成 Effect → 消耗数量 → 数量=0时移除
   │
   └──→ [装备]：穿到对应槽位
```

---

## 3. 不变量（Invariants）

### 3.1 槽位独占性
- **条件**：任何装备穿戴时
- **不变量**：每个 EquipmentSlot 同一时间只能穿戴一件装备
- **违反后果类型**：🔴 规则失败
- **违反后果**：多个装备挤在同一个槽位，系统应拒绝穿戴

### 3.2 双手武器占双槽
- **条件**：装备双手武器时
- **不变量**：双手武器同时占用 MainHand + OffHand，副手槽位在装备期间不可用
- **违反后果类型**：🔴 规则失败
- **违反后果**：双手武器+盾牌同时装备，违反 D&D 5e 规则

### 3.3 装备条件前置检查
- **条件**：任何装备穿戴前
- **不变量**：必须检查穿戴条件（等级/属性/职业/阵营）——使用 Condition 领域
- **违反后果类型**：🔴 规则失败
- **违反后果**：不满足条件的装备被穿戴，角色属性异常

### 3.4 物品堆叠上限
- **条件**：物品入背包时
- **不变量**：可堆叠物品的单格数量不得超过最大堆叠数（默认 99）
- **违反后果类型**：🔴 规则失败
- **违反后果**：堆叠数超限导致物品丢失

### 3.5 装备重量负重限制
- **条件**：角色获得/装备物品时
- **不变量**：角色携带的总重量不得超过最大负重（力量 × 15 磅）
- **违反后果类型**：🔴 规则失败
- **违反后果**：超重导致移动速度降低等负面效果

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：装备直接修改属性值 — 理由：装备通过 Modifier 间接影响属性，不走 bypass
- 🟥 禁止：同一角色同时装备同名唯一装备（Legendary 唯一装备） — 理由：唯一装备只能存在一件
- 🟥 禁止：消耗品使用后效果未生效但物品已被消耗 — 理由：效果必须确认生效后才消耗物品
- 🟥 禁止：从背包移除已穿戴装备时不同步卸下 — 理由：移除背包中的装备应自动执行卸下流程
- 🟥 禁止：InventoryDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 物品获得（入背包）

- **输入**：角色、ItemTemplate/ItemInstance、数量
- **处理**：
  1. 检查是否已有同类可堆叠物品：
     - 如果已有且堆叠未满 → 增加到已有堆叠
     - 如果已满或不同类 → 创建新槽位
  2. 检查背包是否有空位（或扩展背包）
  3. 检查总重量是否超过负重上限（不变量 3.5）
  4. 添加物品到背包
  5. 发布 ItemAcquired 事件
- **输出**：ItemAcquired 事件
- **失败处理**：背包满/超重时物品无法获得 → 这是**规则失败**（预期业务分支，玩家应清理背包或扩容后再试）
- **程序错误**：ItemTemplate/ItemInstance 在数据层不存在时 → 这是**程序错误**（系统异常，应记 Bug）

### 5.2 装备穿戴

- **输入**：角色、背包中的装备物品
- **处理**：
  1. 检查装备条件（Condition 领域——等级/属性/职业/阵营）
  2. 检查槽位占用：
     - 如果是双手武器，检查主手+副手均空闲
     - 如果目标槽位已有装备，先卸下旧装备
  3. 从背包移除装备
  4. 注册到 EquipmentSlot
  5. 注册装备的 Modifier 到 ModifierContainer
  6. 发布 EquipmentChanged 事件
- **输出**：EquipmentChanged 事件
- **失败处理**：条件不满足/槽位被占（非替换）时穿戴失败 → 这是**规则失败**（预期业务分支，玩家需满足条件或更换装备）
- **程序错误**：背包中找不到指定装备物品时 → 这是**程序错误**（系统异常，应记 Bug）

### 5.3 装备卸下

- **输入**：角色、已装备的槽位
- **处理**：
  1. 从 EquipmentSlot 卸下装备
  2. 从 ModifierContainer 移除装备的 Modifier
  3. 将物品放回背包
  4. 检查总重量
  5. 发布 EquipmentChanged 事件
- **输出**：EquipmentChanged 事件
- **失败处理**：背包满时无法卸下（需先腾出空间） → 这是**规则失败**（预期业务分支，玩家应先清理背包）
- **程序错误**：EquipmentSlot 中找不到已装备物品记录时 → 这是**程序错误**（系统异常，应记 Bug）

### 5.4 物品使用（消耗品）

- **输入**：角色、背包中的消耗品、使用目标/位置
- **处理**：
  1. 检查使用条件（Condition——生命不满才能使用治疗药水）
  2. 如果条件满足，执行使用方法（通常为生成 Effect）
  3. 效果确认生效后，消耗 1 个物品数量
  4. 如果数量归零，从背包移除物品
  5. 发布 ItemUsed 事件
- **输出**：ItemUsed 事件
- **失败处理**：条件不满足时使用失败，物品不消耗 → 这是**规则失败**（预期业务分支，玩家需满足使用条件）
- **程序错误**：背包中找不到指定消耗品时 → 这是**程序错误**（系统异常，应记 Bug）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ItemAcquired | 物品进入背包时 | entity_id, item_instance_id, item_template_id, quantity, source | UI（更新背包显示）、Quest（检查物品收集任务）、日志（LogCode: INV001） |
| ItemUsed | 消耗品使用完成时 | entity_id, item_instance_id, quantity_consumed, remaining, effect_result | UI（更新背包数量）、Effect（执行消耗品效果）、日志（LogCode: INV002） |
| EquipmentChanged | 装备穿戴/卸下时 | entity_id, slot, old_item（可为空）, new_item（可为空） | Modifier（注册/移除装备 Modifier）、Attribute（触发属性重算）、UI（更新角色装备显示）、日志（LogCode: INV003） |
| ItemRemoved | 物品从背包移除时 | entity_id, item_instance_id, reason（使用/丢弃/交易/摧毁） | UI（更新背包显示）、日志（LogCode: INV004） |
| LootGenerated | 战利品生成时 | source（击杀/开箱等）, loot_table_id, items_generated[ ] | Inventory（添加物品到背包）、UI（显示战利品界面）、日志（LogCode: INV005） |

### 事件订阅关系图

```
ItemAcquired
    │
    ├──→ UI：背包界面更新
    ├──→ Quest：检查物品收集任务进度
    └──→ 日志：记录物品获得来源

EquipmentChanged
    │
    ├──→ Modifier：注册/移除装备 Modifier
    ├──→ Aggregator：触发属性重算（装备影响属性）
    ├──→ UI：更新角色装备预览
    ├──→ Condition：重新检查装备相关条件
    └──→ Cue：装备特效

ItemUsed
    │
    ├──→ Effect：执行消耗品效果（如治疗药水→HealEffect）
    ├──→ UI：更新背包显示/使用动画
    └──→ 日志：记录物品使用
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Inventory 域位于 `core/domains/inventory/`，components.rs 定义 Inventory/EquipmentSlots/ItemInstance，systems/ 实现背包/装备/使用/掉落系统，rules/ 定义槽位/负重/稀有度规则
- ✅ 装备通过 Modifier 管线：不 bypass 直接修改属性
- ✅ 条件检查复用 Condition 领域：穿戴条件统一走条件检查
- ✅ 消耗品效果通过 Effect 领域：不另建使用效果系统
- ✅ 物品来源多样性：Loot/Quest/Crafting/Economy 均可产生物品
- ✅ LocalizationKey：本领域涉及的用户可见文本使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖物品获得、装备穿戴/卸下、消耗品使用、战利品等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 装备槽位体系 / 物品稀有度 / 堆叠规则 / 负重规则明确
- [x] 每个操作有完整的流程定义（获得、穿戴、卸下、使用）
