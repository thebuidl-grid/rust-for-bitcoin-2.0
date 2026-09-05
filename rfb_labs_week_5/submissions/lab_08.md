# Lab 08 — BIP32 extended keys

## Commands used

cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_08

All tests used the public BIP39 test mnemonic and Regtest network. No real
wallet recovery phrases or private wallet keys were used.

## Terminal output

running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.03s

## Evidence references

The public Lab 08 test output above is the primary execution evidence.

The tests verify that:

- the BIP39 seed can be converted into a Regtest master extended private key
- a derivation path produces matching extended private and public keys
- an xpub can derive a normal public child without private key material
- hardened and normal derivation steps can be distinguished

Private extended keys are intentionally not included in the submission.

## Explanation

### How Lab 08 builds on Lab 07

In Lab 07, the mnemonic was converted into a 512-bit BIP39 seed.

The flow was:

    mnemonic + passphrase
             ↓
        BIP39 seed
             ↓
         512 bits

Lab 08 takes that seed and starts the next stage of wallet key derivation using
**BIP32**.

The overall process is:

    BIP39 mnemonic
          ↓
      BIP39 seed
          ↓
    BIP32 master xpriv
          ↓
    child extended keys
          ↓
       xpub / xpriv
          ↓
    individual wallet keys

This is how a single recovery setup can ultimately produce a large hierarchy of
Bitcoin keys and addresses.

### What is an extended private key?

An ordinary private key represents one private key.

A BIP32 **extended private key**, usually called an `xpriv`, represents a node
in a hierarchical key tree.

Conceptually, an extended private key can be used to derive child keys:

    master xpriv
       ├── child 0
       ├── child 1
       ├── child 2
       └── ...

Each child can itself become a parent:

    master
       └── child
             ├── child 0
             ├── child 1
             └── child 2

This creates a hierarchical deterministic (HD) wallet structure.

The advantage is that a wallet does not need to generate and independently back
up every private key. The hierarchy can be deterministically reproduced from
the original recovery setup.

### What is an xpub?

An **extended public key**, or `xpub`, is the public counterpart of an extended
private key.

The private extended key contains private key material, while the xpub contains
the public information needed to derive normal public children.

The important property demonstrated by this lab is:

    xpriv → xpub

but also:

    xpub → normal child xpub

This means an application can sometimes be given an xpub and generate public
child keys without ever having access to the private keys.

For example, a watch-only wallet could use an xpub to derive public addresses
and monitor balances without being able to spend the funds.

An xpub therefore should not be treated as a private secret in the same way as
an xpriv, but it still reveals wallet structure and addresses and should be
handled appropriately.

### What is the chain code?

An extended key contains more than just a private or public key.

One important additional piece of information is the **chain code**.

The chain code is 32 bytes of data used during BIP32 child-key derivation. It
allows BIP32 to deterministically derive children while maintaining the
hierarchical structure.

Conceptually, an extended key can be thought of as containing:

    key material + chain code + metadata

The key material represents the cryptographic key at that node, while the chain
code provides additional derivation information used to generate its children.

The chain code is therefore one of the reasons an extended key is different from
an ordinary Bitcoin private key or public key.

### What does "master" mean?

The master extended private key is the root of the BIP32 hierarchy.

Lab 08 starts with the BIP39 seed and creates the master xpriv:

    BIP39 seed
        ↓
    master xpriv
        ↓
    child keys

The master xpriv is the root from which the rest of the BIP32 tree can be
derived.

For Regtest, the serialized master extended private key begins with `tprv`, and
the corresponding extended public key begins with `tpub`.

These prefixes indicate that the extended keys are being serialized for a
Bitcoin test network rather than mainnet.

### What is a derivation path?

A derivation path describes which children should be derived from the master.

For example:

    m/84'/1'/0'

The `m` represents the master node.

Each number represents another step down the hierarchy:

    m
     ↓
    84'
     ↓
    1'
     ↓
    0'

The path used in this lab contains hardened steps because the numbers have an
apostrophe (`'`) after them.

### Hardened derivation

A hardened child is indicated by an apostrophe:

    84'
    1'
    0'

Hardened derivation is important because it changes what information is required
to derive the child.

An xpub cannot derive a hardened child.

Therefore:

    xpub → normal child
    xpub → hardened child

The first is possible, while the second is not.

This is one of the important security boundaries in BIP32.

Hardened derivation is commonly used near the top of wallet derivation paths.
For example, later labs will use paths such as BIP44, BIP49, and BIP84 paths,
which contain hardened components.

### Normal derivation

A normal, or non-hardened, child does not have the apostrophe:

    m/84'/1'/0'/0

The final `0` is a normal derivation step.

Normal public derivation allows an xpub to derive the corresponding public child.

That is what the `xpub_derives_a_normal_public_child` test demonstrates.

The test first derives an xpub at:

    m/84'/1'/0'/0

and then derives child `7` from that xpub.

The private key is not needed for this operation.

Conceptually:

    parent xpub
        ↓
    normal child 7
        ↓
    child xpub

This is useful for systems that need to generate or monitor addresses but should
not have access to spending keys.

### Why do we need hardened and normal derivation?

The two types provide different capabilities.

Normal derivation makes public-key-only derivation possible:

    xpub
      ↓
    child xpub

Hardened derivation creates a boundary where the parent private key is required:

    parent xpriv
        ↓
    hardened child

This allows an HD wallet to expose an xpub for a particular branch without
necessarily exposing the private keys needed to derive other branches.

### Connecting BIP39 and BIP32

The most important connection between the last two labs is:

    BIP39
       ↓
    mnemonic
       ↓
    512-bit seed
       ↓
    BIP32
       ↓
    master xpriv
       ↓
    derivation path
       ↓
    child xpriv / xpub
       ↓
    Bitcoin addresses

BIP39 gives us the deterministic seed from the recovery words.

BIP32 gives us the hierarchical key tree built from that seed.

The mnemonic therefore does not directly generate an individual Bitcoin address.
It starts a chain of deterministic transformations that eventually produces the
keys and addresses used by the wallet.

### Key ideas from this lab

An **xpriv** is an extended private key that can derive private descendants.

An **xpub** is an extended public key that can derive normal public descendants
without exposing private key material.

The **chain code** is additional 32-byte information stored with an extended key
and used during child derivation.

A **hardened child** uses a hardened derivation step such as `0'` and cannot be
derived from an xpub.

A **normal child** uses a step such as `0` and can be derived from an xpub.

The main lesson is that BIP32 turns the single BIP39 seed from Lab 07 into a
structured, deterministic tree of keys rather than treating the seed as one
Bitcoin private key.