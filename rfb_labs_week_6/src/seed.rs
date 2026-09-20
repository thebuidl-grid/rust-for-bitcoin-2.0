use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bip39::Mnemonic;

/// Path to the local, gitignored file that persists a generated mnemonic
/// across restarts when one isn't supplied via `.env`.
const SEED_FILE: &str = "wallet_seed.txt";

/// Resolve a stable BIP39 mnemonic for this wallet, in priority order:
///
/// 1. `WALLET_MNEMONIC` in the environment (`.env`), if set — lets the
///    wallet's identity be pinned explicitly, e.g. for testing against a
///    known set of addresses.
/// 2. A previously generated mnemonic saved at `wallet_seed.txt`, if this
///    isn't the first run.
/// 3. Otherwise, generate a fresh random mnemonic and save it to
///    `wallet_seed.txt` so this same wallet identity is reused on every
///    future run.
///
/// This function is what makes "restart without losing wallet state"
/// possible: as long as the same mnemonic comes back on every run, BDK
/// derives the same descriptor, the same addresses, and can find the same
/// UTXOs again after a resync.
pub fn resolve_mnemonic() -> Result<Mnemonic> {
    if let Ok(from_env) = std::env::var("WALLET_MNEMONIC") {
        let mnemonic = from_env
            .parse::<Mnemonic>()
            .context("WALLET_MNEMONIC in .env is not a valid BIP39 mnemonic")?;
        return Ok(mnemonic);
    }

    let path = Path::new(SEED_FILE);
    if path.exists() {
        let saved = fs::read_to_string(path)
            .with_context(|| format!("failed to read existing {SEED_FILE}"))?;
        let mnemonic = saved
            .trim()
            .parse::<Mnemonic>()
            .with_context(|| format!("{SEED_FILE} does not contain a valid mnemonic"))?;
        return Ok(mnemonic);
    }

    // First run with no override and no saved seed: generate a fresh one
    // and persist it immediately, before returning it, so this exact
    // mnemonic is what gets reused on every future run.
    let mnemonic = Mnemonic::generate(12).context("failed to generate a new mnemonic")?;
    fs::write(path, mnemonic.to_string())
        .with_context(|| format!("failed to save generated mnemonic to {SEED_FILE}"))?;

    println!(
        "Generated a new wallet seed and saved it to {SEED_FILE} \
         (this file is gitignored — never commit it)."
    );

    Ok(mnemonic)
}
