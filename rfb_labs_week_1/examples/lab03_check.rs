use rfb_labs_week_1::labs::lab03_maturity::demonstrate_coinbase_maturity;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let mining_address = "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc";
    let classmate_address = "bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at";

    let report =
        demonstrate_coinbase_maturity(&rpc, "miner", mining_address, classmate_address).unwrap();

    println!("{report:#?}");
}
