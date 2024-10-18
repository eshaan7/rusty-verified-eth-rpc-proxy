use alloy::consensus::Account;
use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::BlockNumberOrTag;
use eyre::Result;

pub trait RpcVerifiableMethods {
    fn get_account(
        &self,
        address: Address,
        slots: Option<&[B256]>,
        tag: Option<BlockNumberOrTag>,
    ) -> impl std::future::Future<Output = Result<Account>> + Send;
    fn get_balance(
        &self,
        address: Address,
        tag: Option<BlockNumberOrTag>,
    ) -> impl std::future::Future<Output = Result<U256>> + Send;
    fn get_nonce(
        &self,
        address: Address,
        tag: Option<BlockNumberOrTag>,
    ) -> impl std::future::Future<Output = Result<u64>> + Send;
}
