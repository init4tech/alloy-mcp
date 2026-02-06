# Alloy Primitives & Core Types

## Quick Reference

| Type | Size | Description | Literal Macro |
|------|------|-------------|---------------|
| `Address` | 20 bytes | Ethereum address | `address!("0xd8dA...")` |
| `B256` | 32 bytes | 256-bit hash (tx hash, block hash) | `b256!("0xabcd...")` |
| `U256` | 32 bytes | 256-bit unsigned integer | `uint!(1_000_000_U256)` |
| `I256` | 32 bytes | 256-bit signed integer | — |
| `Bytes` | dynamic | Dynamic byte array | `bytes!("0xdeadbeef")` |
| `FixedBytes<N>` | N bytes | Fixed-size byte array | — |
| `Signature` | 65 bytes | ECDSA signature (r, s, v) | — |
| `TxKind` | — | Transaction destination | — |

All types are in `alloy_primitives` (re-exported as `alloy::primitives`).

## Address

20-byte Ethereum address with checksum support.

```rust
use alloy::primitives::{Address, address};

// Literal macro (compile-time validated)
let addr = address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

// From bytes
let addr = Address::from_slice(&bytes[..20]);

// Zero address
let zero = Address::ZERO;

// Parse from string (runtime)
let addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse()?;
```

## B256 (32-Byte Hash)

Used for transaction hashes, block hashes, storage slots, and other 32-byte values.

```rust
use alloy::primitives::{B256, b256};

// Literal macro
let hash = b256!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");

// Zero hash
let zero = B256::ZERO;

// From a repeated byte (useful for testing)
let test_hash = B256::repeat_byte(0xff);

// From slice
let hash = B256::from_slice(&bytes[..32]);
```

## U256 (256-bit Unsigned Integer)

For token amounts, balances, and Solidity `uint256` values.

```rust
use alloy::primitives::{U256, uint};

// Literal macro (supports _U256 suffix)
let amount = uint!(1_000_000_000_000_000_000_U256); // 1 ETH in wei

// From u64/u128
let small = U256::from(42u64);

// Zero
let zero = U256::ZERO;

// Max value
let max = U256::MAX;

// Arithmetic
let sum = a + b;
let product = a * b;
let quotient = a / b;

// Checked arithmetic (returns Option)
let sum = a.checked_add(b);
let product = a.checked_mul(b);

// Parsing from decimal string
let amount = U256::from_str_radix("1000000000000000000", 10)?;
```

## Bytes (Dynamic Byte Array)

For calldata, return data, and dynamic byte arrays.

```rust
use alloy::primitives::{Bytes, bytes};

// Literal macro
let data = bytes!("0xdeadbeef");

// From Vec<u8>
let data = Bytes::from(vec![0xde, 0xad]);

// Empty bytes
let empty = Bytes::new();

// From static slice
let data = Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef]);
```

## FixedBytes<N>

Generic fixed-size byte array. `B256` is an alias for `FixedBytes<32>`, `Address` wraps `FixedBytes<20>`.

```rust
use alloy::primitives::FixedBytes;

let bytes: FixedBytes<4> = FixedBytes::from([0xde, 0xad, 0xbe, 0xef]);

// Function selectors are 4 bytes
let selector: FixedBytes<4> = FixedBytes::from_slice(&keccak256(b"transfer(address,uint256)")[..4]);
```

## TxKind (Transaction Destination)

Represents `to` field: either a contract call or contract creation.

```rust
use alloy::primitives::TxKind;

// Call an existing contract
let to = TxKind::Call(contract_address);

// Create a new contract (deploy)
let to = TxKind::Create;
```

## keccak256

Hash function used throughout Ethereum.

```rust
use alloy::primitives::keccak256;

// Hash bytes, returns B256
let hash: B256 = keccak256(b"Hello, world!");

// Hash dynamic data
let hash = keccak256(&some_bytes);
```

## Useful Constants

```rust
use alloy::primitives::{Address, B256, U256};

// Zero values
Address::ZERO    // 0x0000000000000000000000000000000000000000
B256::ZERO       // 0x0000...0000
U256::ZERO       // 0

// Max values
U256::MAX        // 2^256 - 1

// ETH unit conversions
const ETH_TO_WEI: u64 = 1_000_000_000_000_000_000;  // 1e18
const GWEI_TO_WEI: u64 = 1_000_000_000;              // 1e9
```

## Address Aliasing (L1 ↔ L2)

Used in Optimism and other L2s where L1 addresses are offset.

```rust
use alloy::primitives::{Address, U160};

const OFFSET: U160 = uint!(0x1111000000000000000000000000000000001111_U160);

// L1 → L2 alias
fn address_to_l2_alias(addr: Address) -> Address {
    let u160 = U160::from_be_bytes(addr.into_array());
    Address::from(u160.wrapping_add(OFFSET))
}

// L2 → L1 unalias
fn address_from_l2_alias(addr: Address) -> Address {
    let u160 = U160::from_be_bytes(addr.into_array());
    Address::from(u160.wrapping_sub(OFFSET))
}
```

## Type Conversions

```rust
// Address ↔ B256
let b256 = B256::left_padding_from(address.as_slice());
let addr = Address::from_slice(&b256[12..]); // last 20 bytes

// U256 ↔ u64
let small: u64 = big_u256.try_into()?;  // fails if > u64::MAX
let big = U256::from(small_u64);

// Bytes ↔ Vec<u8>
let vec: Vec<u8> = bytes.to_vec();
let bytes = Bytes::from(vec);

// B256 ↔ [u8; 32]
let array: [u8; 32] = hash.0;
let hash = B256::from(array);
```

## Common Mistakes

1. **U256 literal overflow** — `U256::from(1_000_000_000_000_000_000u64)` works but `U256::from(1e18)` does NOT; use `uint!()` macro for large literals
2. **Bytes vs &[u8]** — `Bytes` is owned, cloneable; use `.as_ref()` to get `&[u8]`
3. **Address vs B256 size mismatch** — Address is 20 bytes, B256 is 32 bytes; pad/truncate correctly
4. **Parsing without 0x prefix** — `address!()` and `b256!()` macros require the `0x` prefix
5. **Using `as` for U256 conversion** — U256 doesn't support `as`; use `U256::from()` or `.try_into()`
