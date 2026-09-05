# Lab 06 — Weight, virtual size, and fees

## Commands used

cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_06

## Terminal output

running 4 tests
test reproduces_the_class_fee_comparison ... ok
test calculates_bip141_weight ... ok
test rounds_weight_up_to_virtual_bytes ... ok
test calculates_fee_from_feerate ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

## Evidence references

The public Lab 06 test output above is the primary execution evidence for this
lab.

The tests verify:

- BIP141 transaction weight calculation
- conversion from weight to virtual bytes
- fee calculation from virtual size and feerate
- the legacy versus SegWit fee comparison

## Explanation

### What is SegWit?

SegWit stands for **Segregated Witness**. It was a change to Bitcoin's
transaction structure that separated the transaction's witness data from its
traditional transaction data.

To understand this, it helps to compare a legacy transaction with a SegWit
transaction.

In a legacy P2PKH transaction, the spending proof is placed in the ScriptSig:

    ScriptSig:
    signature + public key

The signature and public key are therefore part of the traditional serialized
transaction data.

With native SegWit, such as P2WPKH, the ScriptSig is empty and the spending proof
is placed in the transaction's witness:

    ScriptSig:
    empty

    Witness:
    signature + public key

This is why Lab 04 had an empty ScriptSig for native P2WPKH. The signature and
public key were not removed; they were moved into the witness section.

The word "segregated" refers to this separation of the witness data from the
traditional transaction data.

### Why does the witness matter?

SegWit changed how Bitcoin accounts for transaction size.

Before SegWit, the signature data was part of the normal transaction
serialization. SegWit separates that data into the witness and gives witness
data a lower weight when calculating transaction size.

This is important for two reasons.

First, separating the witness from the traditional transaction data helped
address transaction malleability. The transaction's traditional identifier is
no longer affected in the same way by changes to the witness data.

Second, witness data is counted differently when determining the transaction's
weight. This makes transactions that use SegWit more space-efficient from the
fee calculation perspective.

This does not mean that witness data is free. Witness bytes still contribute to
the transaction's weight; they simply have a lower weight than non-witness
bytes.

### How does Bitcoin calculate transaction weight?

BIP141 uses four weight units for every byte of non-witness transaction data and
one weight unit for every byte of witness data.

The lab receives two sizes:

- `stripped_size`: the transaction size without witness data
- `total_size`: the complete transaction size including witness data

Because the complete transaction contains the stripped transaction plus the
witness, `total_size` must be greater than or equal to `stripped_size`.

The formula used by the lab is:

    weight = stripped_size * 3 + total_size

This may initially look unusual. It works because `total_size` already contains
the stripped transaction.

If:

    total_size = stripped_size + witness_size

then:

    weight = stripped_size * 3 + total_size

becomes:

    weight = stripped_size * 3 + stripped_size + witness_size

and therefore:

    weight = stripped_size * 4 + witness_size

This makes the weighting rule easier to see:

- each non-witness byte contributes 4 weight units
- each witness byte contributes 1 weight unit

For example, if a transaction has 100 stripped bytes and 200 total bytes:

    weight = (100 * 3) + 200
           = 500

This is exactly what the first test verifies.

### What are virtual bytes (vbytes)?

Transaction weight is not the unit normally used when choosing a Bitcoin fee
rate. Fees are usually expressed in satoshis per virtual byte, or sat/vB.

Bitcoin converts weight into virtual size using:

    vsize = ceil(weight / 4)

The result is rounded up because a transaction cannot have a fractional virtual
byte.

For example:

    weight = 564
    vsize = ceil(564 / 4)
          = 141 vbytes

But:

    weight = 565
    vsize = ceil(565 / 4)
          = 141.25
          = 142 vbytes

This is why the lab checks both 564 and 565.

### How are fees calculated?

Once the virtual size is known, the transaction fee can be calculated from the
feerate:

    fee = vsize * feerate

For example, at a feerate of 50 sat/vB:

    141 vbytes * 50 sat/vB
    = 7,050 sats

The implementation uses checked arithmetic so that an integer overflow cannot
silently produce an incorrect fee.

### Why can SegWit transactions have lower fees?

The important point is that SegWit is **not a flat percentage discount**.

Bitcoin does not simply say "SegWit transactions cost 25% less." Instead, the
different parts of the transaction have different weights.

Non-witness data contributes 4 weight units per byte, while witness data
contributes only 1 weight unit per byte.

Because signatures and other spending-proof data can be placed in the witness,
transactions that use SegWit can have a lower virtual size than an equivalent
legacy transaction.

The actual fee still depends on the transaction's virtual size and the
feerate chosen by the sender.

### The class fee comparison

The lab uses these example values:

    Legacy transaction:
    226 vbytes

    Native SegWit transaction:
    141 vbytes

    Feerate:
    50 sat/vB

The legacy fee is:

    226 * 50 = 11,300 sats

The SegWit fee is:

    141 * 50 = 7,050 sats

The savings are:

    11,300 - 7,050 = 4,250 sats

The important lesson is not simply that "SegWit is cheaper." The reason is that
SegWit changes how transaction data is represented and weighted. Witness data
still counts, but it counts less toward virtual size than non-witness data.

### Connecting this lab to the previous labs

The previous labs now fit together:

**Lab 02 — P2PKH**

A legacy P2PKH spend puts the signature and public key in ScriptSig:

    ScriptSig:
    signature + public key

**Lab 04 — P2WPKH**

A native SegWit P2WPKH spend leaves ScriptSig empty and puts the spending proof
in the witness:

    ScriptSig:
    empty

    Witness:
    signature + public key

**Lab 06 — Weight and fees**

Because the witness is separated from the traditional transaction data, Bitcoin
can account for those bytes differently:

    non-witness byte = 4 weight units
    witness byte     = 1 weight unit

That weight is converted into virtual bytes:

    vsize = ceil(weight / 4)

And the virtual size determines the fee:

    fee = vsize * feerate

So the progression is:

    P2PKH
       ↓
    spending data in ScriptSig
       ↓
    SegWit
       ↓
    spending data separated into witness
       ↓
    different weight for witness bytes
       ↓
    virtual size
       ↓
    transaction fee

The key idea from this lab is that SegWit does not simply make transactions
"smaller" or give them a flat discount. It changes the transaction structure and
introduces weight accounting, where witness and non-witness data contribute
differently to the transaction's virtual size and therefore to its fee.