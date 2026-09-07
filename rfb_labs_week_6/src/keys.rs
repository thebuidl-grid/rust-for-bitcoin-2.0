use anyhow::Result;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
use bdk_wallet::miniscript::Segwitv0;


pub fn generate_master_key(network: Network) -> Result<(Mnemonic, Xpriv)> {
    let mnemonic: GeneratedKey<_, Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English))
        .map_err(|e| anyhow::anyhow!("mnemonic generation failed"))?;

    let mnemonic = mnemonic.into_key();

    let seed = mnemonic.to_seed("");
    let master_xprv = Xpriv::new_master(network, &seed)?;

    Ok((mnemonic, master_xprv))
}

pub fn load_master_key(xprv_str: &str) -> Result<Xpriv> {
    xprv_str
        .parse::<Xpriv>()
        .map_err(|e| anyhow::anyhow!("invalid WALLET_XPRV in .env: {e}"))
}

pub fn load_or_generate_master_key(env_path: &str, network: Network) -> Result<Xpriv> {
    let _ = dotenvy::from_path(env_path);

    if let Ok(existing) = std::env::var("WALLET_XPRV") {
        return load_master_key(&existing);
    }

    let (mnemonic, master_xprv) = generate_master_key(network)?;
    println!("Generated a new wallet key (mnemonic shown once, for backup only):");
    println!("  {mnemonic}");

    let line = format!("WALLET_XPRV={master_xprv}\n");
    std::fs::write(env_path, line)?;
    println!("Saved xprv to {env_path} (gitignored) -- do not commit this file.");

    Ok(master_xprv)
}
