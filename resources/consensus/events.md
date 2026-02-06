# Alloy Event & Log Decoding

## Quick Reference

| Goal | Type / Method |
|------|---------------|
| Decode a known event | `MyEvent::decode_log(&log, validate)` |
| Decode any event from contract | `ContractEvents::decode_log(&log, validate)` |
| Filter events on provider | `provider.subscribe_logs(&filter)` |
| Build a log filter | `Filter::new().address(addr).event_signature(sig)` |
| Get event signature hash | `MyEvent::SIGNATURE_HASH` |

## Log Structure

Ethereum logs contain topics and data:

```rust
use alloy::primitives::{Address, B256, Bytes};

// alloy_primitives::Log<T>
struct Log<T> {
    address: Address,     // Contract that emitted the event
    data: T,              // LogData: topics + data
}

// alloy_primitives::LogData
struct LogData {
    topics: Vec<B256>,    // topic0 = event signature, rest = indexed params
    data: Bytes,          // ABI-encoded non-indexed params
}
```

## Decoding Events with sol!

### Define Events

```rust
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract ERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }
}
```

### Decode a Single Event

```rust
use alloy::sol_types::SolEvent;

// From a raw log
let transfer = ERC20::Transfer::decode_log(&log, true)?;
// true = validate topic0 matches; false = skip validation

println!("from: {}", transfer.from);
println!("to: {}", transfer.to);
println!("value: {}", transfer.value);
```

### Decode Any Contract Event (SolEventInterface)

When you don't know which event a log represents:

```rust
use alloy::sol_types::SolEventInterface;

let event = ERC20::ERC20Events::decode_log(&log, true)?;
match event.data {
    ERC20::ERC20Events::Transfer(t) => {
        println!("Transfer: {} -> {} ({})", t.from, t.to, t.value);
    }
    ERC20::ERC20Events::Approval(a) => {
        println!("Approval: {} approved {} for {}", a.owner, a.spender, a.value);
    }
}
```

### Handling Fallible Decoding

Logs may not match your expected events. Use proper error handling:

```rust
for log in receipt.inner.logs() {
    // Try to decode, skip if it doesn't match
    if let Ok(transfer) = ERC20::Transfer::decode_log(log, true) {
        println!("Transfer: {} -> {}", transfer.from, transfer.to);
    }
}
```

Or with the event interface:

```rust
for log in logs {
    match ERC20::ERC20Events::decode_log(&log, true) {
        Ok(event) => match event.data {
            ERC20::ERC20Events::Transfer(t) => handle_transfer(t),
            ERC20::ERC20Events::Approval(a) => handle_approval(a),
        },
        Err(_) => continue, // Not one of our events
    }
}
```

## Event Filtering on Providers

### Subscribe to Events (WebSocket)

```rust
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol_types::SolEvent;

let filter = Filter::new()
    .address(contract_address)
    .event_signature(ERC20::Transfer::SIGNATURE_HASH);

let sub = provider.subscribe_logs(&filter).await?;
let mut stream = sub.into_stream();

while let Some(log) = stream.next().await {
    if let Ok(transfer) = ERC20::Transfer::decode_log(&log, true) {
        println!("Transfer: {} -> {} ({})", transfer.from, transfer.to, transfer.value);
    }
}
```

### Get Historical Logs

```rust
use alloy::rpc::types::Filter;
use alloy::eips::BlockNumberOrTag;

let filter = Filter::new()
    .address(contract_address)
    .event_signature(ERC20::Transfer::SIGNATURE_HASH)
    .from_block(BlockNumberOrTag::Number(18_000_000))
    .to_block(BlockNumberOrTag::Latest);

let logs = provider.get_logs(&filter).await?;

for log in logs {
    let transfer = ERC20::Transfer::decode_log(&log, true)?;
    println!("{} -> {} : {}", transfer.from, transfer.to, transfer.value);
}
```

### Filter by Indexed Parameters

```rust
// Get transfers TO a specific address
let filter = Filter::new()
    .address(contract_address)
    .event_signature(ERC20::Transfer::SIGNATURE_HASH)
    .topic2(recipient_address.into()); // topic2 = 'to' (2nd indexed param)
```

Topic layout for `event Transfer(address indexed from, address indexed to, uint256 value)`:
- `topic0` = event signature hash
- `topic1` = `from` (first indexed param)
- `topic2` = `to` (second indexed param)
- `data` = `value` (non-indexed param, ABI-encoded)

## Event Signature Hash

```rust
use alloy::sol_types::SolEvent;

// Get the event signature hash (topic0)
let sig: B256 = ERC20::Transfer::SIGNATURE_HASH;
// = keccak256("Transfer(address,address,uint256)")
```

## Common Mistakes

1. **Wrong validate flag** — `decode_log(&log, true)` validates topic0; use `false` if you've already filtered by signature
2. **Confusing indexed vs non-indexed params** — indexed params are in topics, non-indexed are ABI-encoded in data
3. **Not handling decode failures** — logs from other contracts or events will fail to decode; always handle errors
4. **Topic indexing off-by-one** — topic0 is the event signature, actual indexed params start at topic1
5. **Using `subscribe_logs` on HTTP** — subscriptions require WebSocket; use `get_logs` with HTTP
