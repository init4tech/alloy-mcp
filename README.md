# alloy-mcp

A minimal MCP (Model Context Protocol) server providing type context for [alloy.rs](https://alloy.rs).

## Purpose

Help LLMs correctly use alloy library types by providing:
- Curated documentation for commonly-confused types
- Quick reference guides for choosing the right type
- Code examples and common patterns
- Gotchas and common mistakes

## Resources

The server exposes these resources:

| URI | Description |
|-----|-------------|
| `alloy://consensus/transactions` | Transaction types: TxLegacy, TxEip1559, TxEip4844, envelopes |
| `alloy://eips/block-identifiers` | Block ID types: BlockId, BlockNumberOrTag, HashOrNumber |
| `alloy://provider/setup` | Provider setup: ProviderBuilder, wallets, WebSocket |

## Tools

| Tool | Description |
|------|-------------|
| `lookup_type` | Fuzzy search for type information across resources |

## Building

Requires Rust 1.75+ and the rmcp crate.

```bash
cargo build --release
```

## Running

The server uses stdio transport:

```bash
./target/release/alloy-mcp
```

## Configuration (Claude Desktop / VS Code)

Add to your MCP config:

```json
{
  "mcpServers": {
    "alloy": {
      "command": "/path/to/alloy-mcp"
    }
  }
}
```

## Development Status

- [x] Design document
- [x] Resource content (Tier 1 types)
- [x] MCP server skeleton
- [ ] Build and test
- [ ] Add more resources (Tier 2/3 types)
- [ ] Add prompts for common patterns

## License

MIT
