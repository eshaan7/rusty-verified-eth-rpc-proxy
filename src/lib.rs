// make the following modules public
pub mod errors;
pub mod http_rpc;
pub mod verified_rpc_client;

#[cfg(test)]
mod tests {
    use alloy::{primitives::Address, rpc::types::BlockNumberOrTag};
    use std::str::FromStr;

    use crate::http_rpc::HttpRpc;
    use crate::verified_rpc_client::VerifiedRpcClient;

    const ETHEREUM_RPC_URL: &str = "https://eth.merkle.io";

    #[tokio::test]
    async fn test_verified_rpc_client_get_account_success() {
        let rpc: HttpRpc = HttpRpc::new(ETHEREUM_RPC_URL).expect("failed to create HttpRpc");

        let mut verified_client =
            VerifiedRpcClient::new(ETHEREUM_RPC_URL).expect("failed to create VerifiedRpcClient");

        // [testing] fetch latest block and mark it trusted
        // public RPC only allows eth_getProof for the latest block
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

        let addr = Address::from_str("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
            .expect("failed to parse address");
        println!("address: {:?}", addr);

        let verified_account = verified_client
            .get_account(addr, None, tag)
            .await
            .expect("failed to VerifiedRpcClient.get_account");
        let account = rpc
            .get_account(addr, tag)
            .await
            .expect("failed to HttpRpc.get_account");

        println!("account: {:?}", account);

        assert_eq!(account, verified_account);
    }
}
