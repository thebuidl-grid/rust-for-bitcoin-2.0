# Lab 10 — Competing branches and reorganization

## Commands used

TODO: Record peer, mining, chain-tip, and reconnection commands for both nodes.
# Lab 10 — Competing branches and reorganization

## Commands used

- `cargo test --test lab_10` — runs the public unit tests against a mocked RPC client.
- `bitcoin-cli -regtest getblockchaininfo` (both nodes) — record the common tip before the split.
- `bitcoin-cli -regtest getpeerinfo` — find the other node's peer address.
- `bitcoin-cli -regtest disconnectnode <address>` — disconnect Node A and Node B from each other.
- `bitcoin-cli -regtest generatetoaddress 2 <address>` (Node A) — mine 2 blocks privately on Node A.
- `bitcoin-cli -regtest generatetoaddress 1 <address>` (Node B) — mine 1 block privately on Node B.
- `bitcoin-cli -regtest getblockchaininfo` (both nodes) — record each node's competing (diverged) tip.
- `bitcoin-cli -regtest addnode <address> onetry` — reconnect the two nodes.
- `bitcoin-cli -regtest getblockchaininfo` (both nodes) — record the final, converged tip.



## Terminal output

TODO: Show the common tip, competing tips, chainwork, and final convergence.
 cargo test --test lab_10
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/lab_10.rs (target/debug/deps/lab_10-04fc6760f55d26e8)

running 4 tests
test disconnects_peer_by_address ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok
test reads_tip_and_accumulated_chainwork ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

TODO: Explain the stale branch, reorganization, and most-work-chain rule.
- **Stale branch:** the "losing" version of the chain — blocks that were valid when mined, but got dropped once a competing chain with more work showed up.

- **Reorganization:** a node switching from its own chain to a different one because that one has more work behind it. It undoes its own blocks and adopts the other chain instead.

- **Most-work rule:** when two chains compete, the one with more total proof-of-work wins , not just whichever has more blocks.