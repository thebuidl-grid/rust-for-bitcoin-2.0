\# Lab 03 — Coinbase maturity and wallet balances



\## Commands used



Commands executed:



```bash

cargo test --test lab\_03



Bitcoin Core RPC concepts used by the lab:



Mining blocks

Checking wallet balances

Handling coinbase maturity rules

Verifying insufficient funds errors

Terminal output



The Lab 03 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.10s

Running tests\\lab\_03.rs



running 4 tests



test preserves\_insufficient\_funds\_error ... ok

test mines\_requested\_number\_of\_blocks ... ok

test reads\_nested\_wallet\_balances ... ok

test demonstrates\_first\_coinbase\_becoming\_spendable\_at\_height\_101 ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application successfully handled Bitcoin regtest mining and wallet maturity rules.



The implementation was able to:



Mine the required number of blocks.

Read wallet balances correctly.

Handle nested wallet balance information.

Detect when coinbase funds are not yet spendable.

Verify that coinbase outputs become spendable after the required maturity period.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_03 passing.

Screenshot reference: lab03-test-success.png.

Explanation



Bitcoin coinbase transactions are the special transactions created when a miner successfully mines a block. Unlike normal transactions, coinbase outputs cannot be spent immediately. Bitcoin requires a maturity period before newly mined coins become spendable.



In regtest, this rule is still enforced, allowing developers to test real Bitcoin behaviour in a private environment.



The Rust application uses Bitcoin Core RPC communication to mine blocks, inspect wallet balances, and verify that immature coinbase funds are correctly handled.



The successful tests confirm that the implementation correctly understands block rewards, wallet balances, and Bitcoin's coinbase maturity rules.

