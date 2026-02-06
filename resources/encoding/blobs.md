# Blob Transactions & Sidecars

## Quick Reference

| Type | Purpose | Crate |
|------|---------|-------|
| `SidecarBuilder` | Build blob sidecars from data | `alloy-consensus` |
| `SimpleCoder` | Simple blob encoding (one blob per data chunk) | `alloy-consensus` |
| `BlobTransactionSidecar` | Sidecar with blobs, commitments, proofs | `alloy-consensus` |
| `TxEip4844` | Blob transaction (without sidecar) | `alloy-consensus` |
| `TxEip4844WithSidecar` | Blob transaction with attached sidecar | `alloy-consensus` |

## Building Blob Sidecars

### Using SidecarBuilder with SimpleCoder

The easiest way to create blob sidecars from arbitrary data:

```rust
use alloy::consensus::SidecarBuilder;
use alloy::consensus::encode::SimpleCoder;

// Encode data into blobs
let data: &[u8] = b"Hello, blobs!";
let sidecar = SidecarBuilder::<SimpleCoder>::from_slice(data).build()?;

// sidecar contains:
// - blobs: Vec<Blob>           — the actual blob data
// - commitments: Vec<Bytes48>  — KZG commitments
// - proofs: Vec<Bytes48>       — KZG proofs
```

### Building with Multiple Data Chunks

```rust
use alloy::consensus::SidecarBuilder;
use alloy::consensus::encode::SimpleCoder;

let mut builder = SidecarBuilder::<SimpleCoder>::new();

// Ingest data (may span multiple blobs)
builder.ingest(b"first chunk of data");
builder.ingest(b"second chunk of data");

let sidecar = builder.build()?;
```

## Sending Blob Transactions

### Via TransactionRequest

```rust
use alloy::rpc::types::TransactionRequest;
use alloy::consensus::SidecarBuilder;
use alloy::consensus::encode::SimpleCoder;

// Build sidecar
let sidecar = SidecarBuilder::<SimpleCoder>::from_slice(blob_data).build()?;

// Create transaction with sidecar
let tx = TransactionRequest::default()
    .with_to(rollup_inbox_address)
    .with_max_fee_per_blob_gas(1_000_000_000)
    .with_blob_sidecar(sidecar);

let pending = provider.send_transaction(tx).await?;
let receipt = pending.get_receipt().await?;
```

### Low-Level Construction

```rust
use alloy::consensus::{TxEip4844, TxEip4844WithSidecar, BlobTransactionSidecar};

// Build the sidecar
let sidecar = SidecarBuilder::<SimpleCoder>::from_slice(data).build()?;

// Get versioned hashes from the sidecar
let versioned_hashes = sidecar.versioned_hashes().collect();

// Build the transaction
let tx = TxEip4844 {
    chain_id: 1,
    nonce: 0,
    max_fee_per_gas: 30_000_000_000,
    max_priority_fee_per_gas: 1_000_000_000,
    max_fee_per_blob_gas: 1_000_000_000,
    gas_limit: 21000,
    to: rollup_inbox,
    value: U256::ZERO,
    input: Bytes::new(),
    access_list: AccessList::default(),
    blob_versioned_hashes: versioned_hashes,
};

// Combine with sidecar
let tx_with_sidecar = TxEip4844WithSidecar {
    tx,
    sidecar,
};
```

## Blob Constants

```rust
// Each blob is 128 KiB (131072 bytes)
const BYTES_PER_BLOB: usize = 131072;

// Max blobs per transaction (Cancun upgrade)
const MAX_BLOBS_PER_BLOCK: usize = 6;

// Usable data per blob with SimpleCoder is slightly less due to encoding overhead
```

## SidecarCoder Trait

`SimpleCoder` is the default implementation. The `SidecarCoder` trait can be implemented for custom encoding:

```rust
use alloy::consensus::encode::SidecarCoder;

// SimpleCoder — straightforward encoding, one logical chunk per blob
// Custom coders can implement compression or erasure coding
```

## Common Mistakes

1. **Forgetting KZG trusted setup** — blob commitments/proofs require the KZG ceremony data; `SidecarBuilder::build()` handles this but may fail if the setup isn't available
2. **Exceeding blob limit** — a single transaction can contain at most 6 blobs; larger data must be split across multiple transactions
3. **Missing `max_fee_per_blob_gas`** — blob transactions MUST set this field; without it, the transaction is invalid
4. **Confusing `TxEip4844` and `TxEip4844WithSidecar`** — the sidecar is needed for submission but stripped during consensus; use `WithSidecar` variant when sending
5. **Not using BlobGasFiller** — if using a provider, add `BlobGasFiller` to auto-fill `max_fee_per_blob_gas`
