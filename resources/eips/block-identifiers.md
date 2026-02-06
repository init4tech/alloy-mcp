# Alloy Block Identifier Types

## Quick Reference

| Type | Use Case | Example |
|------|----------|---------|
| `BlockId` | RPC calls accepting block ref | `eth_getBalance(..., block)` |
| `BlockNumberOrTag` | Block number or named tag | `latest`, `pending`, `12345` |
| `HashOrNumber` | Block hash or number | Query by either |
| `NumHash` | Both number AND hash | When you have both |

All types are in `alloy_eips::eip1898`.

## `BlockId` — The Universal Block Reference

Used in most RPC methods that accept a block parameter.

```rust
use alloy_eips::eip1898::BlockId;

// By number
let block = BlockId::number(12345678);

// By hash
let block = BlockId::hash(block_hash);

// By tag
let block = BlockId::latest();
let block = BlockId::pending();
let block = BlockId::earliest();
let block = BlockId::finalized();
let block = BlockId::safe();
```

### Using with Provider

```rust
// Get balance at specific block
let balance = provider.get_balance(address).block_id(BlockId::number(12345)).await?;

// Get balance at latest
let balance = provider.get_balance(address).await?; // defaults to latest
```

## `BlockNumberOrTag` — Number or Named Tag

```rust
use alloy_eips::eip1898::BlockNumberOrTag;

let block = BlockNumberOrTag::Number(12345678);
let block = BlockNumberOrTag::Latest;
let block = BlockNumberOrTag::Pending;
let block = BlockNumberOrTag::Earliest;
let block = BlockNumberOrTag::Finalized;
let block = BlockNumberOrTag::Safe;

// Parse from string
let block: BlockNumberOrTag = "latest".parse()?;
let block: BlockNumberOrTag = "0xbc614e".parse()?; // hex number
```

### Tags Explained

| Tag | Meaning |
|-----|---------|
| `latest` | Most recent mined block |
| `pending` | Pending block (transactions in mempool) |
| `earliest` | Genesis block (block 0) |
| `finalized` | Most recent finalized block (PoS) |
| `safe` | Most recent safe block (PoS) |

## `HashOrNumber` — Either Hash or Number

Useful when you might have either identifier.

```rust
use alloy_eips::eip1898::HashOrNumber;

let id = HashOrNumber::Number(12345678);
let id = HashOrNumber::Hash(block_hash);

// Convert from BlockId
let hash_or_num: HashOrNumber = block_id.into();
```

## `NumHash` — Number AND Hash Together

When you have both and want to keep them paired.

```rust
use alloy_eips::eip1898::NumHash;

let nh = NumHash {
    number: 12345678,
    hash: block_hash,
};

// Often returned by RPC responses
let header = provider.get_header(BlockId::latest()).await?;
let num_hash = NumHash::new(header.number, header.hash);
```

## `RpcBlockHash` — Hash with Optional Block Number

For RPC calls that accept a block hash with optional number hint.

```rust
use alloy_eips::eip1898::RpcBlockHash;

let rpc_hash = RpcBlockHash {
    block_hash: hash,
    require_canonical: Some(true), // only accept if in canonical chain
};
```

## Common Patterns

### Provider Methods with Block Parameter

```rust
// Methods that take BlockId
provider.get_balance(addr).block_id(BlockId::number(100)).await?;
provider.get_code(addr).block_id(BlockId::latest()).await?;
provider.get_storage_at(addr, slot).block_id(BlockId::finalized()).await?;

// Getting a block
provider.get_block(BlockId::hash(hash)).await?;
provider.get_block(BlockId::number(12345)).await?;
provider.get_block(BlockId::latest()).await?;
```

### Converting Between Types

```rust
// BlockNumberOrTag -> BlockId
let block_id: BlockId = BlockNumberOrTag::Latest.into();

// Number -> BlockId
let block_id = BlockId::number(12345);

// Hash -> BlockId  
let block_id = BlockId::hash(hash);
```

## Common Mistakes

1. **Using raw u64 instead of `BlockId`** — won't compile, use `BlockId::number(n)`
2. **Forgetting about `pending`** — some methods behave differently with pending
3. **Not handling reorgs** — blocks can change, use `finalized` for certainty
4. **Mixing up `safe` vs `finalized`** — `finalized` is stronger guarantee
