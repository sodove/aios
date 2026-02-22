//! Chrome MCP client integration.
//!
//! This module is a placeholder for future integration with the Chrome MCP
//! extension via `rmcp` 0.16.0.
//!
//! # Requirements for full integration
//!
//! 1. **Chrome MCP extension** installed in Chromium
//! 2. **MCP server** running (stdio or SSE transport)
//! 3. **`rmcp` client** connected to the server
//!
//! Once connected, the browser stub tools in [`crate::tools::browser`] will
//! delegate execution to this client instead of returning "not available"
//! errors.
//!
//! # Architecture
//!
//! ```text
//! aios-mcp ToolRegistry
//!   -> BrowserReadPageTool::execute()
//!     -> ChromeMcpClient::call_tool("read_page", args)
//!       -> rmcp transport (stdio / SSE)
//!         -> Chrome MCP extension
//! ```
//!
//! # TODO
//!
//! - Add `rmcp` dependency to `Cargo.toml`
//! - Implement `ChromeMcpClient` with connection lifecycle
//! - Wire browser tools to delegate through the client
//! - Handle reconnection and Chrome extension discovery
