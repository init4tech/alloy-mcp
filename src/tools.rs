use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

use crate::server::AlloyMcpServer;

/// A section extracted from a resource markdown file.
struct Section {
    /// The resource URI this section belongs to.
    uri: String,
    /// The resource name.
    resource_name: String,
    /// The section heading (e.g., "## PrivateKeySigner").
    heading: String,
    /// The full text content of the section.
    content: String,
}

/// Parse a resource's markdown content into sections split on `##` headings.
fn parse_sections(uri: &str, resource_name: &str, content: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_heading = String::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            // Flush previous section
            if !current_heading.is_empty() || !current_lines.is_empty() {
                let heading = if current_heading.is_empty() {
                    "(intro)".to_string()
                } else {
                    current_heading.clone()
                };
                let text = current_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    sections.push(Section {
                        uri: uri.to_string(),
                        resource_name: resource_name.to_string(),
                        heading,
                        content: text,
                    });
                }
            }
            current_heading = line.to_string();
            current_lines.clear();
            current_lines.push(line);
        } else {
            current_lines.push(line);
        }
    }

    // Flush last section
    if !current_lines.is_empty() {
        let heading = if current_heading.is_empty() {
            "(intro)".to_string()
        } else {
            current_heading.clone()
        };
        let text = current_lines.join("\n").trim().to_string();
        if !text.is_empty() {
            sections.push(Section {
                uri: uri.to_string(),
                resource_name: resource_name.to_string(),
                heading,
                content: text,
            });
        }
    }

    sections
}

/// Score how well a section matches a query. Higher is better.
/// Returns 0 for no match.
fn score_section(section: &Section, query: &str) -> u32 {
    let query_lower = query.to_lowercase();
    let heading_lower = section.heading.to_lowercase();
    let content_lower = section.content.to_lowercase();

    // Exact type name in heading (strongest signal)
    if heading_lower.contains(&query_lower) {
        return 100;
    }

    // Exact match in content as a word boundary (backtick-wrapped)
    let backtick_pattern = format!("`{}`", query_lower);
    if content_lower.contains(&backtick_pattern) {
        return 80;
    }

    // Case-insensitive exact match in content
    if content_lower.contains(&query_lower) {
        // Score by frequency - more mentions = more relevant
        let count = content_lower.matches(&query_lower).count();
        return 50 + (count as u32).min(30);
    }

    0
}

impl AlloyMcpServer {
    /// Get all sections from all resources.
    fn all_sections(&self) -> Vec<Section> {
        let mut sections = Vec::new();
        for resource in self.resources.values() {
            sections.extend(parse_sections(
                &resource.uri,
                &resource.name,
                &resource.content,
            ));
        }
        sections
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LookupTypeRequest {
    #[schemars(
        description = "Type name to search for (e.g., 'TxEip1559', 'BlockId', 'Address', 'PrivateKeySigner')"
    )]
    type_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchResourcesRequest {
    #[schemars(description = "Free-text query: type name, concept, or error message")]
    query: String,
    #[schemars(
        description = "Maximum number of results to return (default 5)",
        default = "SearchResourcesRequest::default_max_results"
    )]
    max_results: Option<u32>,
}

impl SearchResourcesRequest {
    fn default_max_results() -> u32 {
        5
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetResourceRequest {
    #[schemars(
        description = "Resource URI to fetch (e.g., 'alloy://consensus/transactions'). Pass 'list' to see all available URIs."
    )]
    uri: String,
}

#[tool_router]
impl AlloyMcpServer {
    pub fn create_tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::tool_router()
    }

    /// Look up information about an alloy type by name.
    /// Returns the most relevant documentation sections containing that type.
    #[tool(
        description = "Look up alloy type information by name. Returns relevant documentation sections with code examples."
    )]
    fn lookup_type(
        &self,
        Parameters(LookupTypeRequest { type_name }): Parameters<LookupTypeRequest>,
    ) -> String {
        let sections = self.all_sections();
        let mut scored: Vec<(u32, &Section)> = sections
            .iter()
            .filter_map(|s| {
                let score = score_section(s, &type_name);
                if score > 0 { Some((score, s)) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(3);

        if scored.is_empty() {
            let uris: Vec<String> = self
                .resources
                .values()
                .map(|r| format!("  - {} ({})", r.uri, r.name))
                .collect();
            format!(
                "No documentation found for '{}'. Available resources:\n{}",
                type_name,
                uris.join("\n")
            )
        } else {
            let mut result = format!("# Results for '{}'\n\n", type_name);
            for (score, section) in scored {
                result.push_str(&format!(
                    "---\n**{}** — {} (relevance: {})\nURI: {}\n\n{}\n\n",
                    section.heading.trim_start_matches('#').trim(),
                    section.resource_name,
                    score,
                    section.uri,
                    section.content
                ));
            }
            result
        }
    }

    /// Search across all alloy documentation resources.
    /// Accepts free-text queries and returns matching sections with context.
    #[tool(
        description = "Full-text search across all alloy documentation. Accepts type names, concepts, or error messages."
    )]
    fn search_resources(
        &self,
        Parameters(SearchResourcesRequest { query, max_results }): Parameters<
            SearchResourcesRequest,
        >,
    ) -> String {
        let max = max_results.unwrap_or(5) as usize;
        let sections = self.all_sections();

        // Split query into terms for multi-word matching
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored: Vec<(u32, &Section)> = sections
            .iter()
            .filter_map(|s| {
                let content_lower = s.content.to_lowercase();
                let heading_lower = s.heading.to_lowercase();

                // Score: full query match first, then individual terms
                let mut total_score = score_section(s, &query);

                // Bonus for individual term matches
                for term in &terms {
                    if heading_lower.contains(term) {
                        total_score += 10;
                    }
                    if content_lower.contains(term) {
                        total_score += 5;
                    }
                }

                if total_score > 0 {
                    Some((total_score, s))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(max);

        if scored.is_empty() {
            let uris: Vec<String> = self
                .resources
                .values()
                .map(|r| format!("  - {} — {}", r.uri, r.description))
                .collect();
            format!(
                "No results for '{}'. Available resources:\n{}",
                query,
                uris.join("\n")
            )
        } else {
            let mut result = format!("# Search results for '{}'\n\n", query);
            for (_score, section) in scored {
                // Truncate long sections to ~40 lines for readability
                let lines: Vec<&str> = section.content.lines().collect();
                let preview = if lines.len() > 40 {
                    format!(
                        "{}\n\n... ({} more lines, fetch full resource: {})",
                        lines[..40].join("\n"),
                        lines.len() - 40,
                        section.uri
                    )
                } else {
                    section.content.clone()
                };

                result.push_str(&format!(
                    "---\n**{}** — {}\nURI: {}\n\n{}\n\n",
                    section.heading.trim_start_matches('#').trim(),
                    section.resource_name,
                    section.uri,
                    preview
                ));
            }
            result
        }
    }

    /// Fetch a specific alloy documentation resource by URI.
    /// Pass 'list' to see all available resource URIs.
    #[tool(
        description = "Fetch a specific alloy documentation resource by URI. Pass uri='list' to see all available resources."
    )]
    fn get_resource(
        &self,
        Parameters(GetResourceRequest { uri }): Parameters<GetResourceRequest>,
    ) -> String {
        if uri == "list" {
            let mut entries: Vec<String> = self
                .resources
                .values()
                .map(|r| format!("- **{}**\n  URI: `{}`\n  {}", r.name, r.uri, r.description))
                .collect();
            entries.sort();
            return format!("# Available Resources\n\n{}", entries.join("\n\n"));
        }

        match self.resources.get(&uri) {
            Some(resource) => resource.content.clone(),
            None => {
                let uris: Vec<String> = self
                    .resources
                    .values()
                    .map(|r| format!("  {} — {}", r.uri, r.name))
                    .collect();
                format!(
                    "Resource not found: '{}'\n\nAvailable URIs:\n{}",
                    uri,
                    uris.join("\n")
                )
            }
        }
    }
}
