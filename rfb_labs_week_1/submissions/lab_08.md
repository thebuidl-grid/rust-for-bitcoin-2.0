\# Lab 08 — Block headers and confirmation depth



\## Commands used



Commands executed:



```bash

cargo test --test lab\_08



Bitcoin Core concepts used by the lab:



Block headers

Block confirmations

Mining additional blocks

Proof-linked block information

Wallet confirmation depth

Terminal output



The Lab 08 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.06s

Running tests\\lab\_08.rs



running 4 tests



test decodes\_proof\_linked\_block\_header ... ok

test mines\_requested\_confirmation\_depth ... ok

test proves\_one\_confirmation\_becomes\_six ... ok

test reads\_wallet\_confirmation\_depth ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application correctly handled Bitcoin block confirmation tracking and block header information.



The implementation was able to:



Decode block header information.

Mine additional blocks to increase confirmation depth.

Verify confirmation changes from one confirmation to deeper confirmation levels.

Read wallet transaction confirmation information.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_08 passing.

Screenshot reference: lab08-test-success.png.

Explanation



Bitcoin confirmations represent the number of blocks added after the block containing a transaction. Each additional block increases confidence that a transaction will remain part of the blockchain.



A transaction included in a newly mined block has one confirmation. As more blocks are mined on top of it, the confirmation count increases. This is an important concept for wallets and applications that need to determine whether a transaction is sufficiently confirmed.



Block headers contain important proof-related information, including links to previous blocks and data used to verify blockchain continuity.



The Rust application uses Bitcoin Core information to inspect blocks, track confirmations, and verify transaction confirmation depth.



The successful tests confirm that the implementation correctly handles block headers, confirmation tracking, and blockchain depth calculations in the regtest environment.

