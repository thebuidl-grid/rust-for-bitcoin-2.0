# Lab 04 — Native SegWit P2WPKH

## Commands used

```bash
cargo test --test lab_04
cargo run -- 4
```

The runner derives a `bcrt1q` address from one fixed compressed public key, prints its
version 0 witness program, and models the unlocking data with an empty ScriptSig.

## Terminal output

```text
$ cargo test --test lab_04
running 4 tests
test reports_a_twenty_byte_program ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test builds_a_version_zero_witness_lock ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 4
== Lab 04: Native P2WPKH ==
  public key    03462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b
  address       bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm
  scriptPubKey  0014888dbbd7998f9f80d4f82b4f9ce6d5882f310aec
  witness program v0 888dbbd7998f9f80d4f82b4f9ce6d5882f310aec (20 bytes)
  ScriptSig     "" (empty)
  witness       ["30440220cafebabe01", "03462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b"]
```

## Evidence references

Implementation in `src/labs/lab04_p2wpkh.rs`, tests in `tests/lab_04.rs`, runner in
`src/main.rs` under `lab04`.

The scriptPubKey `0014888dbbd7998f9f80d4f82b4f9ce6d5882f310aec` is twenty-two bytes.
`00` is OP_0, the witness version. `14` is a 20-byte push length. The rest is the
program. The witness program line repeats those twenty bytes as v0, length 20, so the
scriptPubKey and the address `bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm` encode the
same commitment.

The ScriptSig prints as `""` and the two unlocking items sit in the witness list. The
Lab 02 output has the same pair of items in script_sig_items with an empty witness.

native_spend_template rejects an uncompressed public key before modelling anything,
since BIP143 only defines P2WPKH for compressed keys.

## Explanation

P2WPKH commits to the same HASH160(pubkey) as P2PKH. What changes is where the proof
goes and how the transaction is serialized.

Against P2PKH the scriptPubKey drops from twenty-five bytes of opcodes to twenty-two
bytes with no script logic in them. There is no OP_DUP, no OP_EQUALVERIFY and no
OP_CHECKSIG in the output. The witness program is a pattern consensus recognises:
version 0 with a 20-byte program tells the node to build the implied P2PKH script
internally and validate it under BIP143 using the witness stack. The signature and
public key leave the ScriptSig, which now has to be completely empty. The ScriptSig is
part of the txid preimage and the witness is not, so a third party can no longer change
the txid by re-encoding a signature. That also made chains of unconfirmed transactions
safe to build on.

Against P2SH-wrapped SegWit the difference is packaging, not rules. The wrapped form
puts the same witness program inside a redeemScript, hashes it again, and publishes an
ordinary `2...` or `3...` P2SH address. Its ScriptSig is not empty: it holds that single
redeemScript push, and those bytes are non-witness bytes charged at the full rate. So
wrapped SegWit costs more than native for the same spending policy, and what it buys is
compatibility with senders that cannot read bech32. Lab 05 covers who those senders
are and Lab 06 puts a number on the difference.

Two more things follow from the encoding. The address is bech32, so it is case
insensitive and has a stronger checksum, but older wallets cannot parse it at all. And
a node that does not understand witness version 0 sees an output anyone could claim,
which is why segregated witness shipped as a soft fork with upgraded nodes enforcing
the real rule.
