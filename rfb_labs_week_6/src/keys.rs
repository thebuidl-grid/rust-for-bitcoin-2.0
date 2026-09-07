//! Seed handling and descriptor construction.
//!
//! The descriptors are assembled here by hand with `rust-bitcoin`'s BIP32 types
//! rather than with BDK's `Bip84`/`Bip86` templates. The templates would produce
//! the same strings, but writing them out keeps the key origin, the account path
//! and the two keychain branches visible in one place, which is the part of a
//! descriptor wallet worth being explicit about.

use bdk_wallet::bip39::{Language, Mnemonic};
use bitcoin::Network;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv};
use bitcoin::secp256k1::Secp256k1;
use clap::ValueEnum;

use crate::error::{Error, Result};

/// Which single-key script type the wallet derives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DescriptorKind {
    /// Native SegWit v0, `wpkh(...)`, BIP84 account path.
    Wpkh,
    /// Taproot key spend, `tr(...)`, BIP86 account path.
    Tr,
}

impl DescriptorKind {
    /// BIP43 purpose field: 84 for native SegWit, 86 for Taproot.
    pub fn purpose(self) -> u32 {
        match self {
            DescriptorKind::Wpkh => 84,
            DescriptorKind::Tr => 86,
        }
    }

    pub fn bip(self) -> &'static str {
        match self {
            DescriptorKind::Wpkh => "BIP84",
            DescriptorKind::Tr => "BIP86",
        }
    }

    /// The miniscript fragment name used to wrap the key.
    pub fn fragment(self) -> &'static str {
        match self {
            DescriptorKind::Wpkh => "wpkh",
            DescriptorKind::Tr => "tr",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DescriptorKind::Wpkh => "wpkh",
            DescriptorKind::Tr => "tr",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "wpkh" => Ok(DescriptorKind::Wpkh),
            "tr" => Ok(DescriptorKind::Tr),
            other => Err(Error::Config(format!(
                "unknown descriptor kind `{other}` in the wallet database"
            ))),
        }
    }
}

/// A matched pair of descriptors plus the origin they were derived under.
#[derive(Debug, Clone)]
pub struct Descriptors {
    /// Receiving branch, `.../0/*`. Contains the account xprv.
    pub external: String,
    /// Change branch, `.../1/*`. Contains the account xprv.
    pub internal: String,
    pub master_fingerprint: Fingerprint,
    pub account_path: DerivationPath,
}

/// Generate a fresh 12-word English mnemonic from the OS entropy source.
pub fn generate_mnemonic() -> Result<Mnemonic> {
    let mut entropy = [0u8; 16];
    bitcoin::secp256k1::rand::RngCore::fill_bytes(
        &mut bitcoin::secp256k1::rand::thread_rng(),
        &mut entropy,
    );
    Ok(Mnemonic::from_entropy_in(Language::English, &entropy)?)
}

/// Turn a mnemonic into the external/internal descriptor pair for `kind`.
///
/// The path is `m/{purpose}h/{coin}h/0h`, with the keychain branch (`0` for
/// receiving, `1` for change) appended as an unhardened wildcard so a single
/// account key covers both branches.
pub fn descriptors(
    mnemonic: &str,
    passphrase: &str,
    network: Network,
    coin_type: u32,
    kind: DescriptorKind,
) -> Result<Descriptors> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
    let seed = mnemonic.to_seed(passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network, &seed)?;
    let master_fingerprint = master.fingerprint(&secp);

    let account_path: DerivationPath =
        format!("m/{}h/{}h/0h", kind.purpose(), coin_type)
            .parse()
            .map_err(|e| Error::wallet("building the account derivation path", e))?;
    let account_xprv = master.derive_priv(&secp, &account_path)?;

    // `DerivationPath` prints without a leading "m", which is exactly the form a key
    // origin takes: [73c5da0a/84'/1'/0'].
    let prefix = format!("[{master_fingerprint}/{account_path}]{account_xprv}");

    Ok(Descriptors {
        external: format!("{}({prefix}/0/*)", kind.fragment()),
        internal: format!("{}({prefix}/1/*)", kind.fragment()),
        master_fingerprint,
        account_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdk_wallet::{KeychainKind, Wallet};

    /// The BIP39 test mnemonic used by BIP84 and BIP86. Published, and must
    /// never hold real funds.
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn address(kind: DescriptorKind, keychain: KeychainKind, index: u32) -> String {
        let d = descriptors(TEST_MNEMONIC, "", Network::Bitcoin, 0, kind).expect("descriptors");
        let wallet = Wallet::create(d.external, d.internal)
            .network(Network::Bitcoin)
            .create_wallet_no_persist()
            .expect("wallet");
        wallet.peek_address(keychain, index).address.to_string()
    }

    #[test]
    fn matches_bip84_test_vectors() {
        // https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki
        assert_eq!(
            address(DescriptorKind::Wpkh, KeychainKind::External, 0),
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
        );
        assert_eq!(
            address(DescriptorKind::Wpkh, KeychainKind::External, 1),
            "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g"
        );
        assert_eq!(
            address(DescriptorKind::Wpkh, KeychainKind::Internal, 0),
            "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el"
        );
    }

    #[test]
    fn matches_bip86_test_vectors() {
        // https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki
        assert_eq!(
            address(DescriptorKind::Tr, KeychainKind::External, 0),
            "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"
        );
        assert_eq!(
            address(DescriptorKind::Tr, KeychainKind::Internal, 0),
            "bc1p3qkhfews2uk44qtvauqyr2ttdsw7svhkl9nkm9s9c3x4ax5h60wqwruhk7"
        );
    }

    /// The account xpub is a published vector in its own right, and it pins the
    /// derivation path independently of how addresses are then encoded.
    #[test]
    fn derives_the_bip86_account_xpub() {
        let d = descriptors(TEST_MNEMONIC, "", Network::Bitcoin, 0, DescriptorKind::Tr)
            .expect("descriptors");
        let wallet = Wallet::create(d.external, d.internal)
            .network(Network::Bitcoin)
            .create_wallet_no_persist()
            .expect("wallet");

        let expected = concat!(
            "tr([73c5da0a/86'/0'/0']",
            "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9",
            "QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ/0/*)#rg247h69"
        );
        assert_eq!(
            wallet.public_descriptor(KeychainKind::External).to_string(),
            expected
        );
    }

    #[test]
    fn descriptor_carries_the_key_origin() {
        let d = descriptors(TEST_MNEMONIC, "", Network::Regtest, 1, DescriptorKind::Wpkh)
            .expect("descriptors");

        assert!(d.external.starts_with("wpkh([73c5da0a/84'/1'/0']tprv"));
        assert!(d.external.ends_with("/0/*)"));
        assert!(d.internal.ends_with("/1/*)"));
        assert_eq!(d.master_fingerprint.to_string(), "73c5da0a");
        assert_eq!(d.account_path.to_string(), "84'/1'/0'");
    }

    #[test]
    fn the_two_keychains_are_different_branches() {
        let d = descriptors(TEST_MNEMONIC, "", Network::Regtest, 1, DescriptorKind::Wpkh)
            .expect("descriptors");
        assert_ne!(d.external, d.internal);
        assert_eq!(d.external.replace("/0/*)", "/1/*)"), d.internal);
    }

    #[test]
    fn a_passphrase_produces_a_different_wallet() {
        let plain = descriptors(TEST_MNEMONIC, "", Network::Regtest, 1, DescriptorKind::Wpkh)
            .expect("descriptors");
        let salted = descriptors(
            TEST_MNEMONIC,
            "correct horse",
            Network::Regtest,
            1,
            DescriptorKind::Wpkh,
        )
        .expect("descriptors");

        assert_ne!(plain.master_fingerprint, salted.master_fingerprint);
        assert_ne!(plain.external, salted.external);
    }

    #[test]
    fn taproot_uses_purpose_86() {
        let d = descriptors(TEST_MNEMONIC, "", Network::Regtest, 1, DescriptorKind::Tr)
            .expect("descriptors");
        assert_eq!(d.account_path.to_string(), "86'/1'/0'");
        assert!(d.external.starts_with("tr("));
    }

    #[test]
    fn rejects_an_invalid_mnemonic() {
        let err = descriptors(
            "abandon abandon abandon",
            "",
            Network::Regtest,
            1,
            DescriptorKind::Wpkh,
        );
        assert!(err.is_err());
    }

    #[test]
    fn generated_mnemonics_are_twelve_words_and_distinct() {
        let a = generate_mnemonic().expect("mnemonic").to_string();
        let b = generate_mnemonic().expect("mnemonic").to_string();
        assert_eq!(a.split_whitespace().count(), 12);
        assert_ne!(a, b);
    }

    #[test]
    fn descriptor_kind_round_trips() {
        for kind in [DescriptorKind::Wpkh, DescriptorKind::Tr] {
            assert_eq!(DescriptorKind::parse(kind.as_str()).expect("parse"), kind);
        }
        assert!(DescriptorKind::parse("p2pkh").is_err());
    }
}
