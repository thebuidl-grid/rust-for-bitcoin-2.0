use rfb_labs_week_1::labs::lab05_mempool::observe_unconfirmed_payment;
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

    let classmate_address = "bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at";

    let observation =
        observe_unconfirmed_payment(&rpc, "miner", "receiver", classmate_address, 1.0).unwrap();

    println!("{observation:#?}");
}
