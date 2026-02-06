# RLP & EIP-2718 Encoding

## Quick Reference

| Trait | Purpose | Crate |
|-------|---------|-------|
| `Encodable2718` | Encode typed transactions (EIP-2718) | `alloy-eips` |
| `Decodable2718` | Decode typed transactions (EIP-2718) | `alloy-eips` |
| `RlpEncodable` / `RlpDecodable` | Raw RLP encoding/decoding | `alloy-rlp` |
| `Encodable` | RLP encoding trait | `alloy-rlp` |
| `Decodable` | RLP decoding trait | `alloy-rlp` |

## EIP-2718 Typed Transactions

EIP-2718 defines a standard envelope for typed transactions: `type_id || rlp(tx_data)`.

### Encoding a Transaction

```rust
use alloy::consensus::TxEnvelope;
use alloy::eips::eip2718::Encodable2718;

let envelope: TxEnvelope = /* signed transaction */;

// Encode to bytes
let mut buf = Vec::new();
envelope.encode_2718(&mut buf);

// Or get encoded bytes directly
let encoded: Vec<u8> = envelope.encoded_2718();

// Get the type ID
let type_id: u8 = envelope.type_id();
```

### Decoding a Transaction

```rust
use alloy::consensus::TxEnvelope;
use alloy::eips::eip2718::Decodable2718;

// Decode from bytes
let envelope = TxEnvelope::decode_2718(&mut &encoded_bytes[..])?;

// Match on type
match &envelope {
    TxEnvelope::Legacy(signed) => { /* type 0 */ }
    TxEnvelope::Eip1559(signed) => { /* type 2 */ }
    TxEnvelope::Eip4844(signed) => { /* type 3 */ }
    _ => { /* other types */ }
}
```

## EIP-2718 Type IDs

| Type ID | Transaction Type | EIP |
|---------|-----------------|-----|
| `0x00` (or no prefix) | Legacy | Pre-EIP-2718 |
| `0x01` | Access list | EIP-2930 |
| `0x02` | Fee market | EIP-1559 |
| `0x03` | Blob | EIP-4844 |
| `0x04` | EOA code delegation | EIP-7702 |

## Raw RLP Encoding

For lower-level RLP operations:

### Encoding

```rust
use alloy_rlp::Encodable;

let mut buf = Vec::new();

// Encode a single value
let value: u64 = 12345;
value.encode(&mut buf);

// Encode bytes
let data = vec![0xde, 0xad, 0xbe, 0xef];
data.encode(&mut buf);

// Get encoded length
let len = value.length();
```

### Decoding

```rust
use alloy_rlp::Decodable;

let value = u64::decode(&mut &encoded[..])?;
let data = Vec::<u8>::decode(&mut &encoded[..])?;
```

### Derive Macros

```rust
use alloy_rlp::{RlpEncodable, RlpDecodable};

#[derive(RlpEncodable, RlpDecodable)]
struct MyStruct {
    nonce: u64,
    value: U256,
    data: Vec<u8>,
}

let mut buf = Vec::new();
my_struct.encode(&mut buf);
let decoded = MyStruct::decode(&mut &buf[..])?;
```

## Sending Raw Transactions

```rust
use alloy::eips::eip2718::Encodable2718;

// Encode a signed transaction for raw sending
let raw_tx = signed_envelope.encoded_2718();

// Send via provider
let pending = provider.send_raw_transaction(&raw_tx).await?;
```

## Common Mistakes

1. **Confusing RLP and EIP-2718 encoding** — EIP-2718 prepends a type byte before RLP data; raw RLP doesn't have a type prefix
2. **Forgetting the type byte** — when manually constructing encoded transactions, legacy transactions have NO type prefix, but all others do
3. **Using wrong decode method** — use `Decodable2718` for full transaction envelopes, `Decodable` for raw RLP-only data
4. **Mutable reference for decoding** — `decode` takes `&mut &[u8]`, advancing the cursor; pass `&mut &bytes[..]`
