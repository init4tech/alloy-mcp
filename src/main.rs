//! Alloy MCP Server
//!
//! A minimal MCP server providing type context for alloy.rs.
//! Helps LLMs correctly use alloy library types.

use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        Annotated, ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents, ServerCapabilities,
        ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_router,
};
use std::collections::HashMap;
use tokio::io::{stdin, stdout};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Resource content loaded at compile time
mod resources {
    pub const TRANSACTIONS: &str = include_str!("../resources/consensus/transactions.md");
    pub const BLOCK_IDENTIFIERS: &str = include_str!("../resources/eips/block-identifiers.md");
    pub const PROVIDER_SETUP: &str = include_str!("../resources/provider/setup.md");
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct LookupTypeRequest {
    #[schemars(description = "Type name to search for (e.g., 'TxEip1559', 'BlockId', 'Provider')")]
    type_name: String,
}

/// The alloy MCP server handler
#[derive(Clone)]
struct AlloyMcpServer {
    /// Static resources indexed by URI
    resources: HashMap<String, StaticResource>,
    /// Tool router for handling tool calls (read by generated macro code)
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[derive(Clone)]
struct StaticResource {
    uri: String,
    name: String,
    description: String,
    mime_type: String,
    content: String,
}

impl AlloyMcpServer {
    fn new() -> Self {
        let mut resources = HashMap::new();

        let tx_resource = StaticResource {
            uri: "alloy://consensus/transactions".to_string(),
            name: "Transaction Types".to_string(),
            description:
                "Guide to alloy transaction types: TxLegacy, TxEip1559, TxEip4844, TxEnvelope, etc."
                    .to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::TRANSACTIONS.to_string(),
        };
        resources.insert(tx_resource.uri.clone(), tx_resource);

        let block_id_resource = StaticResource {
            uri: "alloy://eips/block-identifiers".to_string(),
            name: "Block Identifier Types".to_string(),
            description: "Guide to BlockId, BlockNumberOrTag, HashOrNumber, and related types."
                .to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::BLOCK_IDENTIFIERS.to_string(),
        };
        resources.insert(block_id_resource.uri.clone(), block_id_resource);

        let provider_resource = StaticResource {
            uri: "alloy://provider/setup".to_string(),
            name: "Provider Setup".to_string(),
            description:
                "Guide to setting up alloy providers: ProviderBuilder, wallets, WebSocket, layers."
                    .to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::PROVIDER_SETUP.to_string(),
        };
        resources.insert(provider_resource.uri.clone(), provider_resource);

        Self {
            resources,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl AlloyMcpServer {
    /// Look up information about an alloy type by name.
    /// Useful when you know roughly what you're looking for but not the exact path.
    #[tool(description = "Look up alloy type information by name (fuzzy search)")]
    fn lookup_type(
        &self,
        Parameters(LookupTypeRequest { type_name }): Parameters<LookupTypeRequest>,
    ) -> String {
        let type_lower = type_name.to_lowercase();

        let mut matches = Vec::new();

        for resource in self.resources.values() {
            if resource.content.to_lowercase().contains(&type_lower) {
                matches.push(format!(
                    "Found in: {} ({})\n  → {}\n",
                    resource.name, resource.uri, resource.description
                ));
            }
        }

        if matches.is_empty() {
            format!(
                "No resources found mentioning '{}'. Try:\n\
                 - alloy://consensus/transactions - for transaction types\n\
                 - alloy://eips/block-identifiers - for block ID types\n\
                 - alloy://provider/setup - for provider setup",
                type_name
            )
        } else {
            format!(
                "Type '{}' mentioned in:\n\n{}",
                type_name,
                matches.join("\n")
            )
        }
    }
}

impl ServerHandler for AlloyMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting alloy-mcp server");

    let server = AlloyMcpServer::new();
    let transport = (stdin(), stdout());

    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
