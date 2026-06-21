//! Shared constants used across the crate.
//!
//! This module contains game-wide constants that have zero business or technical
//! semantics — they are pure configuration values shared across layers.
//! For domain-specific constants, see the respective domain module.

/// Maximum depth for Observer chain reactions.
///
/// When observers trigger other observers recursively, this limit prevents
/// infinite loops. Set to 10 based on ADR-002 recommendation.
///
/// Violation at runtime triggers a WARN log but does not panic.
pub const MAX_OBSERVER_DEPTH: u32 = 10;

/// Maximum party members.
pub const MAX_PARTY_SIZE: usize = 6;

/// Maximum inventory slots per entity.
pub const MAX_INVENTORY_SIZE: usize = 100;

/// Default maximum buff stack count.
pub const MAX_BUFF_STACK: usize = 5;
