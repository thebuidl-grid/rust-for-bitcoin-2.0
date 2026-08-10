\# Lab 04 — UTXO inspection and coin selection



\## Commands used



Commands executed:



```bash

cargo test --test lab\_04



Bitcoin Core RPC concepts used by the lab:



listunspent

UTXO inspection

Transaction outpoints

Spendable output selection

Terminal output



The Lab 04 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.06s

Running tests\\lab\_04.rs



running 4 tests



test constructs\_unique\_outpoint ... ok

test decodes\_listunspent\_response ... ok

test selects\_most\_confirmed\_spendable\_utxo ... ok

test sums\_only\_spendable\_outputs ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application correctly processed Bitcoin UTXO information.



The implementation was able to:



Decode unspent transaction output data.

Construct unique transaction outpoints.

Select suitable spendable UTXOs.

Calculate available spendable value while ignoring unavailable outputs.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_04 passing.

Screenshot reference: lab04-test-success.png.

Explanation



Bitcoin transactions consume previous unspent transaction outputs (UTXOs) as inputs and create new outputs. A wallet must identify available UTXOs before creating a transaction.



A UTXO is uniquely identified by an outpoint, which consists of the previous transaction ID and the output index. Correctly handling outpoints prevents selecting the wrong transaction outputs.



The Rust implementation communicates with Bitcoin Core data and processes UTXO information to identify spendable funds. Coin selection is an important wallet function because it determines which available outputs should be used when creating transactions.



The successful tests confirm that the application correctly decodes UTXO data, identifies spendable outputs, and performs basic coin selection logic.

