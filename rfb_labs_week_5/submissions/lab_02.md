# Lab 02 — Legacy P2PKH

## Commands used

```shell 
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_02
```

## Terminal output

```terminaloutput

Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
Running tests/lab_02.rs (target/debug/deps/lab_02-3772488ede065006)

running 4 tests
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

```
Code: src/labs/lab02_p2pkh.rs  
Test: tests/lab_02.rs
```

## Explanation

P2PKH (Pay-to-Public-Key-Hash) is the classic mainnet and testnet/regtest payment
script. It locks coins not to the public key directly but to a commitment of that
key. In `committed_pubkey_hash` and `build_p2pkh_script_pubkey` we compute
`HASH160(pubkey)` — `RIPEMD160(SHA256(pubkey))` — which condenses the 33-byte key
into a 20-byte digest.

**Locking.** The scriptPubKey is `OP_DUP OP_HASH160 <20-byte hash> OP_EQUALVERIFY
OP_CHECKSIG`. When a spend is validated, the script makes the spender provide a
public key whose HASH160 exactly matches the committed hash (`OP_EQUALVERIFY`),
then checks that the claimed signature validates against that key (`OP_CHECKSIG`).
Because the bytes on the chain are only the hash, the key is revealed only later at
spend time.

**Unlocking.** To spend, the user supplies a signature and the public key. In a
legacy P2PKH this goes into the `script_sig_items` field of the `ScriptSig`
(`<sig> <pubkey>`), and the `witness_items` list stays empty — P2PKH predates
SegWit, so all unlocking data rides in the ScriptSig and nothing is placed in the
witness. Combined, the stacked script asserts the hash matches and the signature is
valid before the coins are released.

The script separates **key identity** from **spend authorization**. The committed
20-byte hash is only proof of *which* key is entitled to the coins (identity); it is
`OP_CHECKSIG` that verifies the spender actually *owns* that key by checking they can
produce a valid signature for it (authorization). A matching hash alone proves
nothing about possession — only a successfully validated signature authorizes the
spend.

