//! Generate a fresh BIP39 mnemonic and persist it to `.env` (never to source
//! under version control) so later runs reuse the same wallet.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use bdk_wallet::keys::GeneratableKey;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::miniscript::Tap;

/// Generate a fresh 12-word English mnemonic.
///
/// The `Tap` script-context bound is only needed to satisfy
/// `GeneratableKey`'s generic signature; mnemonic generation itself does not
/// depend on which descriptor type the words end up deriving.
pub fn generate() -> Result<String> {
    let generated: bdk_wallet::keys::GeneratedKey<Mnemonic, Tap> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| anyhow!("failed to generate a BIP39 mnemonic"))?;
    Ok(generated.to_string())
}

/// Append `MNEMONIC="<phrase>"` to the `.env` file at `path`, creating it if
/// needed. Refuses to run if a `MNEMONIC` line is already present, since
/// overwriting it would orphan whatever wallet was derived from the old one.
pub fn append_to_env_file(path: &Path, phrase: &str) -> Result<()> {
    if path.exists() {
        let existing = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if existing
            .lines()
            .any(|line| line.trim_start().starts_with("MNEMONIC="))
        {
            return Err(anyhow!(
                "{} already has a MNEMONIC entry; remove it first if you really want to replace the wallet",
                path.display()
            ));
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {} for writing", path.display()))?;
    writeln!(file, "MNEMONIC=\"{phrase}\"")
        .with_context(|| format!("failed to write MNEMONIC to {}", path.display()))?;
    Ok(())
}
