# Alloy Signers & Signing Guide

## Quick Reference

| Goal | Type / Method |
|------|---------------|
| Create signer from private key | `PrivateKeySigner::from_bytes(&B256)` |
| Create random signer (testing) | `PrivateKeySigner::random()` |
| Parse signer from hex string | `"0xac09...".parse::<PrivateKeySigner>()` |
| Wrap signer for provider | `EthereumWallet::from(signer)` |
| Multiple signers in one wallet | `wallet.register_signer(signer2)` |
| Sign raw hash | `signer.sign_hash(&hash).await` |
| Sign message (EIP-191) | `signer.sign_message(msg).await` |
| Sign typed data (EIP-712) | `signer.sign_typed_data(&data, &domain).await` |

## PrivateKeySigner

The most common signer for development and server-side use.

### Creating a Signer

```rust
use alloy::signers::local::PrivateKeySigner;
use alloy::primitives::B256;

// From raw bytes
let key = B256::from_slice(&key_bytes);
let signer = PrivateKeySigner::from_bytes(&key)?;

// From hex string (with or without 0x prefix)
let signer: PrivateKeySigner = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    .parse()?;

// Random signer (for testing)
let signer = PrivateKeySigner::random();

// Get the address
let address = signer.address();
```

## EthereumWallet

Wraps one or more signers for use with a provider.

### Basic Usage

```rust
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xac0974...".parse()?;
let wallet = EthereumWallet::from(signer);

// Use with provider
let provider = ProviderBuilder::new()
    .wallet(wallet)
    .connect(url)
    .await?;
```

### Multiple Signers

```rust
let signer1: PrivateKeySigner = key1.parse()?;
let signer2: PrivateKeySigner = key2.parse()?;

let mut wallet = EthereumWallet::from(signer1);
wallet.register_signer(signer2);

// The wallet can now sign transactions from either address.
// The default signer is the first one registered.
```

## The Signer Trait (Async)

All signers implement `Signer` for async signing operations.

```rust
use alloy::signers::Signer;

// Sign a raw hash (32 bytes)
let signature = signer.sign_hash(&hash).await?;

// Sign an EIP-191 personal message
let signature = signer.sign_message(b"Hello, world!").await?;

// Get the signer's chain ID
let chain_id = signer.chain_id();

// Get the signer's address
let address = signer.address();
```

## The SignerSync Trait

For synchronous contexts (no async runtime):

```rust
use alloy::signers::SignerSync;

let signature = signer.sign_hash_sync(&hash)?;
let signature = signer.sign_message_sync(b"Hello, world!")?;
```

## EIP-712 Typed Data Signing

EIP-712 provides structured data signing with domain separation.

### Step 1: Define the Struct with sol!

```rust
use alloy::sol;

sol! {
    #[derive(Debug)]
    struct OrderData {
        address signer;
        uint64 deadline;
        uint256 nonce;
        bytes32 dataHash;
    }
}
```

Generated types automatically implement `SolStruct` and `Eip712Domain` support.

### Step 2: Define the EIP-712 Domain

```rust
use alloy::sol_types::eip712_domain;

let domain = eip712_domain! {
    name: "MyProtocol",
    version: "1",
    chain_id: 1,
    verifying_contract: contract_address,
};
```

### Step 3: Compute the Signing Hash

```rust
use alloy::sol_types::SolStruct;

let order = OrderData {
    signer: my_address,
    deadline: 1234567890,
    nonce: U256::from(1),
    dataHash: data_hash,
};

// Compute EIP-712 signing hash
let signing_hash = order.eip712_signing_hash(&domain);
```

### Step 4: Sign

```rust
use alloy::signers::Signer;

let signature = signer.sign_hash(&signing_hash).await?;
```

### Complete EIP-712 Example

```rust
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};
use alloy::signers::{local::PrivateKeySigner, Signer};

sol! {
    struct Permit {
        address owner;
        address spender;
        uint256 value;
        uint256 nonce;
        uint256 deadline;
    }
}

let domain = eip712_domain! {
    name: "MyToken",
    version: "1",
    chain_id: 1,
    verifying_contract: token_address,
};

let permit = Permit {
    owner: signer.address(),
    spender: spender_address,
    value: U256::from(1000),
    nonce: U256::ZERO,
    deadline: U256::from(u64::MAX),
};

let hash = permit.eip712_signing_hash(&domain);
let signature = signer.sign_hash(&hash).await?;
```

## Transaction Signing (Low-Level)

For manually signing transactions (most users should use provider + wallet instead):

```rust
use alloy::consensus::SignableTransaction;

// Sign a transaction
let signature = signer.sign_hash(&tx.signature_hash()).await?;
let signed = tx.into_signed(signature);
```

## Common Mistakes

1. **Async vs sync confusion** — `sign_hash()` is async (needs `.await`), use `sign_hash_sync()` in non-async contexts
2. **Missing wallet registration** — if using multiple signers, register each with `wallet.register_signer()`
3. **Wrong hash for EIP-712** — use `eip712_signing_hash(&domain)`, not `keccak256` of the struct directly
4. **Forgetting domain separator** — EIP-712 requires a domain; different domains produce different signatures
5. **Using `sign_message` for typed data** — `sign_message` applies EIP-191 prefix; use `sign_hash` with `eip712_signing_hash` for EIP-712
