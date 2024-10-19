use alloy::consensus::Account;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::rpc::json_rpc::{RpcParam, RpcReturn};
use alloy::rpc::types::{Block, BlockNumberOrTag, EIP1186AccountProofResponse};
use alloy::transports::http::Http;
use eyre::{eyre, Ok, Result};
use reqwest::Client;

use crate::common::RpcVerifiableMethods;

pub struct HttpRpc {
    #[allow(dead_code)]
    url: String,
    provider: RootProvider<Http<Client>>,
}

impl HttpRpc {
    pub fn new(rpc: &str) -> Result<Self> {
        let provider = ProviderBuilder::new().on_http(rpc.parse().unwrap());

        Ok(HttpRpc {
            url: rpc.to_string(),
            provider,
        })
    }

    pub async fn raw_request<P: RpcParam, R: RpcReturn>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R> {
        let response = self
            .provider
            .raw_request(method.to_string().into(), params)
            .await
            .map_err(|e| eyre!("Method: {method}, Error: {e}"))?;

        Ok(response)
    }

    pub async fn get_proof(
        &self,
        address: Address,
        slots: &[B256],
        block_number: u64,
    ) -> Result<EIP1186AccountProofResponse> {
        let proof = self
            .provider
            .get_proof(address, slots.to_vec())
            .block_id(block_number.into())
            .await
            .map_err(|e| eyre!("Method: eth_getProof, Error: {e}"))?;

        Ok(proof)
    }

    pub async fn get_block_number(&self, tag: Option<BlockNumberOrTag>) -> Result<u64> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        // if tag is a number, simply return it
        if tag.is_number() {
            return Ok(tag.as_number().unwrap());
        }

        // if tag is latest, fetch the latest block number using eth_blockNumber method
        if tag.is_latest() {
            let block_number = self.provider.get_block_number().await?;
            return Ok(block_number);
        }

        // otherwise, fetch the block using eth_getBlockByNumber method
        let block = self.get_block(tag.into()).await?;

        Ok(block.header.number)
    }

    pub async fn get_block(&self, tag: Option<BlockNumberOrTag>) -> Result<Block> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        let block = self
            .provider
            .get_block_by_number(tag, false)
            .await
            .map_err(|e| eyre!("Method: eth_getBlockByNumber, Error: {e}"))?
            .ok_or_else(|| eyre!("Block not found for {tag}"))?;

        Ok(block)
    }
}

impl RpcVerifiableMethods for HttpRpc {
    async fn get_account(
        &self,
        address: Address,
        _: Option<&[B256]>,
        tag: Option<BlockNumberOrTag>,
    ) -> Result<Account> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        let account = self
            .raw_request("eth_getAccount".into(), (address, tag))
            .await?;

        Ok(account)
    }

    async fn get_balance(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<U256> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        let balance = self
            .raw_request("eth_getBalance".into(), (address, tag))
            .await?;

        Ok(balance)
    }

    async fn get_nonce(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<u64> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        let nonce = self
            .raw_request("eth_getTransactionCount".into(), (address, tag))
            .await?;

        Ok(nonce)
    }

    async fn get_code(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<Bytes> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);

        let code = self
            .raw_request("eth_getCode".into(), (address, tag))
            .await?;

        Ok(code)
    }

    async fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        tag: Option<BlockNumberOrTag>,
    ) -> Result<U256> {
        let tag = tag.unwrap_or(BlockNumberOrTag::Latest);
        let slot = Into::<U256>::into(slot);

        let storage_value = self
            .raw_request("eth_getStorageAt".into(), (address, slot, tag))
            .await?;

        Ok(storage_value)
    }
}
