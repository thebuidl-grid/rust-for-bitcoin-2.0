# Lab 08 — Block security

## Commands used

Rust: `cargo run --example run` (calls `build_security_report`, which runs):
- `getblockheader <block_hash>` — verbose header inspection
- `gettransaction <txid>` (wallet: receiver) — confirmations before mining more blocks
- `generatetoaddress 5 <miner_address>` — mine 5 additional blocks
- `gettransaction <txid>` (wallet: receiver) — confirmations after

## Terminal output

=== Lab 08: security / block header ===
SecurityReport {
header: BlockHeaderEvidence {
hash: "20cd23ecb5398b06d18f803acb034a128c2d706b22f059c03d8233c41f354e65",
height: 103,
previous_block_hash: Some(
"24519ecb268e54d62204f63d141a76fff35c0e806a2ea132263af98fd9928a7b",
),
merkle_root: "3514217e5fa43d8828582c0afd4c9dae7c770abd64ecadf39c3fe72a4595b660",
nonce: 5,
difficulty: 4.6565423739069247e-10,
bits: "207fffff",
confirmations: 1,
chainwork: "00000000000000000000000000000000000000000000000000000000000000d0",
},
confirmations_before: 1,
confirmations_after: 6,
}

## Evidence references

Screenshot: `evidence/lab08.png`

## Explanation

Every block header commits to two things: `previous_block_hash` links it backward to the block before it, forming the actual "chain" in blockchain — rewriting any past block would change its hash, breaking every link after it. `merkle_root` commits to the exact set of transactions inside this block; changing even one transaction, or their order, would produce a different Merkle root, so the header cryptographically proves the block's contents without needing to list every transaction in the header itself.

`nonce`, `bits`, and `difficulty` are all proof-of-work related. `bits` (here `207fffff`, regtest's minimum-difficulty setting) encodes the target a block's hash must be below to be valid; `nonce` is the value miners vary while searching for a header hash that meets that target — it has no meaning on its own, it's just the field that gets brute-forced. `difficulty` is a human-readable representation of how hard that target currently is to hit (my regtest value is deliberately tiny, since regtest mines instantly on demand rather than requiring real computational work).

Mining 5 more blocks after this one pushed confirmations from 1 to 6 — each additional block mined on top adds more accumulated proof-of-work behind this block and the transaction inside it. Confirmations don't make an invalid transaction valid or retroactively fix a bad signature — validity was already fully determined the moment the block was accepted. What confirmation depth increases is the *cost* of undoing it: reversing a transaction buried under 6 blocks would require an attacker to out-mine 6 blocks' worth of proof-of-work to build a longer competing chain, which is exactly the kind of chainwork race demonstrated directly in Lab 10.
