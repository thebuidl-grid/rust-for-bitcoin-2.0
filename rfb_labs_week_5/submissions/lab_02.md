# Lab 02 — Legacy P2PKH

## Commands used

```bash
cargo test --test lab_02 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_02 -- --nocapture
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 02 section, disposable key [2u8; 32])
derive_p2pkh_address = Ok("1NVYv5jmr9JRF3usPZJQmJFJhbQhrPESTP")
build_p2pkh_script_pubkey = Ok("76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac")
committed_pubkey_hash = Ok("ebc0ee0b2ab9e8277a600c251475e22a3241a1c1")
p2pkh_spend_template = Ok(P2pkhSpendTemplate {
    script_sig_items: ["30440220deadbeef01",
                        "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766"],
    witness_items: [] })
```

## Evidence references

- Implementation: [`src/labs/lab02_p2pkh.rs`](../src/labs/lab02_p2pkh.rs)
- Public test suite: [`tests/lab_02.rs`](../tests/lab_02.rs) — 4/4 passing, logged in
  [`grading/logs/lab_02.log`](../grading/logs/lab_02.log).
- `build_p2pkh_script_pubkey` output decodes to
  `OP_DUP OP_HASH160 <ebc0ee0b...a1c1> OP_EQUALVERIFY OP_CHECKSIG`, i.e. `76 a9 14 <20
  bytes> 88 ac`, matching the standard P2PKH template byte-for-byte.
- The public key used (`024d4b6c...54d0766`) is derived from the disposable secret
  `[2u8; 32]`, never a real key.

## Explanation

P2PKH separates two things that are easy to conflate: **key identity** and **spend
authorization**.

- The **locking script** (`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`,
  built in `build_p2pkh_script_pubkey`) only commits to the HASH160 of a public key —
  `committed_pubkey_hash` in this lab. That commitment is *identity*: it says "whoever
  can produce a public key hashing to this value, and a valid signature under it, may
  spend this output." It does not by itself prove anyone owns the coin yet.
- **Spend authorization** happens later, at spend time, in ScriptSig
  (`p2pkh_spend_template`). The spender must supply the actual public key (so the
  script can re-hash it and check it against the committed hash with
  `OP_EQUALVERIFY`) *and* a valid ECDSA signature over the spending transaction (so
  `OP_CHECKSIG` can verify control of the matching private key).

So HASH160(pubkey) alone only proves *which* key is allowed to spend — it is a
commitment to identity, checkable by anyone with the public key. Only a fresh signature
produced with the corresponding private key, evaluated by `OP_CHECKSIG` against the
current transaction, proves *authorization* to spend. That is why revealing a public
key is safe (it is already committed to via the hash) while a signature is only
produced once, at spend time, over the specific transaction being authorized.
