use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("serialize-trx").unwrap()
}

#[test]
fn serializes_segwit_transaction_matching_known_vector() {
    cmd()
        .args([
            "--tx-version",
            "2",
            "--segwit",
            "--locktime",
            "0",
            "--input",
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,sequence=4294967295,witness=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358",
            "--output",
            "value=69886,script_pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b",
            "--output",
            "value=29442,script_pubkey=00149831122b93d21715c70db626ccc844d3c21f9687",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000",
        ))
        .stdout(predicate::str::contains("Transaction size: 223 bytes"));
}

#[test]
fn serializes_legacy_transaction_without_marker_and_flag() {
    cmd()
        .args([
            "--tx-version",
            "1",
            "--locktime",
            "0",
            "--input",
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0",
            "--output",
            "value=1000,script_pubkey=76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Serialized transaction (hex):"))
        // Legacy transactions must not contain the SegWit marker/flag (0001) right after nVersion.
        .stdout(predicate::str::contains("0100000021").not());
}

#[test]
fn rejects_invalid_hex_with_meaningful_error() {
    cmd()
        .args([
            "--input",
            "txid=zzzz,vout=0",
            "--output",
            "value=1,script_pubkey=aa",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid hex character"));
}

#[test]
fn rejects_odd_length_hex_with_meaningful_error() {
    cmd()
        .args([
            "--input",
            "txid=abc,vout=0",
            "--output",
            "value=1,script_pubkey=aa",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("odd number of characters"));
}

#[test]
fn rejects_wrong_txid_length() {
    cmd()
        .args([
            "--input",
            "txid=aabbcc,vout=0",
            "--output",
            "value=1,script_pubkey=aa",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("32 bytes"));
}

#[test]
fn requires_at_least_one_input() {
    cmd()
        .args(["--output", "value=1,script_pubkey=aa"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--input"));
}

#[test]
fn requires_at_least_one_output() {
    cmd()
        .args([
            "--input",
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--output"));
}

#[test]
fn warns_when_witness_given_without_segwit_flag() {
    cmd()
        .args([
            "--input",
            "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0,witness=aa",
            "--output",
            "value=1,script_pubkey=aa",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("--segwit"));
}
