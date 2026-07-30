use rfb_labs_week_1::labs::lab07_confirm::confirm_and_locate_transaction;
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

    let txid = "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9";
    let mining_address = "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc";

    let report = confirm_and_locate_transaction(&rpc, "receiver", txid, mining_address).unwrap();
    println!("{report:#?}");
}
