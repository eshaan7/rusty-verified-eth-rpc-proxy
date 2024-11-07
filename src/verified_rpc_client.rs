use std::collections::HashMap;

use alloy::consensus::Account;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{Address, BlockHash, BlockNumber, Bytes, B256, U256};
use alloy::rpc::types::TransactionReceipt;
use eyre::{eyre, Ok, Result};

use crate::common::RpcVerifiableMethods;
use crate::http_rpc::HttpRpc;
use crate::proof::{proof_to_account, verify_code_hash, verify_rpc_proof, verify_storage_proof};
use crate::utils::{encode_receipt, ordered_trie_root};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TrustedBlock {
    pub number: BlockNumber,
    pub hash: BlockHash,
    pub state_root: B256,
    pub receipts_root: B256,
}

#[derive(Debug, PartialEq, Eq)]
pub struct State {
    /// A map from block number to state root
    trusted_blocks: HashMap<BlockNumber, TrustedBlock>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            trusted_blocks: HashMap::new(),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trusted_blocks(&mut self, blocks: &[TrustedBlock]) {
        for block in blocks {
            self.trusted_blocks.insert(block.number, block.clone());
        }
    }

    pub fn latest_trusted_block(&self) -> Option<&TrustedBlock> {
        self.trusted_blocks.values().max_by_key(|b| b.number)
    }
}

pub struct VerifiedRpcClient {
    pub state: State,
    rpc: HttpRpc,
}

impl VerifiedRpcClient {
    pub fn new(rpc: &str) -> Result<Self> {
        Ok(Self {
            state: State::default(),
            rpc: HttpRpc::new(rpc)?,
        })
    }
}

impl RpcVerifiableMethods for VerifiedRpcClient {
    async fn get_account(
        &self,
        address: Address,
        slots: Option<&[B256]>,
        tag: Option<BlockNumberOrTag>,
    ) -> Result<Account> {
        let slots = slots.unwrap_or(&[]);

        // Get block number from the tag
        let block_number = self.rpc.get_block_number(tag).await?;
        let tag = Some(BlockNumberOrTag::Number(block_number));

        // Ensure we have this block in our trusted blocks
        let trusted_block = self
            .state
            .trusted_blocks
            .get(&block_number)
            .ok_or_else(|| eyre!("Block {block_number} is not in trusted list"))?;

        // Get account proof from the RPC
        let proof = self.rpc.get_proof(address, slots, block_number).await?;
        // Get account code from the RPC
        let code = self.rpc.get_code(address, tag).await?;

        // MOST IMPORTANT!!
        // Verify the proof fetched from RPC against the trusted state root
        verify_rpc_proof(&proof, &code, &trusted_block.state_root)?;

        let account = proof_to_account(&proof);

        Ok(account)
    }

    async fn get_balance(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<U256> {
        let account = self.get_account(address, None, tag).await?;

        Ok(account.balance)
    }

    async fn get_nonce(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<u64> {
        let account = self.get_account(address, None, tag).await?;

        Ok(account.nonce)
    }

    async fn get_code(&self, address: Address, tag: Option<BlockNumberOrTag>) -> Result<Bytes> {
        // Get block number from the tag
        let block_number = self.rpc.get_block_number(tag).await?;
        let tag = Some(BlockNumberOrTag::Number(block_number));

        // Get account code from the RPC
        let code = self.rpc.get_code(address, tag).await?;
        // Get account proof from the RPC
        let proof = self.rpc.get_proof(address, &[], block_number).await?;

        // MOST IMPORTANT!!
        // Verify the proof fetched from RPC
        verify_code_hash(&proof, &code)?;

        Ok(code)
    }

    async fn get_storage_at(
        &self,
        address: Address,
        slot: B256,
        tag: Option<BlockNumberOrTag>,
    ) -> Result<U256> {
        // Get block number from the tag
        let block_number = self.rpc.get_block_number(tag).await?;

        // Get account proof from the RPC
        let proof = self.rpc.get_proof(address, &[slot], block_number).await?;

        // MOST IMPORTANT!!
        // Verify the proof fetched from RPC
        verify_storage_proof(&proof)?;

        proof
            .storage_proof
            .iter()
            .find(|storage_slot| storage_slot.key.0 == slot)
            .map_or_else(
                || Err(eyre!("Slot not found")),
                |storage_slot| Ok(storage_slot.value),
            )
    }

    async fn get_transaction_receipt(&self, tx_hash: B256) -> Result<TransactionReceipt> {
        let receipt = self.rpc.get_transaction_receipt(tx_hash).await?;

        let block_num = receipt
            .block_number
            .ok_or_else(|| eyre!("Block number not found in receipt for tx hash {tx_hash}"))?;
        let tag = Some(BlockNumberOrTag::Number(block_num));

        let receipts = self.get_block_receipts(tag).await?;

        if !receipts.contains(&&receipt) {
            // Note: for some reason the above check is flaky
            // so we compare again by encoding the receipts
            if encode_receipt(&receipt)
                != encode_receipt(&receipts[receipt.transaction_index.unwrap() as usize])
            {
                return Err(eyre!(
                    "Failed to verify receipt for tx hash {tx_hash} in block {block_num}"
                ));
            }
        }

        Ok(receipt)
    }

    async fn get_block_receipts(
        &self,
        tag: Option<BlockNumberOrTag>,
    ) -> Result<Vec<TransactionReceipt>> {
        // Get block number from the tag
        let block_number = self.rpc.get_block_number(tag).await?;

        // Ensure we have this block in our trusted blocks
        let trusted_block = self
            .state
            .trusted_blocks
            .get(&block_number)
            .ok_or_else(|| eyre!("Block {block_number} is not trusted"))?;

        let receipts = self.rpc.get_block_receipts(tag).await?;

        // MOST IMPORTANT!!
        // Verify the receipts root
        let receipts_encoded: Vec<Vec<u8>> = receipts.iter().map(|r| encode_receipt(r)).collect();
        let computed_receipts_root = ordered_trie_root(receipts_encoded.as_slice());
        if computed_receipts_root != trusted_block.receipts_root {
            return Err(eyre!(
                "Receipts root mismatch for block {:?}: expected {:?}, got {:?}",
                block_number,
                trusted_block.receipts_root,
                computed_receipts_root
            ));
        }

        Ok(receipts)
    }
}
