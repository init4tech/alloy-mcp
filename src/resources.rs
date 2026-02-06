use std::collections::HashMap;

/// A static resource loaded at compile time.
#[derive(Clone)]
pub struct StaticResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    pub content: String,
}

const TRANSACTIONS: &str = include_str!("../resources/consensus/transactions.md");
const BLOCK_IDENTIFIERS: &str = include_str!("../resources/eips/block-identifiers.md");
const PROVIDER_SETUP: &str = include_str!("../resources/provider/setup.md");
const CONTRACT_BINDINGS: &str = include_str!("../resources/sol-macro/contract-bindings.md");
const SIGNING_GUIDE: &str = include_str!("../resources/signers/signing-guide.md");
const CORE_TYPES: &str = include_str!("../resources/primitives/core-types.md");
const EVENTS: &str = include_str!("../resources/consensus/events.md");
const FILLERS: &str = include_str!("../resources/provider/fillers.md");
const SOL_TYPES: &str = include_str!("../resources/sol-macro/sol-types.md");
const TRANSACTION_REQUEST: &str = include_str!("../resources/rpc/transaction-request.md");
const RLP_EIP2718: &str = include_str!("../resources/encoding/rlp-eip2718.md");
const BLOBS: &str = include_str!("../resources/encoding/blobs.md");
const RECOVERED: &str = include_str!("../resources/consensus/recovered.md");

fn resource(uri: &str, name: &str, description: &str, content: &str) -> StaticResource {
    StaticResource {
        uri: uri.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        mime_type: "text/markdown".to_string(),
        content: content.to_string(),
    }
}

/// Returns all static resources indexed by URI.
pub fn all() -> HashMap<String, StaticResource> {
    let resources = [
        resource(
            "alloy://consensus/transactions",
            "Transaction Types",
            "Guide to alloy transaction types: TxLegacy, TxEip1559, TxEip4844, TxEnvelope, etc.",
            TRANSACTIONS,
        ),
        resource(
            "alloy://eips/block-identifiers",
            "Block Identifier Types",
            "Guide to BlockId, BlockNumberOrTag, HashOrNumber, and related types.",
            BLOCK_IDENTIFIERS,
        ),
        resource(
            "alloy://provider/setup",
            "Provider Setup",
            "Guide to setting up alloy providers: ProviderBuilder, wallets, WebSocket, layers.",
            PROVIDER_SETUP,
        ),
        resource(
            "alloy://sol-macro/contract-bindings",
            "sol! Macro & Contract Bindings",
            "Guide to sol! macro: contract interfaces, #[sol(rpc)], SolCall, SolEvent, type mapping.",
            CONTRACT_BINDINGS,
        ),
        resource(
            "alloy://signers/signing-guide",
            "Signers & Signing Guide",
            "Guide to PrivateKeySigner, EthereumWallet, Signer trait, EIP-712 typed data signing.",
            SIGNING_GUIDE,
        ),
        resource(
            "alloy://primitives/core-types",
            "Primitives & Core Types",
            "Guide to Address, B256, U256, Bytes, TxKind, keccak256, literal macros, conversions.",
            CORE_TYPES,
        ),
        resource(
            "alloy://consensus/events",
            "Event & Log Decoding",
            "Guide to SolEvent, SolEventInterface, log decoding, event filtering, subscriptions.",
            EVENTS,
        ),
        resource(
            "alloy://provider/fillers",
            "Provider Fillers",
            "Guide to GasFiller, NonceFiller, ChainIdFiller, BlobGasFiller, custom filler configs.",
            FILLERS,
        ),
        resource(
            "alloy://sol-macro/sol-types",
            "Sol Types: ABI Encoding & Decoding",
            "Guide to SolType, SolValue, SolStruct, SolCall traits for ABI encoding/decoding.",
            SOL_TYPES,
        ),
        resource(
            "alloy://rpc/transaction-request",
            "TransactionRequest Builder",
            "Guide to TransactionRequest: building, sending, gas config, blob txs, MEV bundles.",
            TRANSACTION_REQUEST,
        ),
        resource(
            "alloy://encoding/rlp-eip2718",
            "RLP & EIP-2718 Encoding",
            "Guide to Encodable2718, Decodable2718, RLP encoding/decoding, transaction serialization.",
            RLP_EIP2718,
        ),
        resource(
            "alloy://encoding/blobs",
            "Blob Transactions & Sidecars",
            "Guide to SidecarBuilder, SimpleCoder, BlobTransactionSidecar, blob transaction construction.",
            BLOBS,
        ),
        resource(
            "alloy://consensus/recovered",
            "Recovered Transactions & Type Aliases",
            "Guide to Recovered<T>, sender recovery, custom transaction type aliases, DataCompat.",
            RECOVERED,
        ),
    ];

    resources.into_iter().map(|r| (r.uri.clone(), r)).collect()
}
