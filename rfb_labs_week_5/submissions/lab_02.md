# Lab 02 — Legacy P2PKH construction

## Commands used

```bash
cargo test --test lab_02
cargo run -- 2
```

The runner starts from one fixed compressed public key and prints its HASH160, its
regtest P2PKH address, the scriptPubKey those bytes produce, and the ScriptSig items a
spender would supply.

## Terminal output

```text
$ cargo test --test lab_02
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test commits_to_hash160_of_the_public_key ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 2
== Lab 02: Legacy P2PKH ==
  public key    024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
  HASH160       ebc0ee0b2ab9e8277a600c251475e22a3241a1c1
  address       n31WD8pkfAjg2APV78GnbDTdZb1QonBi5D
  scriptPubKey  76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac
  ScriptSig     ["30440220deadbeef01", "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766"]
  witness       []
```

## Evidence references

Implementation in `src/labs/lab02_p2pkh.rs`, tests in `tests/lab_02.rs`, runner in
`src/main.rs` under `lab02`.

The same twenty bytes turn up three times in the output.
`ebc0ee0b2ab9e8277a600c251475e22a3241a1c1` is the HASH160 result. It is the payload
Base58Check encodes into `n31WD8pkfAjg2APV78GnbDTdZb1QonBi5D`. It is also the push
sitting between `76a914` and `88ac` in the scriptPubKey.

`30440220deadbeef01` is a placeholder, not a real signature. No signing is needed to
show where the item goes. The witness list is empty because legacy inputs have no
witness at all.

## Explanation

The output commits to a key without naming one. `76a914 <hash> 88ac` decodes to
OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG, and the only thing on chain
at funding time is RIPEMD160(SHA256(pubkey)). The public key stays unpublished until
the coin is spent. That gap separates key identity from spend authorization.

Identity is settled by the second ScriptSig item. The spender pushes a public key, the
script hashes it, and OP_EQUALVERIFY checks the result against the committed value.
This answers whether the pushed key is the one the output was addressed to. It says
nothing about permission. Anyone who has seen a previous spend from that address can
copy the public key out of the chain and push it again.

Authorization is settled by the first item. OP_CHECKSIG takes the signature and the
verified public key, recomputes the sighash over the transaction being validated, and
checks the ECDSA signature against it. Only the private key produces a signature that
passes. The sighash covers the spending transaction, so the signature is tied to that
spend and cannot be reused elsewhere.

The push order follows from this. Signature first, public key second, so the key ends
up on top ready for OP_DUP and OP_HASH160, with the signature underneath waiting for
OP_CHECKSIG.

Legacy inputs keep both items inside the ScriptSig, which is part of the txid preimage.
A third party could re-encode the signature, change the txid and leave the transaction
otherwise valid. That was the malleability problem. Lab 04 shows the same two items in
the witness instead, which is how segregated witness fixed it.
