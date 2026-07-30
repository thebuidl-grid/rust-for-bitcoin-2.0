# Lab 10 — Competing branches and reorganization

## Commands used

`cargo test --test lab_10
cargo run --example lab10_check
`

## Terminal output

`common tip before split: ChainTip {
    height: 115,
    best_block_hash: "7b9830df0fb23d4fb0e439892f0cf046bed0c473ea2670bc2d7f201d902074e0",
    chainwork: "00000000000000000000000000000000000000000000000000000000000000e8",
}
disconnected node A from node B
competing tips: ForkSnapshot {
    node_a: ChainTip {
        height: 117,
        best_block_hash: "0dd45514ce7bd9b7e18ef091fcffffff5013c66bcb42ac5e255d530c8be7928c",
        chainwork: "00000000000000000000000000000000000000000000000000000000000000ec",
    },
    node_b: ChainTip {
        height: 119,
        best_block_hash: "2e7eecf7efbcd6121daf1d55ca3f0f884bee41d7d416bd7200a526ee0ba1a183",
        chainwork: "00000000000000000000000000000000000000000000000000000000000000f0",
    },
}
reconnected node B to node A, waiting for sync...
ReorgReport {
    common_tip_before_split: "7b9830df0fb23d4fb0e439892f0cf046bed0c473ea2670bc2d7f201d902074e0",
    competing_tips: ForkSnapshot {
        node_a: ChainTip {
            height: 117,
            best_block_hash: "0dd45514ce7bd9b7e18ef091fcffffff5013c66bcb42ac5e255d530c8be7928c",
            chainwork: "00000000000000000000000000000000000000000000000000000000000000ec",
        },
        node_b: ChainTip {
            height: 119,
            best_block_hash: "2e7eecf7efbcd6121daf1d55ca3f0f884bee41d7d416bd7200a526ee0ba1a183",
            chainwork: "00000000000000000000000000000000000000000000000000000000000000f0",
        },
    },
    final_tips: ForkSnapshot {
        node_a: ChainTip {
            height: 119,
            best_block_hash: "2e7eecf7efbcd6121daf1d55ca3f0f884bee41d7d416bd7200a526ee0ba1a183",
            chainwork: "00000000000000000000000000000000000000000000000000000000000000f0",
        },
        node_b: ChainTip {
            height: 119,
            best_block_hash: "2e7eecf7efbcd6121daf1d55ca3f0f884bee41d7d416bd7200a526ee0ba1a183",
            chainwork: "00000000000000000000000000000000000000000000000000000000000000f0",
        },
    },
    converged: true,
}
`
## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

Node A's two-block branch didn't lose because anything was wrong with it — every block it mined followed the same consensus rules as Node B's blocks. It lost purely because it represented less accumulated proof-of-work than the alternative. My own evidence shows this precisely: at the moment of the split, Node A's competing tip had chainwork: "...ec" after 2 blocks, while Node B's had chainwork: "...f0" after 4 blocks — a strictly larger number. Once the two networks reconnected, that difference was the entire deciding factor.

A reorganization is exactly what happened to Node A when it reconnected: it didn't just stop mining its own branch — it actively discarded it and rewound to a shared ancestor, then adopted Node B's blocks instead. My final_tips prove this happened, not just that both nodes stopped disagreeing: Node A's final best_block_hash (2e7eecf7...) exactly matches Node B's original competing tip, not Node A's own prior tip (0dd45514...). If both nodes had simply frozen where they were, Node A's final hash would still have been 0dd45514.... Instead, Node A's own view of "the chain" changed — the two blocks it had already accepted as valid were quietly abandoned in favor of Node B's four.

Why does accumulated work decide this, and not miner identity, arrival time, or a social claim of authority? Because chainwork is the one property in this whole dispute that's objectively, independently verifiable — any node can recompute it directly from the sequence of block headers, without needing to trust anyone's word about who mined what or when. If instead the rule were "believe whichever miner claims priority" or "trust whichever chain arrived first at some particular observer," that would reintroduce exactly the problem Bitcoin's proof-of-work consensus exists to eliminate: needing a trusted authority to settle disagreements about history. By contrast, "follow the chain with the most cumulative proof-of-work" is a rule every node can enforce completely on its own, using only public, checkable math — which is exactly why my two independently-running nodes converged on the identical tip with no coordination or arbitration between them, purely by each independently applying the same rule to the same data.
