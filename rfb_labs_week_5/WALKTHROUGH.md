# Week 5, explained from scratch

A plain-language walkthrough of what the ten Week 5 labs actually built, why the
Bitcoin rules work the way they do, and what the numbers in your own test run mean.
Every value below came out of an actual run against this code — you can paste any of
these hex strings or addresses back into the tests and get the same thing.

Week 2 modelled a transaction as data. Week 5 is the layer above that: how a public
key turns into the address you paste into a wallet, and how one seed phrase turns into
every address you'll ever use.

---

## Part 1 — Four address formats, one underlying idea

Every Bitcoin address is a wrapper around a **script** — a tiny program that has to
return true before a coin can be spent. The address format tells you which shape of
script you're dealing with before you even decode it.

Using the key `02...766` from Lab 02's test fixture:

```
pubkey:               024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
hash160(pubkey):       ebc0ee0b2ab9e8277a600c251475e22a3241a1c1
p2pkh address:          n31WD8pkfAjg2APV78GnbDTdZb1QonBi5D            (regtest)
p2pkh scriptPubKey:     76a914 ebc0ee0b2ab9e8277a600c251475e22a3241a1c1 88ac
```

Read that scriptPubKey byte by byte and it spells out the opcodes:
`76` = `OP_DUP`, `a9` = `OP_HASH160`, `14` = "push the next 20 bytes", then the hash
itself, then `88` = `OP_EQUALVERIFY`, `ac` = `OP_CHECKSIG`. That's
`OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG` — the P2PKH lock, in hex, with
nothing hidden. This is what Lab 02 (`build_p2pkh_script_pubkey`) constructs and what
Lab 01 (`inspect_address`) reads back out.

**The four formats, and what each one locks to:**

| Format | Locks to | Regtest prefix | Encoding |
|---|---|---|---|
| P2PKH | hash of a public key | `m`/`n` | Base58Check |
| P2SH | hash of a **script** (could be anything, e.g. multisig) | `2` | Base58Check |
| P2WPKH | hash of a public key, SegWit-style | `bcrt1q` | Bech32 |
| P2TR | a Taproot output key | `bcrt1p` | Bech32m |

The prefix is a convenience, not a proof — see the note on `require_network` below.

---

## Part 2 — P2SH: locking to a script instead of a key

P2SH (Lab 03) generalizes the idea in Part 1. Instead of committing to
`hash(pubkey)`, it commits to `hash(any script you like)`. The script you hid —
called the **redeemScript** — gets revealed only when someone spends the output.

The lab built a 2-of-3 multisig redeemScript from three throwaway keys:

```
redeemScript:   52 21<pub1> 21<pub2> 21<pub3> 53 ae
                = OP_2 <pub1> <pub2> <pub3> OP_3 OP_CHECKMULTISIG

p2sh address (regtest):   2N... (starts with 2)
outer scriptPubKey:       a914 <hash160(redeemScript)> 87
                         = OP_HASH160 <scriptHash> OP_EQUAL
```

Two completely separate checks happen when this gets spent. First, the outer script
just confirms "the script you're handing me hashes to what I committed to" — anyone
who's ever seen the redeemScript can pass that part, it proves nothing about who owns
keys. Second, once that passes, the redeemScript itself runs as if it were the real
lock, and *that's* where `OP_CHECKMULTISIG` actually demands two real signatures out
of the three named keys. Matching a hash and proving key ownership are not the same
thing — P2SH just lets you defer revealing the real rule until spend time.

---

## Part 3 — SegWit: moving the unlock data out of ScriptSig

Lab 04 built native SegWit (P2WPKH) from the same key type used in Lab 02:

```
pubkey (compressed):    03...4b (Lab 04's fixture key)
p2wpkh address:          bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm
p2wpkh scriptPubKey:     0014 888dbbd7998f9f80d4f82b4f9ce6d5882f310aec
```

`0014` is `OP_0` (witness version 0) followed by a push of 20 bytes — that whole
scriptPubKey *is* the witness program, nothing more. Compare that to P2PKH's
scriptPubKey above, which is five separate opcodes.

The real change is where the unlock data lives:

- **P2PKH**: signature + pubkey go in ScriptSig, counted at full weight.
- **P2SH-wrapped SegWit**: outer script still looks legacy (so old wallets can build
  an output to it), but the hidden redeemScript is a P2WPKH program — ScriptSig
  carries only that small redeemScript push, the real signature moves to witness.
- **Native P2WPKH**: no legacy wrapper at all. ScriptSig is empty. Everything is in
  the witness field.

Why bother moving it? BIP141 only discounts the **witness** field, not the whole
transaction — see Part 5.

---

## Part 4 — Why an old wallet accepts `3...` but chokes on `bc1q...`

Lab 05 modeled sender compatibility as four independent booleans instead of one
"is this wallet modern" flag, because that's genuinely how it works:

- `3...` (P2SH) decodes with the same Base58Check math as `1...` — same alphabet,
  same checksum, just a different version byte. A wallet from 2012 can build a P2SH
  output blind, with zero idea what's hidden inside.
- `bc1q...` uses Bech32 (BIP173), a completely different alphabet and checksum
  scheme introduced years later, specifically for SegWit. A pre-BIP173 wallet has no
  decoder for that string at all — it's not rejecting it on principle, it literally
  can't parse it.

And this only affects **sending**. A SegWit-aware node validates and spends SegWit
UTXOs perfectly regardless of which wallets in the world can address them — building
an output that pays a format, and recognizing/spending a UTXO already locked to that
format, are different code paths. A wallet can lag on one and be fully capable on the
other.

---

## Part 5 — Weight, virtual size, and why SegWit is cheaper

This is the arithmetic Lab 06 tests, and it's worth working through with real
numbers rather than just trusting the formula.

```
compare_fees(226 vbytes legacy, 141 vbytes segwit, 50 sat/vbyte)
  legacy fee:   226 * 50 = 11,300 sats
  segwit fee:   141 * 50 =  7,050 sats
  savings:                  4,250 sats
```

BIP141 defines weight as `stripped_size * 3 + total_size`, where `stripped_size` is
everything except witness data. Work through why: `stripped_size` is counted three
*extra* times, on top of the one time it's already inside `total_size` — so it ends
up weighted ×4. Witness bytes only appear once, inside `total_size`, so they land at
weight ×1 — a quarter the cost per byte. Divide the whole thing by 4 to get "virtual
bytes" and you're back to a familiar per-byte fee unit, but the underlying 4:1 split
between base data and witness data is the actual mechanism. It's not a flat discount
on the whole transaction — it specifically only discounts the field that pre-SegWit
nodes never validate.

---

## Part 6 — BIP39: from words to a 512-bit seed

The public test mnemonic used throughout this repo:

```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

```
word_count:     12
entropy_bits:   128
checksum_bits:  4
```

BIP39's checksum is the first `ENT/32` bits of `SHA256(entropy)`, tacked onto the end
of the entropy before it gets split into 11-bit chunks (each chunk picks one of 2048
words). 128 bits of entropy → 4 checksum bits → 132 bits total → 132 / 11 = 12 words.
That checksum exists purely to catch a mistyped or reordered word when you're reading
a mnemonic back off paper — it's not secret, anyone can recompute it from the words.

The seed is a different thing entirely:

```
seed (mnemonic + "TREZOR" passphrase):
c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04
```

That 64-byte value comes from `PBKDF2-HMAC-SHA512(mnemonic, "mnemonic" + passphrase,
2048 rounds)`. Change the passphrase and you get a totally different 64 bytes, and
from there a totally different tree of keys — the passphrase never touches the
checksum or the word list, so there's nothing to check a forgotten passphrase against
except re-deriving and looking for coins.

---

## Part 7 — BIP32: turning one seed into a tree of keys

```
master xpriv (regtest):
tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd

derived at m/84'/1'/0':
xpriv:  tprv8fSjiqEQ8YG7Ro7gw2ScwcvweYuuWi1ZzGUtrPz918HvDtBzL5s2voFTrN4y3yUwj5cYD54pLhxk6NKCzHUjcka3zbKjbTEcsuAnkzbjhkL
xpub:   tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M
```

Every extended key is really two things glued together: the key itself, and a
**chain code** — 256 bits mixed into HMAC-SHA512 at every derivation step alongside
the parent key. Without the chain code, deriving a "child" would just be indexing
over the parent's public key, and EC point math is public and reversible — anyone
could reproduce the same children from the parent pubkey and a guessed index. The
chain code is what makes each child depend on something not recoverable from the
public key alone.

`xpub` = public key + chain code, nothing private. That's exactly the right amount
of information for a watch-only wallet: it can derive every non-hardened child pubkey
and every address the account will ever use, so it can watch for payments — but it
can never sign anything.

Hardened children (the ones with the `'` in the path) can't be derived from an xpub
at all, on purpose. BIP32 defines hardened derivation to hash the parent's *private*
key into the derivation, not the public key — specifically so that leaking one
hardened child's private key plus the parent xpub reveals nothing about its siblings.
That containment is why BIP44 hardens `purpose'`, `coin_type'`, and `account'`.

---

## Part 8 — BIP44: reading a path like an address

```
m/44'/0'/2'/1/5
```

```
decoded:   purpose=44  coin=0  account=2  change=1  index=5
described: purpose 44' selects BIP44, coin' 0 selects the coin, this is the third
           account, using the change (internal) chain, and the sixth address in
           that chain
```

Everything below `account'` is zero-based, which trips people up: `account' = 2'` is
the **third** account (`0'`, `1'`, `2'`), and `index = 5` is the **sixth** address on
that branch. `change` is the receive/internal switch — `0` for addresses you hand out
to get paid, `1` for change the wallet sends back to itself. Splitting those onto
separate branches is what lets a watch-only wallet or explorer tell "money coming in"
from "change coming back" from the path alone.

---

## Part 9 — One seed, three wallets: BIP44 vs BIP49 vs BIP84

Same mnemonic, same account, same index 0 — three different address families:

```
BIP44 (m/44'/1'/0'/0/0)  P2PKH:          mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
BIP49 (m/49'/1'/0'/0/0)  P2SH-P2WPKH:    2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2
BIP84 (m/84'/1'/0'/0/0)  P2WPKH:         bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk
```

Same private key at each path — the derivation from mnemonic to key doesn't care
about script family at all. What changes is purely how that key gets *encoded* into
an address: as a P2PKH hash, wrapped in a P2SH script, or as a native witness
program. That's the exact thing Lab 10 tests: derive once, encode three ways, get
three unrelated-looking addresses from one key.

This is also the practical trap in wallet recovery. The derivation itself is
deterministic and always reproduces the same key — that's not where recovery breaks.
It breaks when the *software* guesses the wrong script-family convention: restore a
mnemonic assuming BIP84 when the coins were actually received at BIP44 or BIP49
addresses from that same seed, and the wallet scans the wrong branch entirely and
reports a zero balance. Nothing about the key derivation failed — the convention
layered on top of it just didn't match what was actually used.

---

## What to check against your own run

Every command below reproduces a value in this file:

```bash
cargo test --test lab_01   # regtest prefixes, network-checked parsing
cargo test --test lab_02   # p2pkh address + scriptPubKey above
cargo test --test lab_03   # 2-of-3 redeemScript + p2sh address
cargo test --test lab_04   # p2wpkh address + scriptPubKey above
cargo test --test lab_06   # 11,300 / 7,050 / 4,250 sat fee numbers
cargo test --test lab_07   # seed hex above, against the published BIP39 test vector
cargo test --test lab_08   # master xpriv / xpub above
cargo test --test lab_09   # decoded path + description above
cargo test --test lab_10   # the three addresses in Part 9
```
