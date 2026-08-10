\# Lab 10 — Chain reorganization and peer synchronization



\## Commands used



Commands executed:



```bash

cargo test --test lab\_10



Bitcoin Core concepts used by the lab:



Peer connections

Chain tips

Accumulated chainwork

Branch selection

Blockchain reorganization

Terminal output



The Lab 10 test suite completed successfully.



Finished `test` profile \[unoptimized + debuginfo] target(s) in 0.07s

Running tests\\lab\_10.rs



running 4 tests



test disconnects\_peer\_by\_address ... ok

test reconnects\_peer\_for\_synchronization ... ok

test reads\_tip\_and\_accumulated\_chainwork ... ok

test reports\_convergence\_on\_the\_stronger\_branch ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application correctly handled blockchain synchronization and chain selection behaviour.



The implementation was able to:



Disconnect and reconnect peers.

Inspect chain tip information.

Compare accumulated chainwork between branches.

Detect convergence on the stronger blockchain branch.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_10 passing.

Screenshot reference: lab10-test-success.png.

Explanation



Bitcoin nodes maintain a shared view of the blockchain by communicating with peers and following the chain with the greatest accumulated proof of work.



During a chain reorganization, a node may temporarily see competing branches. The node compares the accumulated chainwork of available branches and follows the stronger valid chain.



Peer management and synchronization are important parts of Bitcoin networking because nodes must exchange blockchain information and converge on the same canonical chain.



The Rust application uses Bitcoin Core information to inspect peers, chain tips, and synchronization state.



The successful tests confirm that the implementation correctly handles peer reconnection, chainwork comparison, and blockchain convergence in the regtest environment.

