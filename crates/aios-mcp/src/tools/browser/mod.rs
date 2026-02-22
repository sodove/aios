//! Browser tools for web page interaction.
//!
//! Currently `browser_navigate` opens URLs in Chromium directly.
//! All other tools are stubs awaiting Chrome MCP extension integration
//! via `rmcp` (see [`crate::chrome_mcp`]).

pub mod click;
pub mod find_element;
pub mod get_page_text;
pub mod navigate;
pub mod read_page;
pub mod screenshot;
pub mod type_text;

pub use click::BrowserClickTool;
pub use find_element::BrowserFindTool;
pub use get_page_text::BrowserGetPageTextTool;
pub use navigate::BrowserNavigateTool;
pub use read_page::BrowserReadPageTool;
pub use screenshot::BrowserScreenshotTool;
pub use type_text::BrowserTypeTool;
