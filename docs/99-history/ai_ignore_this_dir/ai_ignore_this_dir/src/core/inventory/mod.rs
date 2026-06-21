/// 背包模块：容器系统、物品定义、实例管理、物品转移与使用
/// 支持从 content/items/*.ron 外部配置文件加载物品定义

/// 战斗背包容器（消耗品快捷使用）
pub mod battle_bag;
/// Container 容器抽象（背包/箱子/仓库）
pub mod container;
/// ItemDef 定义与 ItemRegistry 注册表
pub mod def;
/// 背包领域模块（错误码 I001-I005 等）
mod domain;
/// ItemInstance, ItemStack 实例管理
pub mod instance;
/// Resources 资源堆叠（金币/素材）
pub mod resources;
/// 物品转移逻辑（TransferItem 消息处理）
pub mod transfer;
/// 物品使用逻辑（消耗品/装备/技能书）
pub mod use_item;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;
use container::{Container, ContainerKind};
use def::{ItemRegistry, ItemType, UseEffect};
pub use domain::*;
use instance::{InstanceIdCounter, ItemBind, ItemInstance, ItemStack};
use resources::{ResourceStack, Resources};
use transfer::{ItemTransferred, TransferItem};
use use_item::{CastSkillEffect, GrantTempTraitEffect, ItemUsed, UseItem};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        let registry = ItemRegistry::load_from_dir("content/items");
        app.insert_resource(registry);
        app.insert_resource(InstanceIdCounter::default());
        // 注册 Reflect 类型
        app.register_type::<ItemType>()
            .register_type::<UseEffect>()
            .register_type::<ItemBind>()
            .register_type::<ItemInstance>()
            .register_type::<ItemStack>()
            .register_type::<InstanceIdCounter>()
            .register_type::<ContainerKind>()
            .register_type::<Container>()
            .register_type::<ResourceStack>()
            .register_type::<Resources>();
        // 注册 Message（Bevy 0.18 要求）
        app.add_message::<UseItem>();
        app.add_message::<ItemUsed>();
        app.add_message::<GrantTempTraitEffect>();
        app.add_message::<CastSkillEffect>();
        app.add_message::<TransferItem>();
        app.add_message::<ItemTransferred>();
        // 系统
        app.add_systems(Update, use_item::use_item_system);
        app.add_systems(Update, transfer::transfer_item_system);
    }
}
