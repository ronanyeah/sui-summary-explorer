use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::{
        CallToolResult, Content, Implementation, InitializeRequestParam, InitializeResult,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};

#[derive(Clone)]
pub struct SuiService {
    packages: crate::PackageTree,
    tool_router: ToolRouter<SuiService>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListModulesRequest {
    #[schemars(description = "package name")]
    pub package: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ModuleRequest {
    #[schemars(description = "package name")]
    pub package: String,
    #[schemars(description = "module name")]
    pub module: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DefinitionRequest {
    #[schemars(description = "package name")]
    pub package: String,
    #[schemars(description = "module name")]
    pub module: String,
    #[schemars(description = "function/struct/enum name")]
    pub definition: String,
}

#[tool_router]
impl SuiService {
    pub fn new(packages: crate::PackageTree) -> Self {
        Self {
            packages,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List packages")]
    async fn list_packages(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let packages = self.packages.list_packages();
        let out = Content::json(packages)?;
        Ok(CallToolResult::success(vec![out]))
    }

    #[tool(description = "List modules")]
    async fn list_modules(
        &self,
        Parameters(data): Parameters<ListModulesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let modules = self.packages.list_modules(&data.package);
        let out = Content::json(modules)?;
        Ok(CallToolResult::success(vec![out]))
    }

    #[tool(description = "Read module")]
    async fn read_module(
        &self,
        Parameters(ModuleRequest { package, module }): Parameters<ModuleRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let module = self.packages.get_module(&package, &module).unwrap();

        let out = serde_json::json!({
            "functions": module.functions.keys().collect::<Vec<_>>(),
            "structs": module.structs.keys().collect::<Vec<_>>(),
            "enums": module.enums.keys().collect::<Vec<_>>()
        });

        let out = Content::json(out)?;
        Ok(CallToolResult::success(vec![out]))
    }

    #[tool(description = "Read module definition")]
    async fn read_module_definition(
        &self,
        Parameters(DefinitionRequest {
            package,
            module,
            definition,
        }): Parameters<DefinitionRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let module = self.packages.get_module(&package, &module).unwrap();
        let def: move_symbol_pool::Symbol = definition.into();
        let definition = if let Some(val) = module.functions.get(&def) {
            ("FUNCTION", serde_json::to_value(val).unwrap())
        } else if let Some(val) = module.structs.get(&def) {
            ("STRUCT", serde_json::to_value(val).unwrap())
        } else if let Some(val) = module.enums.get(&def) {
            ("ENUM", serde_json::to_value(val).unwrap())
        } else {
            return Err(rmcp::ErrorData::internal_error(
                "Definition not found",
                None,
            ));
        };
        let out = Content::json(definition)?;
        Ok(CallToolResult::success(vec![out]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for SuiService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides tools to introspect the definitions and dependencies of a Sui Move project."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, rmcp::ErrorData> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            println!(
                "initialize from http server: headers={:?}, uri={}",
                initialize_headers, initialize_uri
            );
        }
        Ok(self.get_info())
    }
}
