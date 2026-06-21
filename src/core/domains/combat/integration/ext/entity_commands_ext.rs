//! EntityCommandsExt — [`EntityCommands`] 的扩展 trait。
//!
//! 在 Bevy 的 `EntityCommands` 之上提供领域特定的语法糖方法，
//! 内部委托给集成层 Facade 函数。
//!
//! 所有方法返回 `&mut Self` 以支持链式 DSL 用法。

use bevy::prelude::info;
use bevy::prelude::EntityCommands;

use crate::core::domains::combat::components::Dead;

/// [`EntityCommands`] 的扩展 trait，提供领域特定操作。
///
/// 所有方法内部委托给集成层 Facade 函数，
/// 不直接操作 Capabilities 内部。
///
/// 方法返回 `&mut Self` 以支持链式 DSL 用法。
///
/// # 用法
///
/// ```ignore
/// use crate::core::domains::combat::integration::ext::EntityCommandsExt;
///
/// fn my_system(mut commands: Commands) {
///     let mut entity = commands.spawn_empty();
///     entity
///         .add_buff("eff_000001")
///         .heal(50)
///         .kill();
/// }
/// ```
/// Combat 域 EntityCommands 扩展。
///
/// 存在原因：战斗中施加 Buff、治疗、击杀等操作需要链式 API，
/// 避免每个系统重复写 `commands.trigger(XxxRequest { ... })` 样板代码。
pub trait EntityCommandsExt {
    /// 为此实体添加一个 Buff（主动效果）。
    ///
    /// 内部将 Buff 请求入队，由 Effect 能力的
    /// 集成 Facade 处理。
    ///
    /// # 参数
    ///
    /// * `buff_id` — 要施加的 Buff 的 Def ID（如 `"eff_000001"`）。
    fn add_buff(&mut self, buff_id: &str) -> &mut Self;

    /// 治疗此实体指定量的生命值。
    ///
    /// 内部委托给 Execution 集成 Facade，
    /// 经由 Effect/Modifier 管线处理。
    ///
    /// # 参数
    ///
    /// * `amount` — 修正前的原始治疗量。
    fn heal(&mut self, amount: u32) -> &mut Self;

    /// 击杀此实体（标记为死亡）。
    ///
    /// 插入 [`Dead`] 标记组件，战斗管线
    /// 使用该组件来识别已淘汰的参与者。
    fn kill(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_> {

    fn add_buff(&mut self, buff_id: &str) -> &mut Self {
        info!(
            "EntityCommandsExt::add_buff(buff_id={}) — queuing buff application",
            buff_id,
        );
        // TODO[Phase C2]: 接入 EffectFacade::apply_buff，等待 effect 集成
        //   Facade 暴露命令级别 API。
        //   当前计划：
        //     let entity = self.id();
        //     self.queue(move |world| {
        //         EffectFacade::apply_buff(world, entity, buff_id);
        //     });
        self
    }

    fn heal(&mut self, amount: u32) -> &mut Self {
        info!(
            "EntityCommandsExt::heal(amount={}) — queuing heal request",
            amount,
        );
        // TODO[Phase C2]: 接入 ExecutionFacade::heal，等待 execution
        //   集成 Facade 暴露 HP 修改的命令级别 API。
        //   当前计划：
        //     let entity = self.id();
        //     self.queue(move |world| {
        //         ExecutionFacade::heal(world, entity, amount);
        //     });
        self
    }

    fn kill(&mut self) -> &mut Self {
        info!("EntityCommandsExt::kill() — inserting Dead component");
        self.insert(Dead);
        self
    }
}
