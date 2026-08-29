# Lab 02 — Legacy P2PKH

## Commands used

I formatted the implementation, ran each focused public test during development,
and then ran the complete Lab 2 test suite:

```bash
cargo fmt
cargo test --test lab_02 derives_the_expected_p2pkh_address
cargo test --test lab_02 builds_the_standard_p2pkh_lock
cargo test --test lab_02 commits_to_hash160_of_the_public_key
cargo test --test lab_02 puts_unlocking_data_in_scriptsig
cargo test --test lab_02
```

## Terminal output

The complete Lab 2 test suite passed all four public tests:

```text
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test derives_the_expected_p2pkh_address ... ok
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

While inspecting the test key during development, I recorded these derived values:

```text
Compressed public key:
024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766

HASH160 commitment:
ebc0ee0b2ab9e8277a600c251475e22a3241a1c1

P2PKH scriptPubKey:
76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac
```

## Evidence references

- Implementation: `src/labs/lab02_p2pkh.rs`
- Public tests: `tests/lab_02.rs`
- `derive_p2pkh_address` parses the serialized public key and derives its
  network-specific P2PKH address.
- `build_p2pkh_script_pubkey` and `committed_pubkey_hash` demonstrate that the
  locking script contains the key's HASH160 commitment.
- `p2pkh_spend_template` places the signature and public key in ScriptSig and
  leaves the SegWit witness empty.

## Explanation

<!-- Write your explanation here. Describe what the P2PKH script commits to, what
the spender supplies in ScriptSig, and how key identity differs from authorization. -->
