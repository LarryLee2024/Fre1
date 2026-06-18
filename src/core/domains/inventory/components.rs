//! ECS Components — 背包/物品领域组件
//!
//! 定义背包容器、装备槽位、物品实例等 ECS 组件。
//! 详见 docs/02-domain/domains/inventory_domain.md
//! 详见 docs/04-data/domains/inventory_schema.md

use bevy::prelude::*;

// ─── 枚举类型 ─────────────────────────────────────────────────────

/// 物品稀有度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum Rarity {
    /// 普通 — 基础物品，无特殊效果
    #[default]
    Common,
    /// 非凡 — +1 装备，基础附魔
    Uncommon,
    /// 稀有 — +2 装备，强力附魔
    Rare,
    /// 史诗 — +3 装备，独特能力
    VeryRare,
    /// 传说 — +3 装备，特殊剧情能力/唯一装备
    Legendary,
}

/// 装备槽位类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum EquipSlot {
    /// 主手 — 武器/盾牌/法器
    MainHand,
    /// 副手 — 武器/盾牌/法器（双手武器同时占主手+副手）
    OffHand,
    /// 头盔/头饰/帽子
    Helmet,
    /// 铠甲/胸甲/皮甲/布甲
    Armor,
    /// 手套/护腕
    Gloves,
    /// 靴子/护胫
    Boots,
    /// 披风/斗篷
    Cloak,
    /// 戒指（左侧）
    Ring1,
    /// 戒指（右侧）
    Ring2,
    /// 项链/护符
    Amulet,
    /// 特殊槽位 — 任务物品/特殊装备
    Special,
}

impl EquipSlot {
    /// 所有槽位的迭代器。
    pub fn all() -> impl Iterator<Item = EquipSlot> {
        use EquipSlot::*;
        [
            MainHand, OffHand, Helmet, Armor, Gloves, Boots, Cloak, Ring1, Ring2, Amulet, Special,
        ]
        .into_iter()
    }

    /// 槽位的中文名称（调试用）。
    pub fn name(&self) -> &'static str {
        use EquipSlot::*;
        match self {
            MainHand => "主手",
            OffHand => "副手",
            Helmet => "头盔",
            Armor => "铠甲",
            Gloves => "手套",
            Boots => "靴子",
            Cloak => "披风",
            Ring1 => "戒指1",
            Ring2 => "戒指2",
            Amulet => "项链",
            Special => "特殊",
        }
    }
}

/// 物品大类类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ItemType {
    /// 武器
    Weapon(WeaponCategory),
    /// 防具
    Armor(ArmorCategory),
    /// 饰品
    Accessory(EquipSlot),
    /// 消耗品（药水/卷轴/食物）
    Consumable,
    /// 任务物品
    QuestItem,
    /// 材料（制造用）
    Material,
    /// 货币
    Currency,
}

/// 武器分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum WeaponCategory {
    /// 简易近战
    SimpleMelee,
    /// 简易远程
    SimpleRanged,
    /// 军用近战
    MartialMelee,
    /// 军用远程
    MartialRanged,
}

/// 防具分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ArmorCategory {
    /// 轻甲
    Light,
    /// 中甲
    Medium,
    /// 重甲
    Heavy,
    /// 盾牌
    Shield,
}

// ─── 辅助数据类型 ──────────────────────────────────────────────────

/// 耐久度状态。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct DurabilityState {
    /// 当前耐久度
    pub current: u32,
    /// 最大耐久度
    pub max: u32,
    /// 是否损坏
    pub is_broken: bool,
}

impl DurabilityState {
    pub fn new(max: u32) -> Self {
        Self {
            current: max,
            max,
            is_broken: false,
        }
    }

    /// 减少耐久度。
    pub fn reduce(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
        if self.current == 0 {
            self.is_broken = true;
        }
    }

    /// 修复耐久度。
    pub fn repair(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
        if self.current > 0 {
            self.is_broken = false;
        }
    }
}

/// 物品移除原因。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum RemovalReason {
    /// 使用消耗
    Used,
    /// 丢弃
    Discarded,
    /// 交易
    Traded,
    /// 摧毁
    Destroyed,
    /// 装备到身上
    Equipped,
    /// 其他
    Other(String),
}

// ─── ECS Components ───────────────────────────────────────────────

/// 物品实例组件（当物品以独立 Entity 存在时使用）。
///
/// 对于数量庞大的一般性物品，存储在 Inventory.items 中；
/// 对于需要独立生命周期的特殊物品（已装备、附魔武器等），使用独立 Entity + 此组件。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ItemInstance {
    /// 引用的物品模板 ID（ItemDefId）
    pub template_id: String,
    /// 当前数量（可堆叠物品）
    pub quantity: u32,
    /// 耐久度（如果有）
    pub durability: Option<DurabilityState>,
    /// 自定义附魔 Modifier ID 列表
    pub enchants: Vec<String>,
}

impl ItemInstance {
    /// 创建新物品实例。
    pub fn new(template_id: impl Into<String>) -> Self {
        Self {
            template_id: template_id.into(),
            quantity: 1,
            durability: None,
            enchants: Vec::new(),
        }
    }

    /// 创建带数量的物品实例。
    pub fn with_quantity(template_id: impl Into<String>, quantity: u32) -> Self {
        Self {
            template_id: template_id.into(),
            quantity,
            durability: None,
            enchants: Vec::new(),
        }
    }

    /// 是否可堆叠。
    pub fn is_stackable(&self) -> bool {
        self.durability.is_none()
    }
}

/// 背包容器组件。
///
/// 管理角色持有的所有物品。支持堆叠、负重和槽位限制。
/// 详见 inventory_domain.md §5.1
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Inventory {
    /// 所有物品实例
    pub items: Vec<ItemInstance>,
    /// 最大物品格数
    pub max_slots: u32,
    /// 当前总重量（磅）
    pub total_weight: f32,
    /// 最大负重（磅，通常为 力量 × 15）
    pub max_weight: f32,
}

impl Inventory {
    /// 创建默认大小的背包。
    pub fn new(max_slots: u32, max_weight: f32) -> Self {
        Self {
            items: Vec::new(),
            max_slots,
            total_weight: 0.0,
            max_weight,
        }
    }

    /// 创建初始背包（默认 20 格，300 磅负重）。
    pub fn default() -> Self {
        Self::new(20, 300.0)
    }

    /// 当前物品数量。
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// 背包已用格数。
    pub fn used_slots(&self) -> usize {
        self.items.len()
    }

    /// 背包剩余格数。
    pub fn remaining_slots(&self) -> u32 {
        self.max_slots.saturating_sub(self.items.len() as u32)
    }

    /// 是否还有空位。
    pub fn has_free_slot(&self) -> bool {
        self.remaining_slots() > 0
    }

    /// 检查是否有足够空间容纳指定物品。
    pub fn can_hold(&self, item: &ItemInstance, item_weight: f32) -> bool {
        // 检查堆叠合并：如果有同类可堆叠物品且未满，则不需要新格子
        let needs_new_slot = if item.is_stackable() {
            !self
                .items
                .iter()
                .any(|i| i.template_id == item.template_id && i.quantity < 99)
        } else {
            true
        };

        if needs_new_slot && !self.has_free_slot() {
            return false;
        }

        // 检查负重：item_weight 是每单位重量，需要乘以数量
        let total_item_weight = if needs_new_slot && item.quantity > 1 {
            item_weight * item.quantity as f32
        } else {
            // 堆叠合并场景: can_hold 后实际添加量由 add_item 决定，保守用单重
            // 实际调用侧会再次检查
            item_weight
        };
        if self.total_weight + total_item_weight > self.max_weight {
            return false;
        }

        true
    }

    /// 计算可堆叠到已有物品的数量（自动上限 99）。
    fn stackable_to_existing(&self, template_id: &str, quantity: u32) -> u32 {
        for existing in self.items.iter() {
            if existing.template_id == template_id {
                let space = 99u32.saturating_sub(existing.quantity);
                if space > 0 {
                    return space.min(quantity);
                }
            }
        }
        0
    }

    /// 添加物品到背包（自动堆叠合并）。
    ///
    /// `weight` 是每单位重量（磅）。自动将数量上限限制为 99。
    /// 返回实际添加的数量。
    pub fn add_item(&mut self, item: ItemInstance, weight: f32) -> u32 {
        if !self.can_hold(&item, weight) {
            return 0;
        }

        // 尝试堆叠到已有物品
        if item.is_stackable() && item.quantity > 0 {
            let to_add = self.stackable_to_existing(&item.template_id, item.quantity);
            if to_add > 0 {
                for existing in self.items.iter_mut() {
                    if existing.template_id == item.template_id {
                        existing.quantity += to_add;
                        break;
                    }
                }
                self.total_weight += weight * to_add as f32;
                return to_add;
            }
        }

        // 需要新格子：数量上限 99，重量按实际数量计算
        let mut new_item = item;
        let actual_qty = new_item.quantity.min(99);
        new_item.quantity = actual_qty;
        self.total_weight += weight * actual_qty as f32;
        self.items.push(new_item);
        actual_qty
    }

    /// 从背包移除物品。
    ///
    /// 返回移除的数量（0 表示未找到）。
    pub fn remove_item(&mut self, template_id: &str, quantity: u32, weight_per_unit: f32) -> u32 {
        let mut remaining = quantity;
        self.items.retain_mut(|item| {
            if item.template_id == template_id && remaining > 0 {
                let to_remove = remaining.min(item.quantity);
                item.quantity = item.quantity.saturating_sub(to_remove);
                remaining -= to_remove;
                self.total_weight -= weight_per_unit * to_remove as f32;
                // 如果物品数量归零，移除该条目
                item.quantity > 0
            } else {
                true
            }
        });
        quantity - remaining
    }

    /// 查找指定模板 ID 的物品。
    pub fn find_item(&self, template_id: &str) -> Option<&ItemInstance> {
        self.items.iter().find(|i| i.template_id == template_id)
    }

    /// 查找指定模板 ID 的可变引用。
    pub fn find_item_mut(&mut self, template_id: &str) -> Option<&mut ItemInstance> {
        self.items.iter_mut().find(|i| i.template_id == template_id)
    }

    /// 检查是否拥有指定物品（及数量）。
    pub fn has_item(&self, template_id: &str, quantity: u32) -> bool {
        self.items
            .iter()
            .filter(|i| i.template_id == template_id)
            .map(|i| i.quantity)
            .sum::<u32>()
            >= quantity
    }
}

/// 装备槽位组件。
///
/// 定义角色身上各部位的装备位。
/// 每个槽位最多一件装备。
/// 详见 inventory_domain.md §5.2-5.3
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct EquipmentSlots {
    pub main_hand: Option<ItemInstance>,
    pub off_hand: Option<ItemInstance>,
    pub helmet: Option<ItemInstance>,
    pub armor: Option<ItemInstance>,
    pub gloves: Option<ItemInstance>,
    pub boots: Option<ItemInstance>,
    pub cloak: Option<ItemInstance>,
    pub ring_1: Option<ItemInstance>,
    pub ring_2: Option<ItemInstance>,
    pub amulet: Option<ItemInstance>,
    pub special: Option<ItemInstance>,
}

impl EquipmentSlots {
    /// 创建空的装备槽位。
    pub fn new() -> Self {
        Self {
            main_hand: None,
            off_hand: None,
            helmet: None,
            armor: None,
            gloves: None,
            boots: None,
            cloak: None,
            ring_1: None,
            ring_2: None,
            amulet: None,
            special: None,
        }
    }

    /// 获取指定槽位的装备。
    pub fn get(&self, slot: EquipSlot) -> Option<&ItemInstance> {
        match slot {
            EquipSlot::MainHand => self.main_hand.as_ref(),
            EquipSlot::OffHand => self.off_hand.as_ref(),
            EquipSlot::Helmet => self.helmet.as_ref(),
            EquipSlot::Armor => self.armor.as_ref(),
            EquipSlot::Gloves => self.gloves.as_ref(),
            EquipSlot::Boots => self.boots.as_ref(),
            EquipSlot::Cloak => self.cloak.as_ref(),
            EquipSlot::Ring1 => self.ring_1.as_ref(),
            EquipSlot::Ring2 => self.ring_2.as_ref(),
            EquipSlot::Amulet => self.amulet.as_ref(),
            EquipSlot::Special => self.special.as_ref(),
        }
    }

    /// 获取指定槽位的可变引用。
    pub fn get_mut(&mut self, slot: EquipSlot) -> &mut Option<ItemInstance> {
        match slot {
            EquipSlot::MainHand => &mut self.main_hand,
            EquipSlot::OffHand => &mut self.off_hand,
            EquipSlot::Helmet => &mut self.helmet,
            EquipSlot::Armor => &mut self.armor,
            EquipSlot::Gloves => &mut self.gloves,
            EquipSlot::Boots => &mut self.boots,
            EquipSlot::Cloak => &mut self.cloak,
            EquipSlot::Ring1 => &mut self.ring_1,
            EquipSlot::Ring2 => &mut self.ring_2,
            EquipSlot::Amulet => &mut self.amulet,
            EquipSlot::Special => &mut self.special,
        }
    }

    /// 装备物品到指定槽位。
    ///
    /// 如果槽位已有物品，返回旧物品。
    /// 不变量 3.1：每个槽位同一时间只能穿戴一件装备。
    pub fn equip(&mut self, slot: EquipSlot, item: ItemInstance) -> Option<ItemInstance> {
        let slot_ref = self.get_mut(slot);
        let old = slot_ref.take();
        *slot_ref = Some(item);
        old
    }

    /// 卸下指定槽位的装备。
    ///
    /// 返回被卸下的物品。
    pub fn unequip(&mut self, slot: EquipSlot) -> Option<ItemInstance> {
        self.get_mut(slot).take()
    }

    /// 检查槽位是否为空。
    pub fn is_slot_empty(&self, slot: EquipSlot) -> bool {
        self.get(slot).is_none()
    }

    /// 检查是否是双手武器槽位占用。
    ///
    /// 不变量 3.2：双手武器同时占用 MainHand + OffHand。
    /// 如果主手装备了双手武器，副手应标记为不可用。
    pub fn is_two_handed_weapon_equipped(&self) -> bool {
        self.main_hand.is_some() && self.off_hand.is_none()
        // 更准确的检查需要读取 ItemDef 的 is_two_handed 字段
    }
}

impl Default for EquipmentSlots {
    fn default() -> Self {
        Self::new()
    }
}

/// 背包标记组件。
///
/// 标记具有完整背包系统的实体。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct InventoryMarker;
