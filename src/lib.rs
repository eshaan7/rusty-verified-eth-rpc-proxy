// make the following modules public
pub mod common;
pub mod errors;
pub mod http_rpc;
pub mod verified_rpc_client;

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, B256};
    use alloy::rpc::types::BlockNumberOrTag;
    use std::str::FromStr;

    use crate::common::RpcVerifiableMethods;
    use crate::http_rpc::HttpRpc;
    use crate::verified_rpc_client::VerifiedRpcClient;

    const ETHEREUM_RPC_URL: &str = "https://eth.merkle.io";
    const ADDRESS: &str = "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    /// Expected to pass
    #[tokio::test]
    async fn test_verified_rpc_client_get_account_success() {
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
        println!(
            "latest block state root: {:?}",
            latest_block.header.state_root
        );

        verified_client
            .add_trusted_blocks(&[(latest_block.header.number, latest_block.header.state_root)]);
        let tag = Some(BlockNumberOrTag::from(latest_block.header.number));

        let addr = Address::from_str(ADDRESS).expect("failed to parse address");

        let verified_account = verified_client
            .get_account(addr, None, tag)
            .await
            .expect("failed to VerifiedRpcClient.get_account");
        let account = rpc
            .get_account(addr, None, tag)
            .await
            .expect("failed to HttpRpc.get_account");

        println!("account: {:?}", account);

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

        assert!(result.is_err_and(|e| e.to_string() == "Block 1 has no trusted state root"));
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

        verified_client.add_trusted_blocks(&[(latest_block_number, B256::ZERO)]);
        let tag = Some(BlockNumberOrTag::from(latest_block_number));

        let addr = Address::from_str(ADDRESS).expect("failed to parse address");

        let result = verified_client.get_account(addr, None, tag).await;

        assert!(result.is_err_and(|e| e.to_string().starts_with("Failed to verify proof")));
    }
}
