use alloy::consensus::Account;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::rpc::types::{BlockNumberOrTag, TransactionReceipt};
use eyre::Result;

pub trait RpcVerifiableMethods {
    /// RPC method: `eth_getAccount`
    fn get_account(
        &self,
        address: Address,
        slots: Option<&[B256]>,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<Account>> + Send;

    /// RPC method: `eth_getBalance`
    fn get_balance(
        &self,
        address: Address,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<U256>> + Send;

    /// RPC method: `eth_getTransactionCount`
    fn get_nonce(
        &self,
        address: Address,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<u64>> + Send;

    /// RPC method: `eth_getCode`
    fn get_code(
        &self,
        address: Address,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<Bytes>> + Send;

    /// RPC method: `eth_getStorageAt`
    fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<U256>> + Send;

    /// RPC method: `eth_getTransactionReceipt`
    fn get_transaction_receipt(
        &self,
        tx_hash: B256,
    ) -> impl core::future::Future<Output = Result<TransactionReceipt>> + Send;

    /// RPC method: `eth_getBlockReceipts`
    fn get_block_receipts(
        &self,
        tag: Option<BlockNumberOrTag>,
    ) -> impl core::future::Future<Output = Result<Vec<TransactionReceipt>>> + Send;
}
