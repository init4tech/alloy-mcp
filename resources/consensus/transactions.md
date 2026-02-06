# Alloy Transaction Types

## Quick Reference: Which Type Do I Need?

| Use Case | Type | Crate |
|----------|------|-------|
| Build any transaction (recommended) | `TransactionRequest` | `alloy-rpc-types` |
| Legacy transaction (pre-EIP-1559) | `TxLegacy` | `alloy-consensus` |
| EIP-1559 transaction (most common) | `TxEip1559` | `alloy-consensus` |
| Transaction with access list | `TxEip2930` | `alloy-consensus` |
| Blob transaction (L2 data) | `TxEip4844` | `alloy-consensus` |
| Account abstraction delegation | `TxEip7702` | `alloy-consensus` |
| Signed transaction (any type) | `TxEnvelope` | `alloy-consensus` |
| Match on transaction type | `TypedTransaction` | `alloy-consensus` |

## Transaction Variants Explained

### `TxLegacy` — Legacy Transactions
Pre-EIP-1559 transactions with a single `gas_price` field.

```rust
use alloy_consensus::TxLegacy;

let tx = TxLegacy {
    chain_id: Some(1),
    nonce: 0,
    gas_price: 20_000_000_000, // 20 gwei
    gas_limit: 21000,
    to: TxKind::Call(recipient),
    value: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
    input: Bytes::new(),
};
```

**When to use:** Only for chains that don't support EIP-1559, or for specific legacy compatibility.

### `TxEip1559` — EIP-1559 Transactions (Most Common)
Modern transactions with `max_fee_per_gas` and `max_priority_fee_per_gas`.

```rust
use alloy_consensus::TxEip1559;

let tx = TxEip1559 {
    chain_id: 1,
    nonce: 0,
    max_fee_per_gas: 30_000_000_000,      // max you're willing to pay
    max_priority_fee_per_gas: 1_000_000_000, // tip to validator
    gas_limit: 21000,
    to: TxKind::Call(recipient),
    value: U256::from(1_000_000_000_000_000_000u64),
    input: Bytes::new(),
    access_list: AccessList::default(),
};
```

**When to use:** Default choice for Ethereum mainnet and most EVM chains.

### `TxEip4844` — Blob Transactions
For posting data blobs to L1 (used by rollups).

```rust
use alloy_consensus::{TxEip4844, BlobTransactionSidecar};

let tx = TxEip4844 {
    chain_id: 1,
    nonce: 0,
    max_fee_per_gas: 30_000_000_000,
    max_priority_fee_per_gas: 1_000_000_000,
    max_fee_per_blob_gas: 1_000_000_000,  // EIP-4844 specific
    gas_limit: 21000,
    to: rollup_inbox,
    value: U256::ZERO,
    input: Bytes::new(),
    access_list: AccessList::default(),
    blob_versioned_hashes: vec![...],     // KZG commitment hashes
};

// Blob data is sent separately as a sidecar
let sidecar = BlobTransactionSidecar { ... };
```

**When to use:** Submitting data to L1 for rollups. Requires KZG commitments.

## Envelope Types: Signed Transactions

### `TxEnvelope` — The Universal Signed Transaction
A signed transaction of any type. Use this when receiving/sending transactions over the network.

```rust
use alloy_consensus::TxEnvelope;

// Match on transaction type
match tx_envelope {
    TxEnvelope::Legacy(signed) => { /* TxLegacy + signature */ }
    TxEnvelope::Eip1559(signed) => { /* TxEip1559 + signature */ }
    TxEnvelope::Eip2930(signed) => { /* TxEip2930 + signature */ }
    TxEnvelope::Eip4844(signed) => { /* TxEip4844 + signature */ }
    TxEnvelope::Eip7702(signed) => { /* TxEip7702 + signature */ }
}
```

### `Signed<T>` — Generic Signed Wrapper
Wraps any transaction with its signature and computed hash.

```rust
use alloy_consensus::Signed;

let signed: Signed<TxEip1559> = tx.into_signed(signature);
let hash = signed.hash();       // transaction hash
let sig = signed.signature();   // the signature
let inner = signed.tx();        // the inner TxEip1559
```

## Common Patterns

### Building a Transaction (Recommended Way)
Use `TransactionRequest` from `alloy-rpc-types` — it handles type selection automatically:

```rust
use alloy::rpc::types::TransactionRequest;

let request = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_000_000_000_000_000_000u64))
    .with_max_fee_per_gas(30_000_000_000)
    .with_max_priority_fee_per_gas(1_000_000_000);

// Send via provider — it fills in nonce, gas, etc.
let pending = provider.send_transaction(request).await?;
```

### Signing a Transaction
Use the `SignableTransaction` trait:

```rust
use alloy_consensus::SignableTransaction;

let signature = signer.sign_hash(&tx.signature_hash()).await?;
let signed = tx.into_signed(signature);
```

## Type Hierarchy

```
TransactionRequest (building)
    ↓ (filled by provider)
TxEip1559 / TxLegacy / ... (unsigned)
    ↓ (signed)
Signed<TxEip1559> / ...
    ↓ (wrapped in envelope)
TxEnvelope (network transmission)
```

## Common Mistakes

1. **Using `TxLegacy` on EIP-1559 chains** — works but wastes gas
2. **Forgetting `chain_id`** — transaction will be rejected
3. **Confusing `TxEnvelope` with unsigned types** — envelope is always signed
4. **Not using `TransactionRequest`** — manual construction is error-prone
