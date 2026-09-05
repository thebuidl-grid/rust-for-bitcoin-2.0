# Lab 02 — Legacy P2PKH

## Commands used

TODO: List the Rust commands you ran.
```
cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_02
```

## Terminal output

TODO: Record the address, public-key hash, and scriptPubKey.

```
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test commits_to_hash160_of_the_public_key ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed
```
The lab successfully:

derived a P2PKH address from a compressed public key
built the standard P2PKH scriptPubKey
calculated the HASH160 commitment of the public key
placed the signature and public key in the ScriptSig
confirmed that the witness is empty for legacy P2PKH

## Evidence references

TODO: Link screenshots or describe attached evidence.

The public test output above is the primary execution evidence for this lab.

No Bitcoin Core, Polar, or live funds are required for this lab.

## Explanation

P2PKH means **Pay to Public Key Hash**. The basic idea is that when someone receives
Bitcoin, the output does not simply say "this person owns these coins." Instead, it
contains a locking script that says, in effect, "to spend these coins, you must
provide a public key that hashes to this particular value, and you must prove that
you control the corresponding private key."

The first part happens when the output is created. Given a public key, we calculate
its HASH160 and put that hash into the scriptPubKey:

OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG

For example, `committed_pubkey_hash` takes the public key and calculates exactly
this HASH160 commitment. No signature is involved at this point. The resulting hash
is simply a commitment to which public key is expected to be revealed later.

`build_p2pkh_script_pubkey` constructs the actual locking script. The opcodes tell
Bitcoin Script what must happen when somebody later tries to spend the output:

1. `OP_DUP` duplicates the public key that the spender provides.
2. `OP_HASH160` hashes that copy of the public key.
3. `<pubKeyHash>` is the hash that was committed to when the output was created.
4. `OP_EQUALVERIFY` checks that the supplied public key hashes to the committed hash.
5. `OP_CHECKSIG` verifies that the supplied signature is valid for the transaction
   and was produced by the private key corresponding to that public key.

When the owner wants to spend the output, they provide two pieces of information:
a signature and their public key. `p2pkh_spend_template` represents these as the
two ScriptSig items:

[signature, public_key]

The public key allows Bitcoin to check **which key is being used** by hashing it and
comparing the result with the hash committed in the output. The signature then proves
**that the spender actually controls the private key** corresponding to that public
key.

This distinction is important. Knowing or providing the correct public key is not
enough to spend the coins. Someone could know the public key and calculate its
HASH160, but they still could not create a valid signature without the corresponding
private key. P2PKH therefore combines two checks: the HASH160 check identifies the
expected public key, while `OP_CHECKSIG` proves control of the corresponding private
key.

Because this is a legacy P2PKH spend, the signature and public key go into the
ScriptSig rather than the witness. That is why `p2pkh_spend_template` returns an
empty `witness_items` vector.