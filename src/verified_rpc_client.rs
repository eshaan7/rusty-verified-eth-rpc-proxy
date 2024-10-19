use alloy::consensus::Account;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{Address, Bytes, B256, U256};
use eyre::{eyre, Ok, Result};
use std::collections::HashMap;

use crate::common::RpcVerifiableMethods;
use crate::http_rpc::HttpRpc;
use crate::proof::{proof_to_account, verify_code_hash, verify_rpc_proof, verify_storage_proof};

// Type alias for better readability
type BlockNumber = u64;
type StateRoot = B256;

pub struct VerifiedRpcClient {
    /// A map from block number to state root
    trusted_blocks: HashMap<BlockNumber, StateRoot>,
    rpc: HttpRpc,
}

impl VerifiedRpcClient {
    pub fn new(rpc: &str) -> Result<Self> {
        Ok(Self {
            trusted_blocks: HashMap::new(),
            rpc: HttpRpc::new(rpc)?,
        })
    }

    pub fn add_trusted_blocks(&mut self, blocks: &[(BlockNumber, StateRoot)]) {
        for (number, state_root) in blocks {
            self.trusted_blocks.insert(*number, *state_root);
        }
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
        let block_state_root = self
            .trusted_blocks
            .get(&block_number)
            .ok_or_else(|| eyre!("Block {block_number} has no trusted state root"))?;

        // Get account proof from the RPC
        let proof = self.rpc.get_proof(address, slots, block_number).await?;
        // Get account code from the RPC
        let code = self.rpc.get_code(address, tag).await?;

        // MOST IMPORTANT!!
        // Verify the proof fetched from RPC against the trusted state root
        verify_rpc_proof(&proof, &code, &block_state_root)?;

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
}
