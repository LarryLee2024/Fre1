//! Content 加载管线
//!
//! 定义 DefinitionType trait、RON 文件加载器、文件发现和校验逻辑。
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md`

mod definition_type;
mod discovery;
mod errors;
mod ron_loader;

pub use definition_type::{DefinitionType, validate_id_format};
pub use discovery::{ContentFile, discover_ron_files};
pub use errors::{ConfigError, ValidationError};
pub use ron_loader::RonAssetLoader;
