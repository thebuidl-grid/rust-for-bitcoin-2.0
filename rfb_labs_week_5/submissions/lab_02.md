# Lab 02 — Legacy P2PKH

## Commands used

```bash
cargo test --test lab_02
bash grader/grade.sh
```

## Terminal output

running 4 tests

test commits_to_hash160_of_the_public_key ... ok

test builds_the_standard_p2pkh_lock ... ok

test derives_the_expected_p2pkh_address ... ok

test puts_unlocking_data_in_scriptsig ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

All four public tests in `tests/lab_02.rs` pass, covering:
- Deriving a P2PKH address from a compressed public key
  (`derives_the_expected_p2pkh_address`)
- Building the standard `OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG`
  scriptPubKey (`builds_the_standard_p2pkh_lock`)
- Computing the HASH160 commitment made to the public key
  (`commits_to_hash160_of_the_public_key`)
- Placing a signature and public key in ScriptSig, with an empty witness
  (`puts_unlocking_data_in_scriptsig`)

## Explanation

A public key is a key's *identity* — it's the raw data that gets hashed
(HASH160) and committed to inside the scriptPubKey when the output is
created. Anyone can see it once it's revealed, and by itself it proves
nothing about who can spend the coins.

Spend *authorization* comes from the signature: `OP_CHECKSIG` inside the
scriptPubKey verifies that the signature provided in ScriptSig is a valid
ECDSA signature over the spending transaction, produced by the private key
that corresponds to the committed public key. Only whoever holds that
private key can produce a valid signature — the public key alone, without a
matching signature, cannot unlock the output.

So P2PKH's locking script encodes an identity commitment (the HASH160 of the
public key), while spending requires proving control of that identity by
supplying both the public key (to check it hashes to the committed value)
*and* a signature (to prove ownership of the corresponding private key).
Legacy P2PKH puts both of these directly in ScriptSig, with no witness data
at all — that separation of "locking to an identity" versus "unlocking by
proving control of it" is the core idea behind every Bitcoin script type,
not just P2PKH.

