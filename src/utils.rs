use alloy::primitives::B256;
use alloy::{
    consensus::{Receipt, ReceiptWithBloom, TxReceipt, TxType},
    rlp,
    rpc::types::TransactionReceipt,
};
use alloy_trie::{nybbles::Nibbles, HashBuilder, EMPTY_ROOT_HASH};

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

/// Adjust the index of an item for rlp encoding.
/// Modified version of https://github.com/alloy-rs/trie/blob/main/src/root.rs.
fn adjust_index_for_rlp(i: usize, len: usize) -> usize {
    if i > 0x7f {
        i
    } else if i == 0x7f || i + 1 == len {
        0
    } else {
        i + 1
    }
}

/// Compute a trie root of the collection of encoded items.
/// Modified version of https://github.com/alloy-rs/trie/blob/main/src/root.rs.
pub fn ordered_trie_root(items: &[Vec<u8>]) -> B256 {
    if items.is_empty() {
        return EMPTY_ROOT_HASH;
    }

    let mut hb = HashBuilder::default();
    let items_len = items.len();
    for i in 0..items_len {
        let index = adjust_index_for_rlp(i, items_len);
        let index_buffer = rlp::encode_fixed_size(&index);
        let value_buffer = &items[index];

        hb.add_leaf(Nibbles::unpack(&index_buffer), value_buffer);
    }

    hb.root()
}
