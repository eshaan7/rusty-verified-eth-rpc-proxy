use alloy::primitives::B256;
use alloy::{
    consensus::{Receipt, ReceiptWithBloom, TxReceipt, TxType},
    rlp,
    rpc::types::TransactionReceipt,
};
use alloy_trie::root::ordered_trie_root_with_encoder;

pub fn encode_receipt(receipt: &TransactionReceipt) -> Vec<u8> {
    let tx_type = receipt.transaction_type();
    let receipt_with_bloom = receipt.inner.as_receipt_with_bloom().unwrap();
    let logs = receipt
        .inner
        .logs()
        .iter()
        .map(|l| l.inner.clone())
        .collect::<Vec<_>>();

    let consensus_receipt = Receipt {
        cumulative_gas_used: receipt.inner.cumulative_gas_used(),
        status: receipt.inner.status_or_post_state(),
        logs,
    };

    let rwb = ReceiptWithBloom::new(consensus_receipt, receipt_with_bloom.bloom());
    let encoded = rlp::encode(rwb);

    match tx_type {
        TxType::Legacy => encoded,
        _ => [vec![tx_type as u8], encoded].concat(),
    }
}

pub fn encode_receipt_logs(receipt: &TransactionReceipt) -> Vec<Vec<u8>> {
    let encoded_logs = receipt
        .inner
        .logs()
        .iter()
        .map(|l| rlp::encode(&l.inner))
        .collect::<Vec<_>>();

    encoded_logs
}

/// Compute a trie root of the collection of encoded items.
/// Ref: https://github.com/alloy-rs/trie/blob/main/src/root.rs.
pub fn ordered_trie_root(items: &[Vec<u8>]) -> B256 {
    fn noop_encoder(item: &Vec<u8>, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(item);
    }

    ordered_trie_root_with_encoder(items, noop_encoder)
}
