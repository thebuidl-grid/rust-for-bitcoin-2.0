//! Wallet metadata stored next to BDK's own tables.
//!
//! BDK's SQLite persister owns the chain data, the transaction graph and the
//! *public* descriptors. It deliberately does not store private keys. That
//! leaves a small gap: on reopen we need to know which script type the wallet
//! was created with so we can rebuild the matching private descriptors from the
//! mnemonic. One extra table closes it.

use bdk_wallet::rusqlite::{Connection, OptionalExtension};

use crate::error::{Error, Result};
use crate::keys::DescriptorKind;

const TABLE: &str = "rfb_wallet_meta";

pub const KEY_DESCRIPTOR_KIND: &str = "descriptor_kind";
pub const KEY_BIRTHDAY: &str = "birthday_height";
pub const KEY_FINGERPRINT: &str = "master_fingerprint";
pub const KEY_CREATED_AT: &str = "created_at";

pub fn open(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute(
        &format!("CREATE TABLE IF NOT EXISTS {TABLE} (key TEXT PRIMARY KEY, value TEXT NOT NULL)"),
        [],
    )?;
    Ok(conn)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        &format!("INSERT INTO {TABLE} (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2"),
        (key, value),
    )?;
    Ok(())
}

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row(
            &format!("SELECT value FROM {TABLE} WHERE key = ?1"),
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

pub fn require(conn: &Connection, key: &'static str) -> Result<String> {
    get(conn, key)?.ok_or(Error::MissingMetadata(key))
}

pub fn descriptor_kind(conn: &Connection) -> Result<DescriptorKind> {
    DescriptorKind::parse(&require(conn, KEY_DESCRIPTOR_KIND)?)
}

pub fn birthday(conn: &Connection) -> Result<u32> {
    Ok(get(conn, KEY_BIRTHDAY)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory database");
        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {TABLE} (key TEXT PRIMARY KEY, value TEXT NOT NULL)"
            ),
            [],
        )
        .expect("create table");
        conn
    }

    #[test]
    fn round_trips_a_value() {
        let conn = memory();
        set(&conn, KEY_DESCRIPTOR_KIND, "tr").expect("set");
        assert_eq!(
            get(&conn, KEY_DESCRIPTOR_KIND).expect("get").as_deref(),
            Some("tr")
        );
        assert_eq!(descriptor_kind(&conn).expect("kind"), DescriptorKind::Tr);
    }

    #[test]
    fn overwrites_rather_than_conflicting() {
        let conn = memory();
        set(&conn, KEY_BIRTHDAY, "100").expect("set");
        set(&conn, KEY_BIRTHDAY, "250").expect("set again");
        assert_eq!(birthday(&conn).expect("birthday"), 250);
    }

    #[test]
    fn a_missing_key_is_absent_not_an_error() {
        let conn = memory();
        assert!(get(&conn, KEY_FINGERPRINT).expect("get").is_none());
        assert!(require(&conn, KEY_FINGERPRINT).is_err());
    }

    #[test]
    fn birthday_defaults_to_zero() {
        let conn = memory();
        assert_eq!(birthday(&conn).expect("birthday"), 0);
        set(&conn, KEY_BIRTHDAY, "not a number").expect("set");
        assert_eq!(birthday(&conn).expect("birthday"), 0);
    }
}
