# Lab 08 — Block security

## Commands used

`cargo test --test lab_08
cargo run --example lab08_check
`

## Terminal output

`SecurityReport {
    header: BlockHeaderEvidence {
        hash: "2e69f34a27acdd327da77e12aaeb42e7812b0c5977392976b4e3b315a03698d9",
        height: 103,
        previous_block_hash: Some(
            "28a3aa19ee86f171e19c8258ab3d3deb1bc10ed6592b29c8e2be9c38d3ecbb83",
        ),
        merkle_root: "f92dba33d8457dd06d90d9b14bc7e8d856fb85c712462ebbf57b8022284c4049",
        nonce: 0,
        difficulty: 4.6565423739069247e-10,
        bits: "207fffff",
        confirmations: 3,
        chainwork: "00000000000000000000000000000000000000000000000000000000000000d0",
    },
    confirmations_before: 3,
    confirmations_after: 8,
}
`

## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

A block header ties three separate mechanisms together to make the blockchain tamper-evident. The hash link is previous_block_hash — my block at height 103 explicitly embeds the hash of the block before it (28a3aa19...). This means every header is cryptographically bound to its predecessor: if anyone altered a past block, that block's own hash would change, which would break the previous_block_hash reference stored in the next block, cascading forward through every block mined since. Rewriting history isn't just "editing a record" — it requires re-mining the altered block and every single block built on top of it.

The Merkle commitment is the merkle_root field — a single hash that summarizes every transaction inside the block, Changing even one byte of one transaction in that block would produce a completely different Merkle root, which would change the block's own hash, which would in turn break the hash link described above. This is what lets a block header "commit" to an entire set of transactions using a fixed-size, 32-byte value, without needing to store the full transaction list in the header itself.

The proof-of-work search is represented by nonce and bits. bits (207fffff here) encodes the target difficulty a valid block hash must meet; nonce is the value miners increment while repeatedly hashing the header, searching for a result that satisfies that target. On my regtest chain, bits is set to the easiest possible target (difficulty: 4.66e-10), which is why blocks mine instantly

Validity and confirmation depth are answering two completely different questions. A transaction's validity — correct signatures, no double-spend, outputs not exceeding inputs — is checked against a fixed set of consensus rules the instant it's considered, and no amount of additional mining changes that answer. Piling blocks on top of an already-valid transaction cannot retroactively make a broken transaction correct.

What confirmations actually buy is economic finality against reorganization. To reverse a transaction buried N blocks deep, an attacker would need to build an alternative chain that is longer (has more accumulated work) than the honest chain from that point forward — meaning they'd have to out-mine N blocks' worth of proof-of-work from scratch while the rest of the network keeps extending the honest chain in parallel. My own report shows this accumulating directly: chainwork is a running total of proof-of-work invested in the entire chain up to that block, and it only grows. Each additional confirmation isn't "more proof the transaction is correct" — it's more accumulated work an attacker would have to overcome to erase it, which is why six confirmations is treated as far safer than one, even though both are equally "valid" from a rules standpoint.
