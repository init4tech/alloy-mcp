use std::collections::HashMap;

use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::{
        prompt::PromptContext,
        router::{prompt::PromptRouter, tool::ToolRouter},
    },
    model::{
        Annotated, GetPromptRequestParams, GetPromptResult, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents, ServerCapabilities,
        ServerInfo,
    },
    service::RequestContext,
};

use crate::resources::StaticResource;

/// The alloy MCP server handler.
#[derive(Clone)]
pub struct AlloyMcpServer {
    /// Static resources indexed by URI.
    pub(crate) resources: HashMap<String, StaticResource>,
    /// Tool router for handling tool calls (read by generated macro code).
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    /// Prompt router for handling prompt requests.
    #[allow(dead_code)]
    prompt_router: PromptRouter<Self>,
}

impl Default for AlloyMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AlloyMcpServer {
    pub fn new() -> Self {
        Self {
            resources: crate::resources::all(),
            tool_router: Self::create_tool_router(),
            prompt_router: Self::create_prompt_router(),
        }
    }
}

impl ServerHandler for AlloyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .enable_prompts()
                .build(),
            instructions: Some(
                "Provides curated documentation for alloy.rs Ethereum library types.".into(),
            ),
            ..Default::default()
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_ {
        let resources = self
            .resources
            .values()
            .map(|r| Annotated {
                raw: rmcp::model::RawResource {
                    uri: r.uri.clone(),
                    name: r.name.clone(),
                    title: None,
                    description: Some(r.description.clone()),
                    mime_type: Some(r.mime_type.clone()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            })
            .collect();

        std::future::ready(Ok(ListResourcesResult {
            resources,
            ..Default::default()
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, ErrorData>> + Send + '_ {
        let result = match self.resources.get(&request.uri) {
            Some(resource) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: resource.uri.clone(),
                    mime_type: Some(resource.mime_type.clone()),
                    text: resource.content.clone(),
                    meta: None,
                }],
            }),
            None => Err(ErrorData::resource_not_found(
                format!("Resource not found: {}", request.uri),
                None,
            )),
        };

        std::future::ready(result)
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListResourceTemplatesResult {
            resource_templates: vec![Annotated {
                raw: rmcp::model::RawResourceTemplate {
                    uri_template: "alloy://type/{type_name}".to_string(),
                    name: "Type Lookup".to_string(),
                    title: None,
                    description: Some("Look up a specific alloy type by name".to_string()),
                    mime_type: None,
                    icons: None,
                },
                annotations: None,
            }],
            ..Default::default()
        }))
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, ErrorData>> + Send + '_ {
        let prompts = self.prompt_router.list_all();
        std::future::ready(Ok(ListPromptsResult {
            prompts,
            ..Default::default()
        }))
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, ErrorData>> + Send + '_ {
        let prompt_context = PromptContext::new(self, request.name, request.arguments, context);
        async move { self.prompt_router.get_prompt(prompt_context).await }
    }
}
