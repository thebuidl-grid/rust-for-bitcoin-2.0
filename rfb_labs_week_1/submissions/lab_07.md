\# Lab 07 — Transaction confirmation and block inclusion



\## Commands used



Commands executed:



```bash

cargo test --test lab\_07



Bitcoin Core RPC calls used by the lab:



generatetoaddress

getrawmempool

gettransaction

getblock

Terminal output



The Lab 07 test suite was executed.



Finished test profile \[unoptimized + debuginfo] target(s) in 0.06s

Running tests\\lab\_07.rs



running 4 tests



test detects\_empty\_mempool ... ok

test mines\_exactly\_one\_block ... ok

test reads\_confirmation\_count ... ok

test proves\_transaction\_is\_inside\_confirming\_block ... FAILED



Failure:



assertion left == right failed: wrong RPC parameters



left:

\["block-hash", "1"]



right:

\["block-hash"]



test result: FAILED. 3 passed; 1 failed



Testing notes



During validation, a mismatch was identified between the Lab 07 test expectation and the Bitcoin Core RPC call used by the implementation.



The test expected the getblock RPC call to receive:



\["block-hash"]



However, the implementation called:



\["block-hash", "1"]



The second parameter is the verbosity level supported by Bitcoin Core's getblock RPC method. The implementation uses this parameter to request block transaction information required to verify transaction inclusion.



The failure appears to be caused by a mismatch between the provided mock test expectation and the implementation behaviour rather than an RPC communication failure.



Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_07 execution.

Screenshot reference: lab07-test-result.png.

Explanation



This lab verifies transaction confirmation by mining a block, checking that the mempool is empty, retrieving transaction confirmation information, and confirming that the transaction exists inside the confirming block.



Bitcoin Core tracks transaction confirmations by connecting transactions to blocks in the active chain. Once a transaction is mined into a block, its confirmation count increases.



The Rust implementation communicates with Bitcoin Core through RPC calls to inspect transaction status and block contents in the regtest environment.

