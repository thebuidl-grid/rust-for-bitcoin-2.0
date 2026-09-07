use std::str::FromStr;

use bdk_wallet::{
    CreateWithPersistError, KeychainKind, LoadWithPersistError, Wallet,
    descriptor::template::Bip84,
    keys::{
        GeneratableKey, GeneratedKey,
        bip39::{Language, Mnemonic, WordCount},
    },
    miniscript::Segwitv0,
};

use crate::{
    config::WalletConfig,
    core::WalletService,
    error::{WalletError, WalletResult},
    persistence::SqliteStore,
    types::WalletInitialization,
    types::{DerivedAddress, Keychain},
};

impl WalletService {
    pub fn initialize(
        config: &WalletConfig,
        imported_mnemonic: Option<&str>,
    ) -> WalletResult<(Self, WalletInitialization)> {
        let mut store = SqliteStore::open(&config.database_path)?;
        ensure_wallet_is_not_initialized(config, &mut store)?;

        let (mnemonic, recovery_phrase) = mnemonic(imported_mnemonic)?;
        let (external, internal) = bip84_descriptors(&mnemonic);

        let wallet = match Wallet::create(external, internal)
            .network(config.network)
            .create_wallet(store.connection_mut())
        {
            Ok(wallet) => wallet,
            Err(CreateWithPersistError::DataAlreadyExists(_)) => {
                return Err(WalletError::AlreadyInitialized(
                    config.database_path.clone(),
                ));
            }
            Err(CreateWithPersistError::Persist(error)) => return Err(error.into()),
            Err(CreateWithPersistError::Descriptor(error)) => return Err(error.into()),
        };

        let summary = WalletInitialization {
            network: wallet.network(),
            database_path: config.database_path.clone(),
            recovery_phrase,
            external_descriptor: wallet.public_descriptor(KeychainKind::External).to_string(),
            internal_descriptor: wallet.public_descriptor(KeychainKind::Internal).to_string(),
        };

        Ok((Self { wallet, store }, summary))
    }

    pub fn load(config: &WalletConfig, signing_mnemonic: Option<&str>) -> WalletResult<Self> {
        let mut store = SqliteStore::open(&config.database_path)?;
        let mut params = Wallet::load().check_network(config.network);

        if let Some(phrase) = signing_mnemonic {
            let mnemonic = parse_mnemonic(phrase)?;
            let (external, internal) = bip84_descriptors(&mnemonic);
            params = params
                .descriptor(KeychainKind::External, Some(external))
                .descriptor(KeychainKind::Internal, Some(internal))
                .extract_keys();
        }

        let wallet = match params.load_wallet(store.connection_mut()) {
            Ok(Some(wallet)) => wallet,
            Ok(None) => return Err(WalletError::NotInitialized(config.database_path.clone())),
            Err(LoadWithPersistError::Persist(error)) => return Err(error.into()),
            Err(LoadWithPersistError::InvalidChangeSet(error)) => {
                return Err(WalletError::Wallet(error.to_string()));
            }
        };

        Ok(Self { wallet, store })
    }

    pub fn next_address(&mut self, _keychain: Keychain) -> WalletResult<DerivedAddress> {
        Err(WalletError::NotImplemented("address derivation"))
    }
}

fn ensure_wallet_is_not_initialized(
    config: &WalletConfig,
    store: &mut SqliteStore,
) -> WalletResult<()> {
    match Wallet::load().load_wallet(store.connection_mut()) {
        Ok(Some(_)) => Err(WalletError::AlreadyInitialized(
            config.database_path.clone(),
        )),
        Ok(None) => Ok(()),
        Err(LoadWithPersistError::Persist(error)) => Err(error.into()),
        Err(LoadWithPersistError::InvalidChangeSet(error)) => {
            Err(WalletError::Wallet(error.to_string()))
        }
    }
}

fn mnemonic(imported: Option<&str>) -> WalletResult<(Mnemonic, Option<String>)> {
    match imported {
        Some(phrase) => Ok((parse_mnemonic(phrase)?, None)),
        None => {
            let generated: GeneratedKey<Mnemonic, Segwitv0> =
                Mnemonic::generate((WordCount::Words12, Language::English)).map_err(|error| {
                    WalletError::Wallet(format!("could not generate mnemonic: {error:?}"))
                })?;
            let mnemonic = generated.into_key();
            let recovery_phrase = mnemonic.to_string();
            Ok((mnemonic, Some(recovery_phrase)))
        }
    }
}

fn parse_mnemonic(phrase: &str) -> WalletResult<Mnemonic> {
    Mnemonic::from_str(phrase).map_err(|error| WalletError::InvalidMnemonic(error.to_string()))
}

fn bip84_descriptors(mnemonic: &Mnemonic) -> (Bip84<Mnemonic>, Bip84<Mnemonic>) {
    (
        Bip84(mnemonic.clone(), KeychainKind::External),
        Bip84(mnemonic.clone(), KeychainKind::Internal),
    )
}

#[cfg(test)]
mod tests {
    use bdk_wallet::KeychainKind;
    use bitcoin::Network;
    use tempfile::tempdir;

    use super::WalletService;
    use crate::{
        config::{RpcConfig, WalletConfig},
        error::WalletError,
    };

    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn config(data_dir: std::path::PathBuf) -> WalletConfig {
        WalletConfig::new(
            Network::Regtest,
            data_dir,
            RpcConfig::new("http://127.0.0.1:18443".into(), None, None),
        )
        .unwrap()
    }

    #[test]
    fn initializes_and_reopens_the_same_wallet() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));

        let (service, summary) = WalletService::initialize(&config, Some(MNEMONIC)).unwrap();
        assert!(summary.recovery_phrase.is_none());
        assert_eq!(summary.network, Network::Regtest);
        assert!(service.database_path().exists());

        drop(service);

        let reopened = WalletService::load(&config, Some(MNEMONIC)).unwrap();
        assert_eq!(reopened.wallet().network(), Network::Regtest);
        assert_eq!(
            reopened
                .wallet()
                .public_descriptor(KeychainKind::External)
                .to_string(),
            summary.external_descriptor
        );
        assert_eq!(
            reopened
                .wallet()
                .public_descriptor(KeychainKind::Internal)
                .to_string(),
            summary.internal_descriptor
        );
    }

    #[test]
    fn generates_a_twelve_word_recovery_phrase() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));

        let (_service, summary) = WalletService::initialize(&config, None).unwrap();
        let phrase = summary.recovery_phrase.unwrap();

        assert_eq!(phrase.split_whitespace().count(), 12);
        WalletService::load(&config, Some(&phrase)).unwrap();
    }

    #[test]
    fn refuses_to_reinitialize_an_existing_wallet() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        WalletService::initialize(&config, Some(MNEMONIC)).unwrap();

        let result = WalletService::initialize(&config, Some(MNEMONIC));

        assert!(matches!(result, Err(WalletError::AlreadyInitialized(_))));
    }

    #[test]
    fn checks_for_an_existing_wallet_before_parsing_a_mnemonic() {
        let temp = tempdir().unwrap();
        let config = config(temp.path().join("wallet"));
        WalletService::initialize(&config, Some(MNEMONIC)).unwrap();

        let result = WalletService::initialize(&config, Some("not a valid mnemonic"));

        assert!(matches!(result, Err(WalletError::AlreadyInitialized(_))));
    }
}
