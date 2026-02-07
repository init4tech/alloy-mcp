# Recovered Transactions & Type Aliases

## Quick Reference

| Type | Purpose | Crate |
|------|---------|-------|
| `Recovered<T>` | Transaction with recovered sender address | `alloy-consensus` |
| `TxEnvelope` | Signed transaction envelope (any type) | `alloy-consensus` |
| `TypedTransaction` | Unsigned transaction enum | `alloy-consensus` |

## Recovered\<T\>

`Recovered<T>` wraps a signed transaction with its recovered sender address, avoiding repeated `ecrecover` operations.

```rust
use alloy::consensus::Recovered;
use alloy::consensus::TxEnvelope;

// Recover sender from a signed transaction
let recovered: Recovered<TxEnvelope> = envelope.try_into_recovered()?;

// Access the sender
let sender = recovered.signer();

// Access the inner transaction
let envelope: &TxEnvelope = recovered.inner();
```

## Creating Recovered Transactions

### From a Signed Envelope

```rust
use alloy::consensus::{Recovered, TxEnvelope};

// Recover sender by performing ecrecover
let recovered = Recovered::try_from_signed(signed_tx)?;
let sender = recovered.signer();

// If you already know the sender (skips ecrecover)
let recovered = Recovered::new_unchecked(signed_tx, known_sender);
```

### From Network Responses

When receiving transactions from RPC, the sender is typically already known:

```rust
// Transaction from eth_getTransactionByHash includes `from` field
let tx = provider.get_transaction_by_hash(tx_hash).await?;
// tx.from is already populated
```

## Custom Transaction Type Aliases

Projects often define type aliases for their transaction types:

```rust
use alloy::consensus::{TxEnvelope, Recovered};

// Common alias for recovered transactions
type RecoveredTx = Recovered<TxEnvelope>;
```

### Custom Transaction Envelopes

For networks with custom transaction types:

```rust
use alloy::consensus::{Signed, TxEip1559, TxEip4844, TxLegacy};

// You can define your own transaction enum
enum MyTxType {
    Legacy(Signed<TxLegacy>),
    Eip1559(Signed<TxEip1559>),
    Eip4844(Signed<TxEip4844>),
    Custom(Signed<MyCustomTx>),
}
```

## Converting Between Types

```rust
use alloy::consensus::{TxEnvelope, TypedTransaction};

// TxEnvelope (signed) -> access inner unsigned transaction
match &envelope {
    TxEnvelope::Eip1559(signed) => {
        let tx: &TxEip1559 = signed.tx();
        let signature = signed.signature();
        let hash = signed.hash();
    }
    _ => {}
}

// Recovered -> strip recovery info
let (envelope, sender) = recovered.into_parts();
```

## DataCompat Trait (alloy ↔ reth)

When working with reth (Rust Ethereum client), the `DataCompat` trait provides conversions:

```rust
// This pattern appears in projects that bridge alloy and reth types
// The trait provides .compat() method for type conversion

// alloy TxEnvelope -> reth transaction type
// let reth_tx = alloy_tx.compat();

// reth transaction type -> alloy TxEnvelope
// let alloy_tx = reth_tx.compat();
```

Note: `DataCompat` is defined in bridge crates (e.g., `reth-primitives-traits`), not in alloy itself.

## Common Mistakes

1. **Redundant ecrecover** — if you have a `Recovered<T>`, use `.signer()` instead of recovering again; ecrecover is expensive
2. **Using `new_unchecked` with wrong sender** — only use if you're certain of the sender; otherwise use `try_from_signed` which verifies
3. **Forgetting that Recovered wraps the full signed tx** — `Recovered<TxEnvelope>` contains the signature; you can get both signer and signature
4. **Type confusion between TxEnvelope and TypedTransaction** — `TxEnvelope` is signed (with signature), `TypedTransaction` is unsigned
