# Alloy Sol Types: ABI Encoding & Decoding

## Quick Reference

| Trait | Purpose | Key Methods |
|-------|---------|-------------|
| `SolType` | Represents a Solidity type | `abi_encode()`, `abi_decode()` |
| `SolValue` | Rust value that maps to a SolType | `abi_encode()`, `abi_decode()` |
| `SolStruct` | Generated struct from sol! | `abi_encode()`, `eip712_signing_hash()` |
| `SolCall` | Generated function call data | `abi_encode()`, `abi_decode()` |
| `SolEvent` | Generated event data | `decode_log()`, `SIGNATURE_HASH` |

## SolType: Encoding Primitives

`SolType` represents Solidity types at the Rust type level.

```rust
use alloy::sol_types::sol_data;

// Encode a uint256
let encoded = sol_data::Uint::<256>::abi_encode(&U256::from(42));

// Encode an address
let encoded = sol_data::Address::abi_encode(&my_address);

// Encode a tuple (address, uint256)
let encoded = <(sol_data::Address, sol_data::Uint<256>)>::abi_encode(
    &(my_address, U256::from(42))
);

// Decode
let value = sol_data::Uint::<256>::abi_decode(&encoded, true)?;
```

## SolValue: Encode Rust Values Directly

`SolValue` lets you encode Rust values without specifying the Solidity type:

```rust
use alloy::sol_types::SolValue;

// Encode
let encoded: Vec<u8> = U256::from(42).abi_encode();
let encoded: Vec<u8> = my_address.abi_encode();

// Encode a tuple
let encoded = (my_address, U256::from(42)).abi_encode();

// Encode packed (no padding — used for keccak hashing)
let packed: Vec<u8> = (my_address, U256::from(42)).abi_encode_packed();
```

## SolStruct: Generated Structs

Structs defined in `sol!` get `SolStruct` implementation:

```rust
use alloy::sol;
use alloy::sol_types::SolStruct;

sol! {
    struct MyData {
        address owner;
        uint256 amount;
        bytes32 dataHash;
    }
}

let data = MyData {
    owner: my_address,
    amount: U256::from(1000),
    dataHash: hash,
};

// ABI encode
let encoded: Vec<u8> = data.abi_encode();

// ABI decode
let decoded = MyData::abi_decode(&encoded, true)?;

// EIP-712 signing hash (for typed data signing)
let signing_hash = data.eip712_signing_hash(&domain);
```

## SolCall: Function Call Encoding

Generated `*Call` structs encode/decode function calls:

```rust
use alloy::sol;
use alloy::sol_types::SolCall;

sol! {
    function transfer(address to, uint256 amount) external returns (bool);
}

// Encode calldata (includes 4-byte selector)
let call = transferCall {
    to: recipient,
    amount: U256::from(1000),
};
let calldata: Vec<u8> = call.abi_encode();

// Decode calldata
let decoded = transferCall::abi_decode(&calldata[4..], true)?;

// Get function selector
let selector: [u8; 4] = transferCall::SELECTOR;
```

## SolEvent: Event Encoding/Decoding

See the [Events resource](alloy://consensus/events) for detailed event decoding patterns.

```rust
use alloy::sol_types::SolEvent;

// Event signature hash (topic0)
let sig_hash: B256 = ERC20::Transfer::SIGNATURE_HASH;

// Decode from log
let transfer = ERC20::Transfer::decode_log(&log, true)?;
```

## Encoding Modes

### Standard ABI Encoding

```rust
// abi_encode() — standard ABI encoding with 32-byte padding
let encoded = value.abi_encode();
```

### Packed Encoding

```rust
// abi_encode_packed() — no padding, used with keccak256
let packed = (addr, amount).abi_encode_packed();
let hash = keccak256(&packed);
```

### Encode with Selector (for calldata)

```rust
// SolCall::abi_encode() includes the 4-byte function selector
let calldata = my_call.abi_encode(); // [selector | encoded params]
```

## Decoding with Validation

The `validate` parameter in `abi_decode`:

```rust
// validate = true: checks that decoded values are valid (e.g., address has zero upper bits)
let value = MyType::abi_decode(&data, true)?;

// validate = false: skip validation (faster, use when data is trusted)
let value = MyType::abi_decode(&data, false)?;
```

## Common Mistakes

1. **Forgetting the 4-byte selector** — `SolCall::abi_decode` expects data WITHOUT selector; skip first 4 bytes of calldata
2. **Using `abi_encode_packed` for ABI calls** — packed encoding is NOT standard ABI; only use for hash computation
3. **Wrong validate flag** — use `true` for untrusted external data, `false` for data you produced
4. **Confusing SolType vs SolValue** — `SolType` works at the type level (`sol_data::Uint::<256>::abi_encode(&val)`), `SolValue` works on values (`val.abi_encode()`)
