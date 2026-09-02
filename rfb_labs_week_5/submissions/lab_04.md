# Lab 04 — Native P2WPKH

## Commands used

TODO: List the Rust commands you ran.

## Terminal output

TODO: Record the address, witness program, ScriptSig, and witness items.

## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

Native P2WPKH locks an output with the version-0 witness program
`OP_0 <20-byte HASH160(compressed public key)>`. Its Bech32 address encodes this
witness version and program using a network-specific human-readable prefix such as
`bc`, `tb`, or `bcrt`.

When the output is spent, the signature and compressed public key are placed in the
transaction's witness rather than in ScriptSig. ScriptSig remains empty because
there is no legacy redeemScript to reveal for a native witness output. Bitcoin
hashes the supplied public key, compares it with the 20-byte witness program, and
uses the same key to verify the signature. The hash match identifies the committed
key, while the valid signature proves spending authorization. This differs from
legacy P2PKH, which puts both items in ScriptSig, and from P2SH-wrapped P2WPKH, whose
ScriptSig contains the wrapped witness program while its signature and public key
remain in the witness.
