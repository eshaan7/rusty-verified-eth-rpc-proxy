use alloy::consensus::constants::KECCAK_EMPTY;
use alloy::consensus::Account;
use alloy::primitives::{keccak256, Bytes, B256};
use alloy::rlp;
use alloy::rpc::types::EIP1186AccountProofResponse;
use alloy_trie::proof::verify_proof as mpt_verify_proof;
use alloy_trie::Nibbles;
use eyre::{eyre, Ok, Result};

pub fn proof_to_account(proof: &EIP1186AccountProofResponse) -> Account {
    Account {
        nonce: proof.nonce,
        balance: proof.balance,
        storage_root: proof.storage_hash,
        code_hash: proof.code_hash,
    }
}

/// Source: https://github.com/a16z/helios/blob/545d809be0f135f69a8e6f613bb6bdd0fb4b22d1/execution/src/execution.rs#L43-L119
pub fn verify_rpc_proof(
    proof: &EIP1186AccountProofResponse,
    code: &Bytes,
    state_root: &B256,
) -> Result<()> {
    verify_account_proof(proof, state_root)?;
    verify_storage_proof(proof)?;
    verify_code_hash(proof, code)?;

    Ok(())
}

pub fn verify_account_proof(proof: &EIP1186AccountProofResponse, state_root: &B256) -> Result<()> {
    let account_key = Nibbles::unpack(keccak256(proof.address));
    let account = proof_to_account(proof);
    let account_encoded_value = rlp::encode(account);

    mpt_verify_proof(
        *state_root,
        account_key,
        account_encoded_value.into(),
        &proof.account_proof,
    )
    .map_err(|e| eyre!("Failed to verify account proof: {e}"))
}

pub fn verify_storage_proof(proof: &EIP1186AccountProofResponse) -> Result<()> {
    for storage_proof in &proof.storage_proof {
        let key = storage_proof.key.0;
        let key_hash = keccak256(key);
        let key_nibbles = Nibbles::unpack(key_hash);
        let encoded_value = rlp::encode(storage_proof.value);

        mpt_verify_proof(
            proof.storage_hash,
            key_nibbles,
            encoded_value.into(),
            &storage_proof.proof,
        )
        .map_err(|e| eyre!("Failed to verify storage proof: {e}"))?;
    }

    Ok(())
}

pub fn verify_code_hash(proof: &EIP1186AccountProofResponse, code: &Bytes) -> Result<()> {
    if proof.code_hash != KECCAK_EMPTY {
        let code_hash = keccak256(&code);

        if proof.code_hash != code_hash {
            return Err(eyre!(
                "Code hash mismatch for address {:?}: expected {:?}, got {:?}",
                proof.address,
                proof.code_hash,
                code_hash
            ));
        }
    }

    Ok(())
}
