use rfb_labs_week_1::labs::lab08_security::build_security_report;
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
    let block_hash = "2e69f34a27acdd327da77e12aaeb42e7812b0c5977392976b4e3b315a03698d9";
    let mining_address = "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc";

    let report = build_security_report(&rpc, "receiver", txid, block_hash, mining_address).unwrap();

    println!("{report:#?}");
}
