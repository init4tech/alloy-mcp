use rmcp::{
    model::{PromptMessage, PromptMessageRole},
    prompt, prompt_router,
};

use crate::server::AlloyMcpServer;

#[prompt_router]
impl AlloyMcpServer {
    pub fn create_prompt_router() -> rmcp::handler::server::router::prompt::PromptRouter<Self> {
        Self::prompt_router()
    }

    /// Step-by-step guide for building and sending an Ethereum transaction with alloy.
    #[prompt(
        name = "build_transaction",
        description = "Step-by-step guide: provider setup, TransactionRequest, send, receipt"
    )]
    fn build_transaction(&self) -> Vec<PromptMessage> {
        vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Help me build and send an Ethereum transaction using alloy.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Here's a step-by-step guide for building and sending a transaction with alloy:\n\n\
                ## Step 1: Set Up Provider\n\n\
                ```rust\n\
                use alloy::providers::ProviderBuilder;\n\
                use alloy::signers::local::PrivateKeySigner;\n\
                use alloy::network::EthereumWallet;\n\n\
                let signer: PrivateKeySigner = \"0xYOUR_PRIVATE_KEY\".parse()?;\n\
                let wallet = EthereumWallet::from(signer);\n\n\
                let provider = ProviderBuilder::new()\n\
                    .wallet(wallet)\n\
                    .connect(\"https://your-rpc-url\")\n\
                    .await?;\n\
                ```\n\n\
                ## Step 2: Build TransactionRequest\n\n\
                ```rust\n\
                use alloy::rpc::types::TransactionRequest;\n\
                use alloy::primitives::U256;\n\n\
                let tx = TransactionRequest::default()\n\
                    .with_to(recipient_address)\n\
                    .with_value(U256::from(1_000_000_000_000_000_000u64)); // 1 ETH\n\
                ```\n\n\
                Gas, nonce, and chain_id are filled automatically by default fillers.\n\n\
                ## Step 3: Send Transaction\n\n\
                ```rust\n\
                let pending = provider.send_transaction(tx).await?;\n\
                let tx_hash = pending.tx_hash();\n\
                println!(\"Transaction hash: {tx_hash}\");\n\
                ```\n\n\
                ## Step 4: Wait for Receipt\n\n\
                ```rust\n\
                let receipt = pending.get_receipt().await?;\n\
                if receipt.status() {\n\
                    println!(\"Transaction succeeded!\");\n\
                } else {\n\
                    println!(\"Transaction reverted\");\n\
                }\n\
                ```\n\n\
                **Key resources:**\n\
                - `alloy://rpc/transaction-request` — TransactionRequest builder methods\n\
                - `alloy://provider/setup` — Provider configuration\n\
                - `alloy://provider/fillers` — How gas/nonce/chain_id are auto-filled\n\
                - `alloy://signers/signing-guide` — Signer setup",
            ),
        ]
    }

    /// Step-by-step guide for setting up sol! contract bindings with alloy.
    #[prompt(
        name = "setup_contract_bindings",
        description = "Guide: sol! macro, ContractInstance, call/send pattern"
    )]
    fn setup_contract_bindings(&self) -> Vec<PromptMessage> {
        vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Help me set up contract bindings using alloy's sol! macro.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Here's how to set up contract bindings with alloy's sol! macro:\n\n\
                ## Step 1: Define the Contract Interface\n\n\
                ```rust\n\
                use alloy::sol;\n\n\
                sol! {\n\
                    #[sol(rpc)]\n\
                    contract MyContract {\n\
                        event Transfer(address indexed from, address indexed to, uint256 value);\n\n\
                        function balanceOf(address owner) external view returns (uint256);\n\
                        function transfer(address to, uint256 amount) external returns (bool);\n\
                    }\n\
                }\n\
                ```\n\n\
                **Important:** `#[sol(rpc)]` is required to generate the contract instance with \
                `.call()`/`.send()` methods.\n\n\
                ## Step 2: Create Contract Instance\n\n\
                ```rust\n\
                let contract = MyContract::new(contract_address, &provider);\n\
                ```\n\n\
                ## Step 3: Read Data (call)\n\n\
                ```rust\n\
                let result = contract.balanceOf(owner_address).call().await?;\n\
                let balance: U256 = result._0;\n\
                ```\n\n\
                `.call()` simulates the call (free, no state change).\n\n\
                ## Step 4: Write Data (send)\n\n\
                ```rust\n\
                let pending = contract.transfer(recipient, amount).send().await?;\n\
                let receipt = pending.get_receipt().await?;\n\
                ```\n\n\
                `.send()` submits a transaction (costs gas, changes state). Requires wallet on provider.\n\n\
                ## Step 5: Decode Events\n\n\
                ```rust\n\
                use alloy::sol_types::SolEvent;\n\n\
                for log in receipt.inner.logs() {\n\
                    if let Ok(transfer) = MyContract::Transfer::decode_log(log, true) {\n\
                        println!(\"{} -> {}: {}\", transfer.from, transfer.to, transfer.value);\n\
                    }\n\
                }\n\
                ```\n\n\
                **Key resources:**\n\
                - `alloy://sol-macro/contract-bindings` — Full sol! macro reference\n\
                - `alloy://sol-macro/sol-types` — ABI encoding/decoding\n\
                - `alloy://consensus/events` — Event decoding patterns",
            ),
        ]
    }

    /// Step-by-step guide for setting up signing with alloy, including EIP-712.
    #[prompt(
        name = "setup_signing",
        description = "Guide: signer creation, wallet, signing flow including EIP-712"
    )]
    fn setup_signing(&self) -> Vec<PromptMessage> {
        vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Help me set up signing with alloy, including EIP-712 typed data.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Here's how to set up signing with alloy:\n\n\
                ## Step 1: Create a Signer\n\n\
                ```rust\n\
                use alloy::signers::local::PrivateKeySigner;\n\n\
                // From hex string\n\
                let signer: PrivateKeySigner = \"0xac0974...\".parse()?;\n\n\
                // Random (for testing)\n\
                let signer = PrivateKeySigner::random();\n\n\
                let address = signer.address();\n\
                ```\n\n\
                ## Step 2: Wrap in Wallet (for provider)\n\n\
                ```rust\n\
                use alloy::network::EthereumWallet;\n\n\
                let wallet = EthereumWallet::from(signer.clone());\n\
                let provider = ProviderBuilder::new()\n\
                    .wallet(wallet)\n\
                    .connect(url)\n\
                    .await?;\n\
                ```\n\n\
                ## Step 3: Sign a Message (EIP-191)\n\n\
                ```rust\n\
                use alloy::signers::Signer;\n\n\
                let signature = signer.sign_message(b\"Hello, world!\").await?;\n\
                ```\n\n\
                ## Step 4: Sign Typed Data (EIP-712)\n\n\
                ### 4a. Define the struct in sol!\n\n\
                ```rust\n\
                use alloy::sol;\n\n\
                sol! {\n\
                    struct MyMessage {\n\
                        address sender;\n\
                        uint256 amount;\n\
                        uint256 nonce;\n\
                    }\n\
                }\n\
                ```\n\n\
                ### 4b. Create the EIP-712 domain\n\n\
                ```rust\n\
                use alloy::sol_types::eip712_domain;\n\n\
                let domain = eip712_domain! {\n\
                    name: \"MyProtocol\",\n\
                    version: \"1\",\n\
                    chain_id: 1,\n\
                    verifying_contract: contract_address,\n\
                };\n\
                ```\n\n\
                ### 4c. Compute signing hash and sign\n\n\
                ```rust\n\
                use alloy::sol_types::SolStruct;\n\
                use alloy::signers::Signer;\n\n\
                let message = MyMessage {\n\
                    sender: signer.address(),\n\
                    amount: U256::from(1000),\n\
                    nonce: U256::ZERO,\n\
                };\n\n\
                let hash = message.eip712_signing_hash(&domain);\n\
                let signature = signer.sign_hash(&hash).await?;\n\
                ```\n\n\
                **Key resources:**\n\
                - `alloy://signers/signing-guide` — Full signer reference\n\
                - `alloy://sol-macro/sol-types` — SolStruct for EIP-712\n\
                - `alloy://primitives/core-types` — Address, B256, U256 types",
            ),
        ]
    }
}
