//! Central registry for discovering and dispatching tools.

use std::collections::HashMap;

use aios_common::ToolDefinition;

use crate::executor::Tool;

/// A registry that holds all available tools keyed by name.
///
/// Use [`ToolRegistry::with_defaults`] to get a registry pre-populated with
/// every built-in tool, or [`ToolRegistry::new`] to build one selectively.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool. If a tool with the same name already exists it will be
    /// replaced.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }

    /// Look up a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(AsRef::as_ref)
    }

    /// Return the definitions of every registered tool (unordered).
    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Create a registry pre-populated with all built-in tools.
    #[must_use]
    pub fn with_defaults() -> Self {
        use crate::tools::*;

        let mut registry = Self::new();

        // File tools
        registry.register(Box::new(file_read::FileReadTool));
        registry.register(Box::new(file_write::FileWriteTool));
        registry.register(Box::new(file_delete::FileDeleteTool));
        registry.register(Box::new(file_list::FileListTool));
        registry.register(Box::new(file_search::FileSearchTool));

        // System tools
        registry.register(Box::new(shell_exec::ShellExecTool));
        registry.register(Box::new(wifi_list::WifiListTool));
        registry.register(Box::new(wifi_connect::WifiConnectTool));
        registry.register(Box::new(brightness::BrightnessTool));
        registry.register(Box::new(volume::VolumeTool));
        registry.register(Box::new(system_info::SystemInfoTool));
        registry.register(Box::new(open_url::OpenUrlTool));

        // Browser tools (Chrome MCP bridge)
        registry.register(Box::new(browser::BrowserNavigateTool));
        registry.register(Box::new(browser::BrowserReadPageTool));
        registry.register(Box::new(browser::BrowserFindTool));
        registry.register(Box::new(browser::BrowserClickTool));
        registry.register(Box::new(browser::BrowserTypeTool));
        registry.register(Box::new(browser::BrowserScreenshotTool));
        registry.register(Box::new(browser::BrowserGetPageTextTool));

        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
