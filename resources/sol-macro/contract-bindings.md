# sol! Macro & Contract Bindings

## Quick Reference

| Goal | Syntax |
|------|--------|
| Define contract interface (calls only) | `sol! { contract Foo { ... } }` |
| Contract interface with RPC methods | `sol! { #[sol(rpc)] contract Foo { ... } }` |
| Deployable contract (with bytecode) | `sol! { #[sol(rpc, bytecode = "0x...")] contract Foo { ... } }` |
| Standalone function ABI | `sol! { function myFunc(uint256) returns (bool); }` |
| Standalone event | `sol! { event Transfer(address indexed, address indexed, uint256); }` |
| Standalone error | `sol! { error InsufficientBalance(uint256 available, uint256 required); }` |

## Defining Contract Bindings

### Basic Contract (Read-Only)

```rust
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract ERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function symbol() external view returns (string);
    }
}
```

This generates:
- `ERC20::balanceOfCall` — struct for encoding the call
- `ERC20::balanceOfReturn` — struct for decoding the return
- `ERC20::ERC20Instance` — contract instance with `.balanceOf(owner)` methods

### Full Contract with Events and Errors

```rust
sol! {
    #[sol(rpc)]
    contract ERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
        error InsufficientBalance(uint256 available, uint256 required);

        function balanceOf(address owner) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
    }
}
```

### Deployable Contract

```rust
sol! {
    #[sol(rpc, bytecode = "0x608060...")]
    contract Counter {
        uint256 public number;
        function increment() external;
        function setNumber(uint256 newNumber) external;
        function number() external view returns (uint256);
    }
}
```

## Using Contract Instances

### Creating an Instance

```rust
use alloy::providers::ProviderBuilder;

let provider = ProviderBuilder::new()
    .connect("https://eth.llamarpc.com")
    .await?;

// Create instance from address + provider
let contract = ERC20::new(token_address, &provider);
```

### Reading (call)

```rust
// .call() simulates the call (eth_call), does not send a transaction
let balance = contract.balanceOf(owner_address).call().await?;
// balance is ERC20::balanceOfReturn { _0: U256 }
let value: U256 = balance._0;
```

### Writing (send)

```rust
// .send() submits a transaction (requires wallet on provider)
let pending = contract.transfer(recipient, amount).send().await?;
let receipt = pending.get_receipt().await?;
```

### Deploying

```rust
// Only works with #[sol(rpc, bytecode = "0x...")]
let contract = Counter::deploy(&provider).await?;
let address = contract.address();
```

## SolCall: Manual ABI Encoding

Generated `*Call` structs implement `SolCall` for manual encoding/decoding:

```rust
use alloy::sol_types::SolCall;

// Encode function call data
let call = ERC20::transferCall {
    to: recipient,
    amount: U256::from(1000),
};
let calldata: Vec<u8> = call.abi_encode();

// Decode return data
let return_data = ERC20::transferReturn::abi_decode(&output, true)?;
```

## SolEvent: Log/Event Decoding

Generated event structs implement `SolEvent`:

```rust
use alloy::sol_types::SolEvent;

// Decode a log into a Transfer event
let transfer = ERC20::Transfer::decode_log(&log, true)?;
println!("from: {}, to: {}, value: {}", transfer.from, transfer.to, transfer.value);
```

### SolEventInterface: Decode Any Event from a Contract

```rust
use alloy::sol_types::SolEventInterface;

// Decode any event from the contract
let event = ERC20::ERC20Events::decode_log(&log, true)?;
match event.data {
    ERC20::ERC20Events::Transfer(t) => println!("transfer: {}", t.value),
    ERC20::ERC20Events::Approval(a) => println!("approval: {}", a.value),
}
```

## Solidity-to-Rust Type Mapping

| Solidity Type | Rust Type | Crate |
|--------------|-----------|-------|
| `address` | `Address` | `alloy-primitives` |
| `uint256` | `U256` | `alloy-primitives` |
| `uint128` | `u128` | std |
| `uint64` | `u64` | std |
| `uint32` | `u32` | std |
| `uint8` | `u8` | std |
| `int256` | `I256` | `alloy-primitives` |
| `bool` | `bool` | std |
| `bytes32` | `FixedBytes<32>` / `B256` | `alloy-primitives` |
| `bytes` | `Bytes` | `alloy-primitives` |
| `string` | `String` | std |
| `address[]` | `Vec<Address>` | std + `alloy-primitives` |
| `(uint256, address)` | tuple `(U256, Address)` | — |

## Common Mistakes

1. **Missing `#[sol(rpc)]`** — without this attribute, no `ContractInstance` or `.call()`/`.send()` methods are generated
2. **Using Rust types inside sol!** — the macro expects Solidity syntax: `uint256` not `U256`, `address` not `Address`
3. **Wrong import path for generated types** — types are generated under the contract module: `ERC20::Transfer`, not just `Transfer`
4. **Forgetting `external`/`view` on functions** — the macro needs visibility and mutability modifiers
5. **Using `.call()` when you mean `.send()`** — `.call()` simulates (free, no state change), `.send()` submits a transaction (costs gas, changes state)
6. **Not handling `_0` on return values** — single return values are in `._0`, named returns use their name
