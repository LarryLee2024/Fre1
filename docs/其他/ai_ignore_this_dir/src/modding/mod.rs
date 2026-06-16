//! Modding 层：MOD 支持（Layer 6）
//!
//! Layer 6 职责：通过稳定 API 扩展，禁止绕过 Effect Pipeline。
//!
//! ## 子模块
//!
//! - `api/` — 稳定公开接口，MOD 作者唯一需要了解的部分
//! - `registry/` — MOD 注册表、依赖解析、加载顺序
//! - `loaders/` — 内容加载器，基础内容与 MOD 内容合并
//! - `validators/` — Schema 校验、引用完整性、冲突检测
//! - `sandbox/` — 沙箱环境、权限控制
//! - `compatibility/` — 版本检查、兼容性矩阵
//!
//! ## 依赖规则
//!
//! Modding → Core    ✅ 允许（通过稳定 API 接口）
//! Modding → Shared  ✅ 允许
//! Modding → Content ✅ 允许（MOD 也是一种内容加载）
//! Modding → Infra   ✅ 允许（MOD 加载需要基础设施）
//! Modding → UI/App  ❌ 禁止
//!
//! 详细设计见 `docs/architecture/modding-design.md`。
//! 参见 `docs/architecture/layer-contracts.md` §第六层。

pub mod api;
pub mod compatibility;
pub mod loaders;
pub mod registry;
pub mod sandbox;
pub mod validators;
