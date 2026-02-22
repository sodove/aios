//! MCP tool definitions and execution framework for AIOS.
//!
//! Provides the [`Tool`](executor::Tool) trait, [`ToolRegistry`](registry::ToolRegistry),
//! and a collection of built-in tools for file operations, system management,
//! and device control.

pub mod chrome_mcp;
pub mod executor;
pub mod registry;
pub mod tools;
