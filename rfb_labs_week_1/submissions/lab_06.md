\# Lab 06 — Transaction decoding and fee analysis



\## Commands used



Commands executed:



```bash

cargo test --test lab\_06



Bitcoin Core RPC concepts used by the lab:



Transaction decoding

Transaction inputs and outputs

Virtual size calculation

Fee calculation

Output identification

Terminal output



The Lab 06 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.09s

Running tests\\lab\_06.rs



running 4 tests



test calculates\_fee\_from\_input\_and\_output\_values ... ok

test distinguishes\_receiver\_output\_from\_change ... ok

test decodes\_inputs\_outputs\_and\_virtual\_size ... ok

test returns\_consumed\_outpoints ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application correctly decoded Bitcoin transaction information and performed fee analysis.



The implementation was able to:



Decode transaction inputs and outputs.

Identify consumed transaction outpoints.

Calculate transaction fees from input and output values.

Distinguish receiver outputs from wallet change outputs.

Process transaction virtual size information.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_06 passing.

Screenshot reference: lab06-test-success.png.

Explanation



Bitcoin transactions are composed of inputs that spend previous UTXOs and outputs that create new UTXOs. Understanding this structure is essential for wallet development and transaction analysis.



Transaction fees are calculated from the difference between the total value of inputs and the total value of outputs. The fee rate depends on the transaction size, which is measured using virtual size (vsize).



The Rust application processes decoded transaction data to identify inputs, outputs, consumed outpoints, and payment destinations. This allows wallet software to understand transaction behaviour and calculate fees accurately.



The successful tests confirm that the implementation correctly handles transaction decoding, fee calculation, and output analysis in the Bitcoin regtest environment.

