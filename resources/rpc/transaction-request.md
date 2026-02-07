# Alloy TransactionRequest Builder

## Quick Reference

| Method | Sets | Example |
|--------|------|---------|
| `.with_to(addr)` | Recipient | `.with_to(contract_address)` |
| `.with_value(u256)` | ETH value | `.with_value(U256::from(1e18 as u64))` |
| `.with_input(bytes)` | Calldata | `.with_input(calldata)` |
| `.with_nonce(u64)` | Nonce | `.with_nonce(0)` |
| `.with_chain_id(u64)` | Chain ID | `.with_chain_id(1)` |
| `.with_gas_limit(u64)` | Gas limit | `.with_gas_limit(21000)` |
| `.with_max_fee_per_gas(u128)` | Max fee | `.with_max_fee_per_gas(30_000_000_000)` |
| `.with_max_priority_fee_per_gas(u128)` | Priority fee | `.with_max_priority_fee_per_gas(1_000_000_000)` |
| `.with_max_fee_per_blob_gas(u128)` | Blob gas fee | For EIP-4844 transactions |
| `.with_blob_sidecar(sidecar)` | Blob data | For EIP-4844 transactions |

## Basic Usage

### Simple ETH Transfer

```rust
use alloy::rpc::types::TransactionRequest;
use alloy::primitives::U256;

let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH

let pending = provider.send_transaction(tx).await?;
let receipt = pending.get_receipt().await?;
```

### Contract Call

```rust
let tx = TransactionRequest::default()
    .with_to(contract_address)
    .with_input(calldata); // ABI-encoded function call

let pending = provider.send_transaction(tx).await?;
```

### Contract Deployment

```rust
// No `to` field = contract creation
let tx = TransactionRequest::default()
    .with_input(deployment_bytecode);

let pending = provider.send_transaction(tx).await?;
let receipt = pending.get_receipt().await?;
let deployed_address = receipt.contract_address;
```

## Using with sol! Contract Bindings

Most contract interactions go through generated bindings (see [sol! macro guide](alloy://sol-macro/contract-bindings)). But sometimes you need a raw TransactionRequest:

```rust
use alloy::sol_types::SolCall;

// Encode call manually
let call = ERC20::transferCall {
    to: recipient,
    amount: U256::from(1000),
};

let tx = TransactionRequest::default()
    .with_to(token_address)
    .with_input(call.abi_encode());

provider.send_transaction(tx).await?;
```

## Gas Configuration

### EIP-1559 (Default)

```rust
let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_value(U256::from(1_000_000_000_000_000_000u64))
    .with_max_fee_per_gas(30_000_000_000)       // max total fee per gas
    .with_max_priority_fee_per_gas(1_000_000_000) // tip to validator
    .with_gas_limit(21000);
```

With default fillers enabled, gas fields are estimated automatically. You only need to set them for overrides.

### Legacy Gas Price

```rust
let tx = TransactionRequest::default()
    .with_to(recipient)
    .with_gas_price(20_000_000_000); // sets legacy gas_price
```

## Blob Transactions (EIP-4844)

```rust
use alloy::consensus::BlobTransactionSidecar;

let tx = TransactionRequest::default()
    .with_to(rollup_inbox)
    .with_max_fee_per_blob_gas(1_000_000_000)
    .with_blob_sidecar(sidecar); // BlobTransactionSidecar
```

See [Blob encoding guide](alloy://encoding/blobs) for building sidecars.

## Transaction Simulation (eth_call)

```rust
// Simulate without sending
let result = provider.call(&tx).await?;

// Simulate at a specific block
let result = provider.call(&tx)
    .block(BlockId::number(12345678))
    .await?;
```

## Waiting for Receipts

```rust
let pending = provider.send_transaction(tx).await?;

// Wait for inclusion
let receipt = pending.get_receipt().await?;

// Check status
if receipt.status() {
    println!("Transaction succeeded");
} else {
    println!("Transaction reverted");
}

// Get transaction hash
let tx_hash = pending.tx_hash();
```

## MEV Bundle Types

For MEV (Flashbots-style) bundles:

```rust
use alloy::rpc::types::mev::{EthSendBundle, EthCallBundle};

// Send bundle
let bundle = EthSendBundle {
    txs: vec![raw_tx_bytes],
    block_number: target_block,
    min_timestamp: None,
    max_timestamp: None,
    reverting_tx_hashes: vec![],
    replacement_uuid: None,
};

// Simulate bundle
let call_bundle = EthCallBundle {
    txs: vec![raw_tx_bytes],
    block_number: target_block,
    state_block_number: BlockNumberOrTag::Latest,
    timestamp: None,
    gas_limit: None,
    difficulty: None,
    base_fee: None,
};
```

## Common Mistakes

1. **Setting `to` for contract deployment** — omit `.with_to()` for CREATE transactions
2. **Not waiting for receipt** — `send_transaction` returns a pending tx; call `.get_receipt().await?` to wait for inclusion
3. **Setting gas fields with fillers enabled** — if using default fillers, your explicit values may be overridden; use `disable_recommended_fillers()` for full control
4. **Forgetting sidecar for blob tx** — blob transactions need both `max_fee_per_blob_gas` and a sidecar
5. **Using `gas_price` with EIP-1559** — don't mix legacy `gas_price` with `max_fee_per_gas`; pick one style
