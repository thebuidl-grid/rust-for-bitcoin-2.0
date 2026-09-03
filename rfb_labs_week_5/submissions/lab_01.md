# Lab 01 — Address and network identification

## Commands used

```cargo test --test lab_01
cargo fmt
cargo fmt --check


## Terminal output

```running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

```A human-readable prefix (1, 3, bc1q, bc1p, or their testnet/regtest equivalents m/n, 2, tb1q/bcrt1q, tb1p/bcrt1p) is only a naming convention layered on top of the address encoding — it tells you which script family and network an address claims to be, nothing more. identify_prefix in this lab does exactly that: a plain string check, with no cryptographic guarantee behind it. That's why the string "bc1pexample" — not a real, validly encoded address — still gets classified as P2tr by that function.

Real validation happens in inspect_address, which parses the address into Address<NetworkUnchecked>. Base58Check addresses embed a 4-byte checksum, and Bech32/Bech32m addresses embed their own checksum, so parsing alone already rejects typos and corrupted addresses that would otherwise "look" right by prefix alone. Parsing does not, however, confirm the address belongs to the network the caller expects — a syntactically valid mainnet address is not automatically safe to pay from a regtest or testnet wallet context. That's why require_network is a separate, mandatory step: it's how rust-bitcoin's type system forces the caller to assert the expected network before the address can be used for anything real, such as reading its address_type() or its script_pubkey(). rejects_an_address_for_the_wrong_network demonstrates this: a well-formed, checksum-valid regtest address is still rejected by inspect_address and script_pubkey_hex when checked against Network::Bitcoin, because a correct prefix and a correct checksum say nothing about which chain the address was meant for.

