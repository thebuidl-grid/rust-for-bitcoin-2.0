\# Lab 05 — Transaction creation and mempool inspection



\## Commands used



Commands executed:



```bash

cargo test --test lab\_05



Bitcoin Core RPC concepts used by the lab:



Sending wallet transactions

Checking transaction status

Inspecting mempool contents

Tracking unconfirmed transactions

Terminal output



The Lab 05 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.06s

Running tests\\lab\_05.rs



running 4 tests



test sends\_payment\_in\_sender\_wallet\_context ... ok

test reads\_wallet\_transaction\_status ... ok

test reads\_local\_mempool\_txids ... ok

test observes\_broadcast\_without\_confirmation ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application successfully handled Bitcoin transaction creation and mempool tracking.



The implementation was able to:



Send payments using a wallet context.

Retrieve wallet transaction status information.

Inspect transactions waiting in the local mempool.

Confirm that a broadcast transaction can exist before receiving block confirmation.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_05 passing.

Screenshot reference: lab05-test-success.png.

Explanation



The Bitcoin mempool is a temporary storage area where valid transactions wait before being included in a block by miners.



When a wallet creates and broadcasts a transaction, Bitcoin Core validates the transaction and places it into the mempool until it receives confirmation through mining.



The Rust application communicates with Bitcoin Core using RPC calls to create transactions, check transaction details, and inspect mempool contents.



The successful tests confirm that the implementation correctly handles transaction broadcasting, wallet transaction status checks, and unconfirmed transaction tracking in the regtest environment.

