# Alloy Provider Setup

## Quick Reference

| Goal | Approach |
|------|----------|
| Simple HTTP provider | `ProviderBuilder::new().connect(url)` |
| With wallet/signer | `.wallet(wallet)` on builder |
| WebSocket (subscriptions) | `.connect_ws(url)` |
| Custom network (Optimism, etc) | `ProviderBuilder::new_with_network::<N>()` |
| Low-level control | `RootProvider::new_http(url)` |

## Basic Setup

### HTTP Provider (Most Common)

```rust
use alloy::providers::{Provider, ProviderBuilder};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Recommended: use ProviderBuilder
    let provider = ProviderBuilder::new()
        .connect("https://eth.llamarpc.com")
        .await?;
    
    // Now you can make calls
    let block_number = provider.get_block_number().await?;
    let chain_id = provider.get_chain_id().await?;
    
    Ok(())
}
```

### With a Wallet (For Sending Transactions)

```rust
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::network::EthereumWallet;

let signer: PrivateKeySigner = "0xac0974...".parse()?;
let wallet = EthereumWallet::from(signer);

let provider = ProviderBuilder::new()
    .wallet(wallet)
    .connect("https://eth.llamarpc.com")
    .await?;

// Can now send transactions
let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_000_000_000_000_000_000u64));

let pending = provider.send_transaction(tx).await?;
let receipt = pending.get_receipt().await?;
```

### WebSocket Provider (For Subscriptions)

```rust
use alloy::providers::ProviderBuilder;

let provider = ProviderBuilder::new()
    .connect_ws("wss://eth.llamarpc.com/ws")
    .await?;

// Subscribe to new blocks
let sub = provider.subscribe_blocks().await?;
let mut stream = sub.into_stream();

while let Some(block) = stream.next().await {
    println!("New block: {}", block.header.number);
}
```

## Network-Specific Providers

For non-Ethereum chains (Optimism, Arbitrum, etc):

```rust
use alloy::providers::ProviderBuilder;
use op_alloy::network::Optimism;

// Optimism provider
let provider = ProviderBuilder::new_with_network::<Optimism>()
    .connect("https://mainnet.optimism.io")
    .await?;
```

## Provider Layers (Advanced)

Providers can be composed with layers for middleware-like behavior:

```rust
use alloy::providers::{ProviderBuilder, layers::*};

let provider = ProviderBuilder::new()
    .layer(RetryLayer::new(3))           // Retry failed requests
    .layer(TimeoutLayer::new(Duration::from_secs(10)))
    .connect("https://eth.llamarpc.com")
    .await?;
```

## Fillers

Fillers automatically populate transaction fields:

```rust
let provider = ProviderBuilder::new()
    .filler(GasFiller)       // Estimates gas
    .filler(NonceFiller)     // Sets nonce
    .filler(ChainIdFiller)   // Sets chain_id
    .wallet(wallet)
    .connect(url)
    .await?;

// Transaction will have gas, nonce, chain_id filled automatically
let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_ether));

provider.send_transaction(tx).await?;
```

**Note:** `ProviderBuilder::new()` includes recommended fillers by default.

## Low-Level: RootProvider

For direct control without builder niceties:

```rust
use alloy::providers::RootProvider;
use alloy::network::Ethereum;

let provider = RootProvider::<Ethereum>::new_http(
    "https://eth.llamarpc.com".parse()?
);
```

## The `Provider` Trait

All providers implement the `Provider` trait:

```rust
use alloy::providers::Provider;

async fn do_stuff<P: Provider>(provider: P) -> eyre::Result<()> {
    let block = provider.get_block_number().await?;
    let balance = provider.get_balance(address).await?;
    Ok(())
}
```

### Key Methods

| Method | Description |
|--------|-------------|
| `get_block_number()` | Latest block number |
| `get_chain_id()` | Chain ID |
| `get_balance(addr)` | ETH balance |
| `get_block(id)` | Block by hash/number |
| `get_transaction(hash)` | Transaction by hash |
| `get_transaction_receipt(hash)` | Receipt by tx hash |
| `call(tx)` | Simulate transaction (eth_call) |
| `send_transaction(tx)` | Send transaction |
| `send_raw_transaction(bytes)` | Send raw signed tx |

## Common Patterns

### Retry on Failure

```rust
let provider = ProviderBuilder::new()
    .connect("https://eth.llamarpc.com")
    .await?;

// Built-in retry for transient errors
let block = provider.get_block_number().await?;
```

### Multiple RPC Endpoints

```rust
// Use a load-balanced RPC or implement your own fallback
let urls = ["https://eth1.example.com", "https://eth2.example.com"];
// ... implement fallback logic
```

### Local Development (Anvil)

```rust
use alloy::node_bindings::Anvil;

// Start local Anvil instance
let anvil = Anvil::new().spawn();

let provider = ProviderBuilder::new()
    .connect(&anvil.endpoint())
    .await?;
```

## Common Mistakes

1. **Forgetting `await` on `connect()`** — it's async!
2. **Using HTTP for subscriptions** — need WebSocket or IPC
3. **Not handling rate limits** — add retry logic
4. **Wrong network type** — use `new_with_network::<N>()` for non-Ethereum
5. **Missing wallet for `send_transaction`** — need `.wallet()` on builder
