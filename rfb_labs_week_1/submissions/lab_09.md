\# Lab 09 — Coin selection, payments, and transaction auditing



\## Commands used



Commands executed:



```bash

cargo test --test lab\_09



Bitcoin Core concepts used by the lab:



Wallet funding

Transaction creation

UTXO selection

Payment outputs

Change outputs

Transaction fee auditing

Terminal output



The Lab 09 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.06s

Running tests\\lab\_09.rs



running 4 tests



test creates\_three\_separate\_funding\_transactions ... ok

test sends\_one\_btc\_from\_alice ... ok

test filters\_confirmed\_utxos\_for\_alice\_address ... ok

test audits\_three\_input\_spend\_payment\_change\_and\_fee ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application correctly handled transaction creation, UTXO selection, and transaction auditing.



The implementation was able to:



Create multiple funding transactions.

Select confirmed UTXOs for spending.

Send a payment from a wallet.

Identify payment outputs, change outputs, and transaction fees.

Audit transactions containing multiple inputs.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_09 passing.

Screenshot reference: lab09-test-success.png.

Explanation



Bitcoin wallets must select available UTXOs when creating transactions. This process is known as coin selection and determines which previous outputs will be consumed as transaction inputs.



A transaction may contain multiple inputs and outputs. The outputs can include the payment destination and a change output returning remaining funds back to the sender. The difference between total input value and total output value represents the transaction fee.



The Rust application processes wallet and transaction data to select appropriate UTXOs, create payments, and verify transaction details.



The successful tests confirm that the implementation correctly performs coin selection, payment creation, and transaction auditing in the Bitcoin regtest environment.

