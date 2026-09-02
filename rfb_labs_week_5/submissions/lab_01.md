# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01
cargo run -- 1
```

The runner builds four regtest addresses from one fixed test key, one per script
family, and puts each through identify_prefix, inspect_address and expected_prefix.
The last line hands a valid mainnet address to the regtest checker.

## Terminal output

```text
$ cargo test --test lab_01
running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 1
== Lab 01: Address formats and networks ==
  mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ
    prefix guess P2pkh | parsed P2pkh | network regtest
    expected prefix "m/n" | scriptPubKey 76a91479b000887626b294a914501a4cd226b58b23598388ac
  2N5CqcsNGUJBFzYNPU3kGWWzcY2T4yqXXcL
    prefix guess P2sh | parsed P2sh | network regtest
    expected prefix "2" | scriptPubKey a914832e012d4cd5f23df82efd34e473345a2f8aa4fb87
  bcrt1q0xcqpzrky6eff2g52qdye53xkk9jxkvrl4xfg5
    prefix guess P2wpkh | parsed P2wpkh | network regtest
    expected prefix "bcrt1q" | scriptPubKey 001479b000887626b294a914501a4cd226b58b235983
  bcrt1p33wm0auhr9kkahzd6l0kqj85af4cswn276hsxg6zpz85xe2r0y8s7hfsm7
    prefix guess P2tr | parsed P2tr | network regtest
    expected prefix "bcrt1p" | scriptPubKey 51208c5db7f797196d6edc4dd7df6048f4ea6b883a6af6af032342088f436543790f
  1BoatSLRHtKNngkdXEeobR76b53LETtpyT
    prefix guess P2pkh | regtest check rejected: address network mismatch: validation error
```

## Evidence references

Implementation in `src/labs/lab01_addresses.rs`, tests in `tests/lab_01.rs`, runner in
`src/main.rs` under `lab01`.

The four scriptPubKeys show the opcode shape of each family. `76a914...88ac` is
OP_DUP OP_HASH160, a 20-byte push, OP_EQUALVERIFY, OP_CHECKSIG. `a914...87` is
OP_HASH160, a 20-byte push, OP_EQUAL. `0014...` is witness version 0 with a 20-byte
program, and `5120...` is witness version 1 with a 32-byte program.

The P2PKH and P2WPKH lines both carry the hash
`79b000887626b294a914501a4cd226b58b235983`, because I built them from the same public
key. The last line is the one that matters most: `1BoatSLRHtKNngkdXEeobR76b53LETtpyT`
is a valid mainnet address and is still refused under Network::Regtest.

## Explanation

A prefix is a hint. identify_prefix looks at leading characters and nothing else, so it
will label a truncated or corrupted string quite happily. The public test suite feeds
it `bc1pexample`, which is not an address at all, and the function still answers P2TR.
Keeping that function separate from inspect_address is the point of the lab.

Three things have to happen before a string can be trusted.

The checksum comes first. Base58Check appends four bytes of double SHA256 over the
version byte and payload. Bech32 and bech32m append a six character BCH checksum. Both
catch mistyped and transposed characters, which a prefix check cannot see at all.

The network comes second. The leading character is a decoded version byte or a human
readable part, not decoration. `1` and `bc1` are mainnet. `m`, `n`, `2` and `tb1` are
the shared test networks, and `bcrt1` is regtest. Paying an address that was decoded
under the wrong network assumption is unrecoverable, so inspect_address calls
require_network and returns WrongNetwork. That produces the
`rejected: address network mismatch` line above.

The script family comes third, and it has to be read from the decoded payload, because
the payload is what becomes the scriptPubKey. Bech32 shows why. A `bcrt1q` prefix
covers P2WPKH and P2WSH both, and only the program length tells them apart: 20 bytes
for a single key, 32 for a script hash. My implementation checks the encoded length
before it claims P2WPKH, and still lets the parser give the final answer. Witness
version 1 with a 32-byte program is P2TR under BIP350. A decoder that only knows
bech32 will reject it, since BIP350 changed the checksum constant from 1 to
0x2bc830a3 so that older software fails safely.
