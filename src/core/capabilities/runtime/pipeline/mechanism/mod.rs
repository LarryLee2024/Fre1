//! Pipeline Mechanism — 管线执行引擎

pub mod executor;

pub use executor::{StepExecutor, execute_pipeline, validate_pipeline};
