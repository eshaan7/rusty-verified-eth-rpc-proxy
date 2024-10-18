# rusty-verified-eth-rpc-proxy

A Rust crate that turns an untrusted Ethereum JSON-RPC Execution API provider into a verified data source by requesting Merkle proofs and checking them against the state hash.

Refer to the tests in [`src/lib.rs`](/src/lib.rs) for example usage.

## Inspiration

- [Helios](https://github.com/a16z/helios)
- [Nimbus Verified Proxy](https://github.com/status-im/nimbus-eth1/tree/master/nimbus_verified_proxy)
- [Lodestar prover](https://github.com/ChainSafe/lodestar/tree/unstable/packages/prover)
