# Alloy Provider Fillers

## Quick Reference

| Filler | What It Fills | When You Need It |
|--------|---------------|------------------|
| `GasFiller` | `gas_limit`, `max_fee_per_gas`, `max_priority_fee_per_gas` | Almost always (default) |
| `BlobGasFiller` | `max_fee_per_blob_gas` | Blob (EIP-4844) transactions |
| `NonceFiller` | `nonce` | Almost always (default) |
| `ChainIdFiller` | `chain_id` | Almost always (default) |
| `WalletFiller` | Signs transaction | When sending transactions |

## Default Fillers

`ProviderBuilder::new()` includes recommended fillers automatically:
- `GasFiller` — estimates gas price and limit
- `NonceFiller` — manages nonce
- `ChainIdFiller` — sets chain ID

```rust
// These two are equivalent:
let provider = ProviderBuilder::new()
    .connect(url)
    .await?;

// Explicit form:
let provider = ProviderBuilder::new()
    .filler(GasFiller)
    .filler(NonceFiller::default())
    .filler(ChainIdFiller::default())
    .connect(url)
    .await?;
```

## Disabling Default Fillers

For low-level control where you manage gas, nonce, and chain ID yourself:

```rust
let provider = ProviderBuilder::new()
    .disable_recommended_fillers()
    .connect(url)
    .await?;

// Now you must set all fields manually:
let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_000_000_000_000_000_000u64))
    .with_nonce(0)
    .with_chain_id(1)
    .with_gas_limit(21000)
    .with_max_fee_per_gas(30_000_000_000)
    .with_max_priority_fee_per_gas(1_000_000_000);
```

## Individual Fillers

### GasFiller

Estimates gas parameters by calling the node.

```rust
use alloy::providers::fillers::GasFiller;

let provider = ProviderBuilder::new()
    .filler(GasFiller)
    .connect(url)
    .await?;
```

### BlobGasFiller

Required for EIP-4844 blob transactions. Not included in default fillers.

```rust
use alloy::providers::fillers::BlobGasFiller;

let provider = ProviderBuilder::new()
    .filler(BlobGasFiller)
    .connect(url)
    .await?;
```

### NonceFiller

Manages transaction nonces. Uses `SimpleNonceManager` by default.

```rust
use alloy::providers::fillers::{NonceFiller, SimpleNonceManager};

// Default (SimpleNonceManager — caches and increments locally)
let provider = ProviderBuilder::new()
    .filler(NonceFiller::default())
    .connect(url)
    .await?;
```

### ChainIdFiller

Fetches and caches chain ID from the node.

```rust
use alloy::providers::fillers::ChainIdFiller;

let provider = ProviderBuilder::new()
    .filler(ChainIdFiller::default())
    .connect(url)
    .await?;
```

## Combining Fillers with Wallet

```rust
use alloy::providers::ProviderBuilder;
use alloy::network::EthereumWallet;

let provider = ProviderBuilder::new()
    .wallet(wallet) // adds WalletFiller internally
    .connect(url)
    .await?;

// Or with custom fillers + wallet:
let provider = ProviderBuilder::new()
    .disable_recommended_fillers()
    .filler(GasFiller)
    .filler(NonceFiller::default())
    .filler(ChainIdFiller::default())
    .filler(BlobGasFiller)
    .wallet(wallet)
    .connect(url)
    .await?;
```

## Complex FillProvider Type Aliases

When you need to name the provider type (e.g., in struct fields), the full type can be verbose:

```rust
use alloy::providers::{
    fillers::{FillProvider, GasFiller, NonceFiller, ChainIdFiller, BlobGasFiller, JoinFill},
    RootProvider,
};

// The type from ProviderBuilder::new() is roughly:
type DefaultProvider = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<Identity, GasFiller>,
            NonceFiller,
        >,
        ChainIdFiller,
    >,
    RootProvider,
>;

// Tip: use `impl Provider` in function signatures to avoid spelling this out:
async fn do_something(provider: impl Provider) -> eyre::Result<()> {
    // ...
    Ok(())
}
```

## Common Mistakes

1. **Forgetting BlobGasFiller for blob transactions** — blob transactions will fail without `max_fee_per_blob_gas` set
2. **Disabling fillers without setting fields** — if you `disable_recommended_fillers()`, you must set nonce, gas, and chain_id manually
3. **Nonce conflicts with multiple concurrent transactions** — `SimpleNonceManager` caches locally; for high-throughput, manage nonces externally
4. **Trying to name the provider type** — use `impl Provider` trait bounds instead of spelling out `FillProvider<...>`
