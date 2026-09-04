use std::str::FromStr;

use bdk_wallet::bitcoin::bip32::DerivationPath;
use bdk_wallet::bitcoin::secp256k1::Secp256k1;
use bdk_wallet::bitcoin::{Network, NetworkKind};
use bdk_wallet::descriptor;
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::GeneratableKey;
use bdk_wallet::miniscript;

/// Generate a fresh 12-word BIP39 mnemonic. Only used by `init` when no `MNEMONIC` is
/// already set in the environment — an existing one is never overwritten.
pub fn generate_mnemonic() -> anyhow::Result<String> {
    let mnemonic: bdk_wallet::keys::GeneratedKey<_, bdk_wallet::miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| anyhow::anyhow!("mnemonic generation failed"))?;
    Ok(mnemonic.to_string())
}

/// BIP84 external/internal wpkh descriptors (with private key material) for a mnemonic.
///
/// We use wpkh (native SegWit, BIP84) rather than taproot: it's the format the address
/// labs already covered, keeps `bitcoin-cli`/BDK output easy to eyeball, and the
/// discounted witness weight it buys over legacy P2PKH is the whole reason SegWit
/// wallets default to it today.
pub fn wpkh_descriptors(
    mnemonic_phrase: &str,
    network: Network,
) -> anyhow::Result<(String, String)> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
        .map_err(|error| anyhow::anyhow!("invalid mnemonic: {error}"))?;

    let network_kind = match network {
        Network::Bitcoin => NetworkKind::Main,
        _ => NetworkKind::Test,
    };
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let secp = Secp256k1::new();
    let external_path = DerivationPath::from_str(&format!("m/84h/{coin_type}h/0h/0"))?;
    let internal_path = DerivationPath::from_str(&format!("m/84h/{coin_type}h/0h/1"))?;
    let key = (mnemonic, None::<String>);

    let (external_descriptor, ext_keymap) = descriptor!(wpkh((key.clone(), external_path)))?
        .into_wallet_descriptor(&secp, network_kind)?;
    let (internal_descriptor, int_keymap) =
        descriptor!(wpkh((key, internal_path)))?.into_wallet_descriptor(&secp, network_kind)?;

    Ok((
        external_descriptor.to_string_with_secret(&ext_keymap),
        internal_descriptor.to_string_with_secret(&int_keymap),
    ))
}
