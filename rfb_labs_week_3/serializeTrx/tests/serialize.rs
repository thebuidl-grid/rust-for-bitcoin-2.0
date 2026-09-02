//! End to end tests: the binary is run exactly as a user would run it, and the
//! hex it prints is compared against transactions that are already known good.
//!
//! Three of the four vectors are real serialisations: the transaction that
//! used to be hardcoded here, an on-chain P2WPKH spend, and the first Bitcoin
//! payment from block 170. Reproducing them byte for byte shows that moving
//! the data to the command line changed nothing else.

use std::process::{Command, Output};

/// The transaction that used to be written into `main.rs`.
const HARDCODED_TX: &str = "020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000";

/// A v2 P2WPKH spend: one input, two outputs, a two item witness stack.
const SEGWIT_TX: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

/// Block 170: Satoshi to Hal Finney, the first spend on the chain.
const LEGACY_TX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

/// Hand built to reach shapes the on-chain samples do not: three inputs, a
/// scriptSig alongside witnesses, one input with an empty stack, an output
/// worth the entire supply, and a locktime that is a block height.
const MULTI_INPUT_TX: &str = "02000000000103000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0100000000fdffffff202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3ffe00000000fdffffffabababababababababababababababababababababababababababababababab0700000048471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111fdffffff02a086010000000000160014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd0040075af0750700016a0248303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030210202020202020202020202020202020202020202020202020202020202020202020140515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151510020a10700";

fn serializetrx(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_serializetrx"))
        .args(args)
        .output()
        .expect("the binary under test should run")
}

/// Run the program, expect success, and return the serialised hex it printed.
fn expect_hex(args: &[&str]) -> String {
    let output = serializetrx(args);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "expected success, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut lines = stdout.lines();
    lines
        .find(|line| line.starts_with("Serialized transaction (hex):"))
        .expect("the hex heading should be printed");
    lines.next().expect("a hex line should follow").to_string()
}

/// Run the program, expect it to refuse, and return what it said.
fn expect_error(args: &[&str]) -> String {
    let output = serializetrx(args);
    assert!(
        !output.status.success(),
        "expected a failure, but it succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// The size the program reports, read back off its own output.
fn reported_size(args: &[&str]) -> usize {
    let output = serializetrx(args);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("Transaction size: "))
        .and_then(|rest| rest.trim_end_matches(" bytes").parse().ok())
        .expect("a size line should be printed")
}

#[test]
fn reproduces_the_transaction_that_used_to_be_hardcoded() {
    let hex = expect_hex(&[
        "--version",
        "2",
        "--txid-order",
        "internal",
        "--input",
        "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,witness=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358",
        "--output",
        "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b",
        "--output",
        "29442:00149831122b93d21715c70db626ccc844d3c21f9687",
        "--locktime",
        "0",
    ]);

    assert_eq!(hex, HARDCODED_TX);
}

#[test]
fn reproduces_an_on_chain_segwit_transaction() {
    // The txid is given the way an explorer shows it, which is the default,
    // so the program is the one that has to reverse it.
    let args = [
        "--version",
        "2",
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1",
        "--witness",
        "0:304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201|020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
        "--output",
        "4462282:0014599bcef6387256c6b019030c421b4a4d382fe260",
    ];

    assert_eq!(expect_hex(&args), SEGWIT_TX);
    // The reported size is the real length of what it produced.
    assert_eq!(reported_size(&args), SEGWIT_TX.len() / 2);
}

#[test]
fn reproduces_the_block_170_legacy_transaction() {
    let hex = expect_hex(&[
        "--version",
        "1",
        "--input",
        "txid=0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9,vout=0,script_sig=47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901",
        "--output",
        "1000000000:4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac",
        "--output",
        "4000000000:410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac",
    ]);

    // No witness anywhere, so no marker and no flag: the format follows the
    // data without being told.
    assert_eq!(hex, LEGACY_TX);
}

#[test]
fn reproduces_multiple_inputs_with_mixed_witness_stacks() {
    let hex = expect_hex(&[
        "--version",
        "2",
        "--txid-order",
        "internal",
        "--input",
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f:1,sequence=0xfffffffd",
        "--input",
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f:0xfe,sequence=0xfffffffd",
        "--input",
        "abababababababababababababababababababababababababababababababab:7,sequence=0xfffffffd,script_sig=471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
        "--witness",
        "0:303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030|020202020202020202020202020202020202020202020202020202020202020202",
        "--witness",
        "1:51515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151",
        // Input 2 is left without a stack: it still writes its zero count.
        "--output",
        "100000:0014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
        "--output",
        "2100000000000000:6a",
        "--locktime",
        "500000",
    ]);

    assert_eq!(hex, MULTI_INPUT_TX);
}

#[test]
fn an_explicitly_empty_witness_stack_is_the_same_as_leaving_it_out() {
    let base = |third: &str| {
        vec![
            "--txid-order".to_string(),
            "internal".to_string(),
            "--input".to_string(),
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f:1,sequence=0xfffffffd,witness=303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030|020202020202020202020202020202020202020202020202020202020202020202".to_string(),
            "--input".to_string(),
            "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f:0xfe,sequence=0xfffffffd,witness=51515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151".to_string(),
            "--input".to_string(),
            format!("abababababababababababababababababababababababababababababababab:7,sequence=0xfffffffd,script_sig=471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111{third}"),
            "--output".to_string(),
            "100000:0014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string(),
            "--output".to_string(),
            "2100000000000000:6a".to_string(),
            "--locktime".to_string(),
            "500000".to_string(),
        ]
    };

    let implicit: Vec<String> = base("");
    let explicit: Vec<String> = base(",witness=");
    fn as_args(v: &[String]) -> Vec<&str> {
        v.iter().map(String::as_str).collect()
    }

    assert_eq!(expect_hex(&as_args(&implicit)), MULTI_INPUT_TX);
    assert_eq!(expect_hex(&as_args(&explicit)), MULTI_INPUT_TX);
}

#[test]
fn the_shorthand_and_the_key_form_agree() {
    let shorthand = expect_hex(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1",
        "--witness",
        "0:aabb",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    let keyed = expect_hex(&[
        "--input",
        "txid=bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796,vout=1,witness=aabb",
        "--output",
        "amount=100,script_pubkey=0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);

    assert_eq!(shorthand, keyed);
}

#[test]
fn the_reported_size_is_the_length_of_the_hex() {
    let args = [
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ];
    assert_eq!(reported_size(&args), expect_hex(&args).len() / 2);
}

#[test]
fn verbose_reads_the_transaction_back() {
    let output = serializetrx(&[
        "--verbose",
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1,sequence=0xfffffffd",
        "--witness",
        "0:aabb",
        "--output",
        "100000:0014274ae586ad2035efb4c25049c155f98310d7e106",
        "--locktime",
        "500000",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("SegWit (BIP144"));
    assert!(stdout.contains("500000  (block height)"));
    assert!(stdout.contains("replaceable, BIP125"));
    // The outpoint is echoed in the order it was typed, not the internal one.
    assert!(stdout.contains("bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1"));
    assert!(stdout.contains("100,000 sat  (0.00100000 BTC)"));
    assert!(stdout.contains("P2WPKH"));
    assert!(stdout.contains("virtual size"));
}

#[test]
fn bad_hexadecimal_is_caught_before_any_bytes_are_produced() {
    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c279g:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("--input #1 txid"), "{stderr}");
    assert!(
        stderr.contains("`g` is not a hexadecimal digit"),
        "{stderr}"
    );

    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e10",
    ]);
    assert!(stderr.contains("odd length"), "{stderr}");

    let stderr = expect_error(&[
        "--input",
        "bb16396a:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("txid must be 32 bytes, got 4"), "{stderr}");

    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "100:0x0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("must not start with `0x`"), "{stderr}");
}

#[test]
fn bad_numbers_name_the_field_they_came_from() {
    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:4294967296",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("--input #1 vout"), "{stderr}");
    assert!(stderr.contains("out of range"), "{stderr}");

    // One satoshi more than will ever exist.
    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "2100000000000001:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("--output #1 amount"), "{stderr}");
    assert!(stderr.contains("the whole supply"), "{stderr}");

    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "one hundred:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("is not a whole number"), "{stderr}");
}

#[test]
fn malformed_specs_are_explained() {
    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("cannot read"), "{stderr}");
    assert!(stderr.contains("txid:vout"), "{stderr}");

    let stderr = expect_error(&[
        "--input",
        "txid=bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796,vout=0,fee=500",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("unknown key `fee`"), "{stderr}");
    assert!(
        stderr.contains("txid, vout, script_sig, sequence, witness"),
        "{stderr}"
    );

    let stderr = expect_error(&[
        "--input",
        "vout=0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("missing required key `txid`"), "{stderr}");
}

#[test]
fn a_witness_must_belong_to_an_input_that_exists() {
    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--witness",
        "1:aabb",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("there is no input 1"), "{stderr}");
    assert!(stderr.contains("highest usable index is 0"), "{stderr}");

    // Given twice for the same input, once inline and once by index.
    let stderr = expect_error(&[
        "--input",
        "txid=bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796,vout=0,witness=aabb",
        "--witness",
        "0:ccdd",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("already has a witness"), "{stderr}");
}

#[test]
fn the_format_and_the_witness_data_must_agree() {
    let stderr = expect_error(&[
        "--no-segwit",
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--witness",
        "0:aabb",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("--no-segwit was given"), "{stderr}");

    let stderr = expect_error(&[
        "--segwit",
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(
        stderr.contains("no witness items were supplied"),
        "{stderr}"
    );
    assert!(stderr.contains("BIP144"), "{stderr}");

    // The two flags cannot both be given.
    let stderr = expect_error(&[
        "--segwit",
        "--no-segwit",
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
        "--output",
        "100:0014274ae586ad2035efb4c25049c155f98310d7e106",
    ]);
    assert!(stderr.contains("cannot be used with"), "{stderr}");
}

#[test]
fn a_transaction_needs_an_input_and_an_output() {
    let stderr = expect_error(&["--output", "100:6a"]);
    assert!(stderr.contains("--input"), "{stderr}");

    let stderr = expect_error(&[
        "--input",
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:0",
    ]);
    assert!(stderr.contains("--output"), "{stderr}");
}
