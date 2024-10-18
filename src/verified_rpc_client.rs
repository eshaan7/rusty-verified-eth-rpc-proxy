use std::collections::HashMap;

use alloy::consensus::Account;
use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{keccak256, Address, B256, U256};
use alloy_trie::proof::verify_proof;
use alloy_trie::Nibbles;
use eyre::{Ok, Result};

use crate::common::RpcVerifiableMethods;
use crate::http_rpc::HttpRpc;

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

        // Ensure we have this block in our trusted blocks
        let block_state_root = self.trusted_blocks.get(&block_number).ok_or_else(|| {
            eyre::eyre!(format!("Block {} has no trusted state root", block_number))
        })?;

        // Get the proof from the RPC
        let proof = self.rpc.get_proof(address, slots, block_number).await?;

        // MOST IMPORTANT!!
        // Verify the proof fetched from RPC against the trusted state root
        let account_key = Nibbles::unpack(keccak256(address)); // key in the trie
        let account = Account {
            nonce: proof.nonce,
            balance: proof.balance,
            storage_root: proof.storage_hash,
            code_hash: proof.code_hash,
        };
        let account_encoded_value = alloy::rlp::encode(account); // value in the trie
        verify_proof(
            *block_state_root,
            account_key,
            account_encoded_value.into(),
            &proof.account_proof,
        )
        .map_err(|e| eyre::eyre!(format!("Failed to verify proof: {:?}", e)))?; // throw error if verification fails

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
}
