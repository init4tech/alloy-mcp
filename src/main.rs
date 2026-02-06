//! Alloy MCP Server
//!
//! A minimal MCP server providing type context for alloy.rs.
//! Helps LLMs correctly use alloy library types.

use rmcp::{
    handler::server::wrapper::Json,
    model::{
        resource::{Resource, ResourceContents, ResourceTemplate},
        GetResourceResult, ListResourceTemplatesResult, ListResourcesResult,
    },
    schemars, tool, Error, ServerHandler, ServiceExt,
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

/// The alloy MCP server handler
#[derive(Clone)]
struct AlloyMcpServer {
    /// Static resources indexed by URI
    resources: HashMap<String, StaticResource>,
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

        // Register consensus/transaction resource
        let tx_resource = StaticResource {
            uri: "alloy://consensus/transactions".to_string(),
            name: "Transaction Types".to_string(),
            description: "Guide to alloy transaction types: TxLegacy, TxEip1559, TxEip4844, TxEnvelope, etc.".to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::TRANSACTIONS.to_string(),
        };
        resources.insert(tx_resource.uri.clone(), tx_resource);

        // Register eips/block-identifiers resource
        let block_id_resource = StaticResource {
            uri: "alloy://eips/block-identifiers".to_string(),
            name: "Block Identifier Types".to_string(),
            description: "Guide to BlockId, BlockNumberOrTag, HashOrNumber, and related types.".to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::BLOCK_IDENTIFIERS.to_string(),
        };
        resources.insert(block_id_resource.uri.clone(), block_id_resource);

        // Register provider/setup resource
        let provider_resource = StaticResource {
            uri: "alloy://provider/setup".to_string(),
            name: "Provider Setup".to_string(),
            description: "Guide to setting up alloy providers: ProviderBuilder, wallets, WebSocket, layers.".to_string(),
            mime_type: "text/markdown".to_string(),
            content: resources::PROVIDER_SETUP.to_string(),
        };
        resources.insert(provider_resource.uri.clone(), provider_resource);

        Self { resources }
    }
}

#[tool(tool_box)]
impl AlloyMcpServer {
    /// Look up information about an alloy type by name.
    /// Useful when you know roughly what you're looking for but not the exact path.
    #[tool(description = "Look up alloy type information by name (fuzzy search)")]
    fn lookup_type(
        &self,
        #[tool(param, description = "Type name to search for (e.g., 'TxEip1559', 'BlockId', 'Provider')")] 
        type_name: String,
    ) -> Result<String, Error> {
        let type_lower = type_name.to_lowercase();
        
        // Search through resources for mentions of the type
        let mut matches = Vec::new();
        
        for resource in self.resources.values() {
            if resource.content.to_lowercase().contains(&type_lower) {
                matches.push(format!(
                    "Found in: {} ({})\n  → {}\n",
                    resource.name,
                    resource.uri,
                    resource.description
                ));
            }
        }
        
        if matches.is_empty() {
            Ok(format!(
                "No resources found mentioning '{}'. Try:\n\
                 - alloy://consensus/transactions - for transaction types\n\
                 - alloy://eips/block-identifiers - for block ID types\n\
                 - alloy://provider/setup - for provider setup",
                type_name
            ))
        } else {
            Ok(format!(
                "Type '{}' mentioned in:\n\n{}",
                type_name,
                matches.join("\n")
            ))
        }
    }
}

impl ServerHandler for AlloyMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            name: "alloy-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ..Default::default()
        }
    }

    async fn list_resources(&self) -> Result<ListResourcesResult, Error> {
        let resources: Vec<Resource> = self
            .resources
            .values()
            .map(|r| Resource {
                uri: r.uri.clone(),
                name: Some(r.name.clone()),
                description: Some(r.description.clone()),
                mime_type: Some(r.mime_type.clone()),
                ..Default::default()
            })
            .collect();

        Ok(ListResourcesResult {
            resources,
            ..Default::default()
        })
    }

    async fn get_resource(&self, uri: String) -> Result<GetResourceResult, Error> {
        let resource = self.resources.get(&uri).ok_or_else(|| {
            Error::ResourceNotFound(format!("Resource not found: {}", uri))
        })?;

        Ok(GetResourceResult {
            contents: vec![ResourceContents::Text {
                uri: resource.uri.clone(),
                mime_type: Some(resource.mime_type.clone()),
                text: resource.content.clone(),
            }],
        })
    }

    async fn list_resource_templates(&self) -> Result<ListResourceTemplatesResult, Error> {
        // We could add templates for dynamic lookups later
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![
                ResourceTemplate {
                    uri_template: "alloy://type/{type_name}".to_string(),
                    name: Some("Type Lookup".to_string()),
                    description: Some("Look up a specific alloy type by name".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting alloy-mcp server");

    // Create server and run over stdio
    let server = AlloyMcpServer::new();
    let transport = (stdin(), stdout());
    
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
