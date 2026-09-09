# Lab 02 — Legacy P2PKH

## Commands used

```
cargo test --test lab_02 -- --nocapture
```

## Terminal output

```
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test commits_to_hash160_of_the_public_key ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Using the disposable test secret key `[0x02; 32]`, compressed public key
`024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766`:

```
p2pkh address:  1NVYv5jmr9JRF3usPZJQmJFJhbQhrPESTP
scriptPubKey:   76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac
pubkey hash:    ebc0ee0b2ab9e8277a600c251475e22a3241a1c1
```

`build_p2pkh_script_pubkey` reproduces `OP_DUP OP_HASH160 <ebc0ee...a1c1> OP_EQUALVERIFY
OP_CHECKSIG` byte-for-byte, and `committed_pubkey_hash` returns exactly the 20-byte
HASH160 embedded in that script. `p2pkh_spend_template("30440220deadbeef01", <pubkey>)`
placed both items in `script_sig_items` and left `witness_items` empty, matching legacy
(pre-SegWit) spending.

## Explanation

P2PKH separates *key identity* from *spend authorization*. The scriptPubKey only commits
to the HASH160 of the public key — it identifies who is allowed to spend the output
without ever revealing the public key itself on-chain until spend time. Spend
authorization happens later, in ScriptSig, where the spender must supply both the
original public key (so the hash can be verified) and a valid ECDSA signature over the
spending transaction (so `OP_CHECKSIG` can verify ownership of the matching private key).
`OP_EQUALVERIFY` only proves "this is the public key that was committed to" — it says
nothing about authorization; `OP_CHECKSIG` is what actually proves the spender controls
the private key. Knowing a public key that hashes correctly is not enough to spend: an
attacker would also need a valid signature, which requires the private key.
