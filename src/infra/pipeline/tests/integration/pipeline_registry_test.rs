//! 管线-注册中心跨层集成测试（Pipeline → Registry）
//!
//! 验证 Pipeline 系统与 Registry 系统协同工作时：
//! - PipelinePlugin + RegistryPlugin 可共存
//! - PipelineRegistry 与 DefinitionRegistry 各自独立初始化
//! - 管线定义可在 Registry 中注册和查询
//! - 使用 Registry 数据的步骤执行器输出正确
//!
//! 领域规则来源：
//! - ADR-044: pipeline-schema.md（执行管线架构）
//! - docs/04-data/infrastructure/registry_schema.md（注册中心架构）

use bevy::prelude::*;

use crate::infra::pipeline::{
    PipelineRegistry, PipelineDefinition, PipelineStage, PipelineStep, PipelineContext, StepResult,
    execute_pipeline, validate_pipeline,
};
use crate::infra::pipeline::PipelinePlugin;
use crate::infra::registry::registry::{DefinitionRegistry, RegistryEntry};
use crate::infra::registry::RegistryPlugin;
use crate::shared::error::ErrorContext;

/// PipelinePlugin 和 RegistryPlugin 共存测试。
///
/// Given: 同时添加 PipelinePlugin 和 RegistryPlugin 的 App
/// When: 检查所有 Resource
/// Then: 两种 Resource 正确初始化
#[test]
fn pipeline_and_registry_plugins_coexist() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins((PipelinePlugin, RegistryPlugin));

    let world = app.world();
    assert!(
        world.contains_resource::<PipelineRegistry>(),
        "PipelineRegistry should exist"
    );
    assert!(
        world.contains_resource::<DefinitionRegistry>(),
        "DefinitionRegistry should exist"
    );
    assert_eq!(
        world.resource::<PipelineRegistry>().count(),
        0,
        "PipelineRegistry should start empty"
    );
    assert!(
        world.resource::<DefinitionRegistry>().is_empty(),
        "DefinitionRegistry should start empty"
    );
}

/// Pipeline 定义可在 PipelineRegistry 中注册和查询。
///
/// Given: 一个 PipelineDefinition
/// When: 注册到 PipelineRegistry 后查询
/// Then: 可正确检索定义，且定义数据与注册时一致
#[test]
fn pipeline_registry_register_and_query() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins((PipelinePlugin, RegistryPlugin));

    // 创建管线定义
    let def = PipelineDefinition::new("combat_pipeline")
        .stage(
            PipelineStage::new("init")
                .step(PipelineStep::System("spawn_units"))
                .step(PipelineStep::System("init_attributes")),
        )
        .stage(
            PipelineStage::new("resolve")
                .step(PipelineStep::Rule("calculate_damage"))
                .step(PipelineStep::Rule("apply_effects")),
        );

    // 注册管线
    app.world_mut()
        .resource_mut::<PipelineRegistry>()
        .register(def);

    // 查询验证
    assert_eq!(app.world().resource::<PipelineRegistry>().count(), 1);

    let registered = app
        .world()
        .resource::<PipelineRegistry>()
        .get("combat_pipeline")
        .expect("pipeline should be found");
    assert_eq!(registered.id, "combat_pipeline");
    assert_eq!(registered.stage_count(), 2);

    let init_stage = registered.find_stage("init").expect("init stage");
    assert_eq!(init_stage.steps.len(), 2);

    let resolve_stage = registered.find_stage("resolve").expect("resolve stage");
    assert_eq!(resolve_stage.steps.len(), 2);
}

/// DefinitionRegistry 中添加 Def 定义后，
/// Pipeline 执行时可从 PipelineContext 中读取对应的 Def 数据。
///
/// Given: DefinitionRegistry 中注册了 Def,
///        同时 PipelineRegistry 中注册了对此 Def 的管线定义
/// When: 执行管线时，步骤执行器从上下文读取 Def ID
/// Then: 执行结果与 Def 数据一致
#[test]
fn pipeline_execution_uses_registry_data() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins((PipelinePlugin, RegistryPlugin));

    // ── 准备：向 DefinitionRegistry 注册 Def ──
    {
        let mut registry = app.world_mut().resource_mut::<DefinitionRegistry>();
        let abilities = registry
            .bucket_mut("abilities")
            .expect("abilities bucket");
        let entry = RegistryEntry::new("abl_fireball")
            .with_data(serde_json::json!({"damage": 50, "aoe": true}));
        abilities.insert("abl_fireball", entry);

        let effects = registry.bucket_mut("effects").expect("effects bucket");
        let entry = RegistryEntry::new("eff_burn")
            .with_data(serde_json::json!({"duration": 3, "dot_per_turn": 10}));
        effects.insert("eff_burn", entry);
    }

    // ── 准备：向 PipelineRegistry 注册管线 ──
    {
        let def = PipelineDefinition::new("damage_pipeline")
            .stage(
                PipelineStage::new("calc")
                    .step(PipelineStep::Rule("lookup_ability"))
                    .step(PipelineStep::Rule("compute_damage")),
            )
            .stage(
                PipelineStage::new("apply")
                    .step(PipelineStep::Rule("apply_damage"))
                    .step(PipelineStep::System("spawn_effects")),
            );

        app.world_mut()
            .resource_mut::<PipelineRegistry>()
            .register(def);
    }

    // ── 验证两个注册中心的数据 ──
    assert_eq!(
        app.world().resource::<DefinitionRegistry>().total_count(),
        2,
        "2 definitions should be registered"
    );
    assert_eq!(
        app.world().resource::<PipelineRegistry>().count(),
        1,
        "1 pipeline should be registered"
    );

    // ── 执行管线：上下文预载 Def ID（模拟实际系统的工作流）──
    let pipeline = app
        .world()
        .resource::<PipelineRegistry>()
        .get("damage_pipeline")
        .expect("pipeline should exist");

    let mut context = PipelineContext::new("damage_pipeline");
    // 预载 registry 数据到上下文——模拟实际执行管线前的准备阶段
    context.set_stage_data("calc", "abl_fireball");
    context.set_stage_data("apply", "eff_burn");

    let result = execute_pipeline(pipeline, &mut context, |step_name, ctx, _stage_name| {
        match step_name {
            "lookup_ability" => {
                let ability_id = ctx.get_stage_data("calc");
                assert!(ability_id.is_some(), "ability ID should be in context");
                assert_eq!(ability_id.unwrap(), "abl_fireball");
                StepResult::Success
            }
            "compute_damage" => StepResult::Success,
            "apply_damage" => StepResult::Success,
            "spawn_effects" => {
                // 读取效果定义 ID 并记录到上下文
                let eff_id = ctx.get_stage_data("apply").unwrap_or(&"none".into());
                ctx.set_stage_data("spawned", &format!("effects:{}", eff_id));
                StepResult::Success
            }
            _ => StepResult::Failure(ErrorContext {
                domain: "pipeline_test",
                source: format!("unknown step: {}", step_name),
                context: None,
            }),
        }
    });

    assert!(result.is_ok(), "pipeline execution should succeed");
    assert_eq!(context.execution_log.len(), 4, "4 steps should be logged");
    assert!(!context.aborted, "pipeline should not abort");

    // 验证 spawned 数据被正确写入
    let spawned = context.get_stage_data("spawned");
    assert_eq!(spawned.unwrap(), "effects:eff_burn");
}

/// 管线验证（validate_pipeline）确保定义合法性，
/// 与 Registry 系统协同：只接受合法的管线注册。
///
/// Given: 一个空 ID 的 PipelineDefinition
/// When: 调用 validate_pipeline
/// Then: 返回 Err
#[test]
fn pipeline_validation_rejects_empty_id() {
    let def = PipelineDefinition::new("")
        .stage(PipelineStage::new("s1").step(PipelineStep::System("step1")));

    let result = validate_pipeline(&def);
    assert!(result.is_err(), "empty pipeline ID should be rejected");
}

/// 管线验证拒绝空步骤名称的阶段。
///
/// Given: 一个阶段名称为空的 PipelineDefinition
/// When: 调用 validate_pipeline
/// Then: 返回 Err
#[test]
fn pipeline_validation_rejects_empty_stage_name() {
    let def = PipelineDefinition::new("test")
        .stage(PipelineStage::new("").step(PipelineStep::System("step1")));

    let result = validate_pipeline(&def);
    assert!(result.is_err(), "empty stage name should be rejected");
}

/// 管线验证拒绝无步骤且不可跳过的阶段。
///
/// Given: 无步骤且非 skippable 的 PipelineStage
/// When: 调用 validate_pipeline
/// Then: 返回 Err
#[test]
fn pipeline_validation_rejects_empty_stage_without_skip() {
    let def = PipelineDefinition::new("test").stage(PipelineStage::new("empty_stage"));

    let result = validate_pipeline(&def);
    assert!(
        result.is_err(),
        "stage with no steps should be rejected unless skippable"
    );
}
