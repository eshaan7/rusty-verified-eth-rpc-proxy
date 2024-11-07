// make the following modules public
pub mod common;
pub mod http_rpc;
pub mod proof;
pub mod utils;
pub mod verified_rpc_client;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::{Address, B256};
    use alloy::rpc::types::BlockNumberOrTag;

    use crate::common::RpcVerifiableMethods;
    use crate::http_rpc::HttpRpc;
    use crate::verified_rpc_client::{TrustedBlock, VerifiedRpcClient};

    const ETHEREUM_RPC_URL: &str = "https://eth.merkle.io";
    const ADDRESS: &str = "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    async fn setup() -> (HttpRpc, VerifiedRpcClient, TrustedBlock) {
        let rpc = HttpRpc::new(ETHEREUM_RPC_URL).expect("failed to create HttpRpc");

        let mut verified_client =
            VerifiedRpcClient::new(ETHEREUM_RPC_URL).expect("failed to create VerifiedRpcClient");

        // [testing] fetch latest block and mark it trusted
        // HTTP call is required bcz public RPC only allows eth_getProof for the latest block
        let latest_block = rpc
            .get_block(BlockNumberOrTag::Latest.into())
            .await
            .expect("failed to fetch latest block");
        println!("latest block number: {:?}", latest_block.header.number);

        let trusted_block = TrustedBlock {
            number: latest_block.header.number,
            hash: latest_block.header.hash,
            state_root: latest_block.header.state_root,
            receipts_root: latest_block.header.receipts_root,
        };
        verified_client.state.add_trusted_blocks(&[trusted_block]);

        (rpc, verified_client, trusted_block)
    }

    /// Expected to pass
    #[tokio::test]
    async fn test_verified_rpc_client_get_account_success() {
        let (rpc, verified_client, trusted_block) = setup().await;

        let tag = Some(BlockNumberOrTag::Number(trusted_block.number));

        let addr = Address::from_str(ADDRESS).expect("failed to parse address");

        let account = rpc
            .get_account(addr, None, tag)
            .await
            .expect("failed to HttpRpc.get_account");
        let verified_account = verified_client
            .get_account(addr, None, tag)
            .await
            .expect("failed to VerifiedRpcClient.get_account");

        assert_eq!(account, verified_account);
    }

    /// Expected to fail because the requested block number is not available as a trusted block
    #[tokio::test]
    async fn test_verified_rpc_client_get_account_error_untrusted_block() {
        let verified_client =
            VerifiedRpcClient::new(ETHEREUM_RPC_URL).expect("failed to create VerifiedRpcClient");

        // [testing] request for a block number that is not trusted
        let tag = Some(BlockNumberOrTag::Number(1));

        let addr = Address::from_str(ADDRESS).expect("failed to parse address");

        let result = verified_client.get_account(addr, None, tag).await;

        assert!(result.is_err_and(|e| e.to_string() == "Block 1 is not in trusted list"));
    }

    /// Expected to fail because the provided state root is invalid
    #[tokio::test]
    async fn test_verified_rpc_client_get_account_error_invalid_state_root() {
        let rpc = HttpRpc::new(ETHEREUM_RPC_URL).expect("failed to create HttpRpc");

        let mut verified_client =
            VerifiedRpcClient::new(ETHEREUM_RPC_URL).expect("failed to create VerifiedRpcClient");

        // [testing] fetch latest block and mark it trusted but with an invalid state root
        let latest_block_number = rpc
            .get_block_number(BlockNumberOrTag::Latest.into())
            .await
            .expect("failed to fetch latest block number");
        println!("latest block number: {:?}", latest_block_number);
        let trusted_block = TrustedBlock {
            number: latest_block_number,
            hash: B256::ZERO,
            state_root: B256::ZERO,
            receipts_root: B256::ZERO,
        };

        verified_client.state.add_trusted_blocks(&[trusted_block]);
        let tag = Some(BlockNumberOrTag::Number(latest_block_number));

        let addr = Address::from_str(ADDRESS).expect("failed to parse address");

        let result = verified_client.get_account(addr, None, tag).await;

        assert!(result.is_err_and(|e| e.to_string().starts_with("Failed to verify account proof")));
    }

    /// Expected to pass
    #[tokio::test]
    async fn test_verified_rpc_client_get_transaction_receipt_success() {
        let (rpc, verified_client, trusted_block) = setup().await;

        let tag = Some(BlockNumberOrTag::Number(trusted_block.number));

        let block = rpc.get_block(tag).await.expect("failed to fetch block");

        let tx_hash = block
            .transactions
            .hashes()
            .next()
            .expect("no transactions in block");

        let receipt = rpc
            .get_transaction_receipt(tx_hash)
            .await
            .expect("failed to HttpRpc.get_transaction_receipt");
        let verified_receipt = verified_client
            .get_transaction_receipt(tx_hash)
            .await
            .expect("failed to VerifiedRpcClient.get_transaction_receipt");

        assert_eq!(receipt, verified_receipt);
    }

    /// Expected to pass
    #[tokio::test]
    async fn test_verified_rpc_client_get_block_receipts_success() {
        let (rpc, verified_client, trusted_block) = setup().await;

        let tag = Some(BlockNumberOrTag::Number(trusted_block.number));

        let receipts = rpc
            .get_block_receipts(tag)
            .await
            .expect("failed to HttpRpc.get_block_receipts");
        let verified_receipts = verified_client
            .get_block_receipts(tag)
            .await
            .expect("failed to VerifiedRpcClient.get_block_receipts");

        assert_eq!(receipts, verified_receipts);
    }
}
