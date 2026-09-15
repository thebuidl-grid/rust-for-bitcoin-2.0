//! Raw transaction parsing tests for `trxparse`.
//!
//! `trxparse` walks the same bytes as `decodetrx` but with a `Cursor` and
//! hand-built JSON, so these tests double as a cross-check: both crates must
//! agree on the fields they share.

use trxparse::parse_transaction;

const LEGACY_TX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

const SEGWIT_TX: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

#[test]
fn parses_legacy_transaction() {
    let tx = parse_transaction(LEGACY_TX).expect("legacy tx should parse");

    assert_eq!(tx["version"], 1);
    assert_eq!(tx["segwit"], false);
    assert_eq!(tx["locktime"], 0);
    assert_eq!(tx["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(tx["outputs"].as_array().unwrap().len(), 2);

    assert_eq!(
        tx["inputs"][0]["prev_txid"],
        "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9"
    );
    assert_eq!(tx["outputs"][0]["value_sats"], 1_000_000_000u64);
    assert_eq!(tx["outputs"][1]["value_sats"], 4_000_000_000u64);
}

#[test]
fn legacy_input_has_no_witness_key() {
    let tx = parse_transaction(LEGACY_TX).unwrap();
    assert!(tx["inputs"][0].get("witness").is_none());
}

#[test]
fn parses_segwit_transaction() {
    let tx = parse_transaction(SEGWIT_TX).expect("segwit tx should parse");

    assert_eq!(tx["version"], 2);
    assert_eq!(tx["segwit"], true);
    assert_eq!(tx["locktime"], 0);

    assert_eq!(
        tx["inputs"][0]["prev_txid"],
        "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796"
    );
    assert_eq!(tx["inputs"][0]["vout"], 1);
    assert_eq!(tx["inputs"][0]["script_sig"], "");
    assert_eq!(tx["inputs"][0]["witness"].as_array().unwrap().len(), 2);

    assert_eq!(tx["outputs"][0]["value_sats"], 100);
    assert_eq!(tx["outputs"][1]["value_sats"], 4_462_282u64);
}

#[test]
fn agrees_with_decodetrx_on_shared_fields() {
    for raw in [LEGACY_TX, SEGWIT_TX] {
        let parsed = parse_transaction(raw).unwrap();
        let decoded: serde_json::Value =
            serde_json::from_str(&decodetrx::decode_transaction(raw.to_string()).unwrap()).unwrap();

        assert_eq!(parsed["version"], decoded["version"]);
        assert_eq!(parsed["locktime"], decoded["lock_time"]);
        assert_eq!(
            parsed["inputs"].as_array().unwrap().len(),
            decoded["inputs"].as_array().unwrap().len()
        );
        assert_eq!(parsed["inputs"][0]["prev_txid"], decoded["inputs"][0]["txid"]);
        assert_eq!(
            parsed["inputs"][0]["script_sig"],
            decoded["inputs"][0]["script_sig"]
        );
        assert_eq!(
            parsed["outputs"][0]["script_pubkey"],
            decoded["outputs"][0]["script_pubkey"]
        );
    }
}

#[test]
fn rejects_malformed_input() {
    assert!(parse_transaction("not hex").is_err());
    assert!(parse_transaction("010000000").is_err(), "odd-length hex");
    assert!(
        parse_transaction(&LEGACY_TX[..LEGACY_TX.len() / 2]).is_err(),
        "truncated"
    );
    assert!(
        parse_transaction(&format!("{}deadbeef", LEGACY_TX)).is_err(),
        "trailing bytes"
    );
}
