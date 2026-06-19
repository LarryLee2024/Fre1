//! 标准测试数据 Builder
//!
//! 提供 Unit_001/002/003 标准测试角色构造器。
//! 所有测试必须使用这里的 Builder，禁止自定义测试数据。

use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeDefinition, AttributeId,
};
use crate::core::capabilities::effect::foundation::{EffectDuration, EffectInstance};
use crate::core::capabilities::modifier::foundation::{
    ModifierData, ModifierInstanceId, ModifierOp, ModifierSource, ModifierSourceType,
};
use crate::core::capabilities::tag::foundation::{TagDefinition, TagId, TagNamespace};

// ── 标准测试单位 ─────────────────────────────────────────

/// Unit_001（基础战士）: HP=100, ATK=30, DEF=10, SPD=10, Range=1
pub struct Unit001;

/// Unit_002（基础法师）: HP=80, ATK=40, DEF=5, SPD=12, Range=3
pub struct Unit002;

/// Unit_003（基础坦克）: HP=150, ATK=20, DEF=20, SPD=5, Range=1
pub struct Unit003;

// ── 属性 Builder ─────────────────────────────────────────

/// 属性定义 Builder，支持链式构造。
pub struct AttributeDefBuilder {
    id: String,
    category: AttributeCategory,
    default_value: f32,
    min_value: f32,
    max_value: f32,
    dependencies: Vec<String>,
}

impl AttributeDefBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category: AttributeCategory::Primary,
            default_value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            dependencies: Vec::new(),
        }
    }

    pub fn category(mut self, cat: AttributeCategory) -> Self {
        self.category = cat;
        self
    }

    pub fn default_value(mut self, v: f32) -> Self {
        self.default_value = v;
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn build(self) -> AttributeDefinition {
        AttributeDefinition {
            id: AttributeId::new(&self.id),
            category: self.category,
            default_base_value: self.default_value,
            min_value: self.min_value,
            max_value: self.max_value,
            derived_dependencies: self
                .dependencies
                .into_iter()
                .map(AttributeId::new)
                .collect(),
            hidden: false,
        }
    }
}

// ── 标准属性集 Builder ──────────────────────────────────

/// 构造标准 Unit_001 的全部属性定义。
pub fn attributes_for_unit_001() -> Vec<AttributeDefinition> {
    vec![
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(0.0, 100.0)
            .build(),
        AttributeDefBuilder::new("attr_atk")
            .category(AttributeCategory::Primary)
            .default_value(30.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_def")
            .category(AttributeCategory::Primary)
            .default_value(10.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_spd")
            .category(AttributeCategory::Primary)
            .default_value(10.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_range")
            .category(AttributeCategory::Primary)
            .default_value(1.0)
            .range(1.0, 20.0)
            .build(),
    ]
}

/// 构造标准 Unit_002 的全部属性定义。
pub fn attributes_for_unit_002() -> Vec<AttributeDefinition> {
    vec![
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(80.0)
            .range(0.0, 80.0)
            .build(),
        AttributeDefBuilder::new("attr_atk")
            .category(AttributeCategory::Primary)
            .default_value(40.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_def")
            .category(AttributeCategory::Primary)
            .default_value(5.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_spd")
            .category(AttributeCategory::Primary)
            .default_value(12.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_range")
            .category(AttributeCategory::Primary)
            .default_value(3.0)
            .range(1.0, 20.0)
            .build(),
    ]
}

/// 构造标准 Unit_003 的全部属性定义。
pub fn attributes_for_unit_003() -> Vec<AttributeDefinition> {
    vec![
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(150.0)
            .range(0.0, 150.0)
            .build(),
        AttributeDefBuilder::new("attr_atk")
            .category(AttributeCategory::Primary)
            .default_value(20.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_def")
            .category(AttributeCategory::Primary)
            .default_value(20.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_spd")
            .category(AttributeCategory::Primary)
            .default_value(5.0)
            .range(0.0, 999.0)
            .build(),
        AttributeDefBuilder::new("attr_range")
            .category(AttributeCategory::Primary)
            .default_value(1.0)
            .range(1.0, 20.0)
            .build(),
    ]
}

// ── Modifier Builder ────────────────────────────────────

/// Modifier Builder，支持链式构造。
pub struct ModifierBuilder {
    id: u64,
    op: ModifierOp,
    target: String,
    magnitude: f32,
    priority: u8,
    source_type: ModifierSourceType,
    source_id: String,
    duration: Option<u64>,
}

impl ModifierBuilder {
    pub fn new(target: impl Into<String>, magnitude: f32) -> Self {
        Self {
            id: 1,
            op: ModifierOp::Add,
            target: target.into(),
            magnitude,
            priority: 50,
            source_type: ModifierSourceType::Buff,
            source_id: "buf_000001".to_string(),
            duration: None,
        }
    }

    pub fn op(mut self, op: ModifierOp) -> Self {
        self.op = op;
        self
    }

    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    pub fn source(mut self, source_type: ModifierSourceType, source_id: impl Into<String>) -> Self {
        self.source_type = source_type;
        self.source_id = source_id.into();
        self
    }

    pub fn duration_frames(mut self, frames: u64) -> Self {
        self.duration = Some(frames);
        self
    }

    pub fn build(self) -> ModifierData {
        ModifierData {
            id: ModifierInstanceId::new(self.id),
            op: self.op,
            target_attribute: self.target,
            magnitude: self.magnitude,
            priority: self.priority,
            source: ModifierSource {
                source_type: self.source_type,
                source_id: self.source_id,
            },
            duration_frames: self.duration,
            elapsed_frames: 0,
        }
    }
}

// ── Effect Builder ──────────────────────────────────────

/// Effect Builder，支持链式构造。
pub struct EffectBuilder {
    instance_id: String,
    def_id: String,
    category: String,
    source: String,
    target: String,
    duration: EffectDuration,
    created_at_turn: u64,
}

impl EffectBuilder {
    pub fn new(
        def_id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        let def_id_str = def_id.into();
        Self {
            instance_id: format!("eff_{}", &def_id_str),
            def_id: def_id_str,
            category: "Buff".to_string(),
            source: source.into(),
            target: target.into(),
            duration: EffectDuration::Instant,
            created_at_turn: 1,
        }
    }

    pub fn instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = id.into();
        self
    }

    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }

    pub fn duration(mut self, d: EffectDuration) -> Self {
        self.duration = d;
        self
    }

    pub fn created_at_turn(mut self, turn: u64) -> Self {
        self.created_at_turn = turn;
        self
    }

    pub fn build(self) -> EffectInstance {
        EffectInstance::new(
            &self.instance_id,
            &self.def_id,
            &self.category,
            &self.source,
            &self.target,
            self.duration,
            self.created_at_turn,
        )
    }
}

// ── Tag Builder ─────────────────────────────────────────

/// Tag Builder，支持链式构造。
pub struct TagDefBuilder {
    id: String,
    path: String,
    parent: Option<String>,
    bit_index: u32,
    is_abstract: bool,
    namespace: TagNamespace,
}

impl TagDefBuilder {
    pub fn new(id: impl Into<String>, namespace: TagNamespace) -> Self {
        Self {
            id: id.into(),
            path: String::new(),
            parent: None,
            bit_index: 0,
            is_abstract: false,
            namespace,
        }
    }

    pub fn path(mut self, p: impl Into<String>) -> Self {
        self.path = p.into();
        self
    }

    pub fn parent(mut self, p: impl Into<String>) -> Self {
        self.parent = Some(p.into());
        self
    }

    pub fn bit_index(mut self, i: u32) -> Self {
        self.bit_index = i;
        self
    }

    pub fn abstract_(mut self, v: bool) -> Self {
        self.is_abstract = v;
        self
    }

    pub fn build(self) -> TagDefinition {
        TagDefinition {
            id: TagId::new(&self.id),
            path: self.path,
            parent_id: self.parent.map(TagId::new),
            bit_index: self.bit_index,
            is_abstract: self.is_abstract,
            namespace: self.namespace,
        }
    }
}

// ── 标准 Tag 集 ─────────────────────────────────────────

/// 构造标准伤害类型 Tag 层级。
pub fn standard_damage_tags() -> Vec<TagDefinition> {
    vec![
        TagDefBuilder::new("tag_dmg_elemental", TagNamespace::Damage)
            .path("DamageType.Elemental")
            .bit_index(0)
            .abstract_(true)
            .build(),
        TagDefBuilder::new("tag_dmg_fire", TagNamespace::Damage)
            .path("DamageType.Elemental.Fire")
            .parent("tag_dmg_elemental")
            .bit_index(1)
            .build(),
        TagDefBuilder::new("tag_dmg_ice", TagNamespace::Damage)
            .path("DamageType.Elemental.Ice")
            .parent("tag_dmg_elemental")
            .bit_index(2)
            .build(),
        TagDefBuilder::new("tag_dmg_phys", TagNamespace::Damage)
            .path("DamageType.Physical")
            .bit_index(3)
            .build(),
    ]
}
