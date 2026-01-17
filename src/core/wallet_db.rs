
use rusqlite::{Connection, params};
use serde_json::json;

pub struct WalletDb {
    pub db_path: String,
    pub conn: Connection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
    pub address: String,
    pub label: String,
    pub public_key: String,
    pub encrypted_private_key: String,
    pub balance: f64,
    pub created: i64,
    pub is_locked: bool,
    pub available_balance: f64,
}

impl WalletDb {
    pub fn new(db_path: &str) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open wallet db");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wallets (
                address TEXT PRIMARY KEY,
                label TEXT,
                public_key TEXT,
                encrypted_private_key TEXT,
                balance REAL,
                created INTEGER,
                metadata TEXT
            )",
            [],
        ).unwrap();
        WalletDb { db_path: db_path.to_string(), conn }
    }

    pub fn save_wallet(&self, wallet: &Wallet) -> bool {
        let meta = json!({
            "is_locked": wallet.is_locked,
            "available_balance": wallet.available_balance
        });
        self.conn.execute(
            "REPLACE INTO wallets (address, label, public_key, encrypted_private_key, balance, created, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                wallet.address,
                wallet.label,
                wallet.public_key,
                wallet.encrypted_private_key,
                wallet.balance,
                wallet.created,
                meta.to_string()
            ]
        ).is_ok()
    }

    pub fn load_wallet(&self, address: &str) -> Option<Wallet> {
        let mut stmt = self.conn.prepare("SELECT address, label, public_key, encrypted_private_key, balance, created, metadata FROM wallets WHERE address=?1").ok()?;
        let mut rows = stmt.query(params![address]).ok()?;
        if let Some(row) = rows.next().ok()? {
            let meta_str: String = row.get(6).unwrap_or_default();
            let meta: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or_default();
            Some(Wallet {
                address: row.get(0).unwrap_or_default(),
                label: row.get(1).unwrap_or_default(),
                public_key: row.get(2).unwrap_or_default(),
                encrypted_private_key: row.get(3).unwrap_or_default(),
                balance: row.get(4).unwrap_or(0.0),
                created: row.get(5).unwrap_or(0),
                is_locked: meta.get("is_locked").and_then(|v| v.as_bool()).unwrap_or(false),
                available_balance: meta.get("available_balance").and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
        } else {
            None
        }
    }

    pub fn list_wallets(&self) -> Vec<Wallet> {
        let mut stmt = self.conn.prepare("SELECT address, label, public_key, encrypted_private_key, balance, created, metadata FROM wallets").unwrap();
        let rows = stmt.query_map([], |row| {
            let meta_str: String = row.get(6).unwrap_or_default();
            let meta: serde_json::Value = serde_json::from_str(&meta_str).unwrap_or_default();
            Ok(Wallet {
                address: row.get(0).unwrap_or_default(),
                label: row.get(1).unwrap_or_default(),
                public_key: row.get(2).unwrap_or_default(),
                encrypted_private_key: row.get(3).unwrap_or_default(),
                balance: row.get(4).unwrap_or(0.0),
                created: row.get(5).unwrap_or(0),
                is_locked: meta.get("is_locked").and_then(|v| v.as_bool()).unwrap_or(false),
                available_balance: meta.get("available_balance").and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
        }).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn close(self) {
        // rusqlite::ConnectionはDropで自動クローズ
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = format!("test_wallets_{}.db", nanos);
        let _ = fs::remove_file(&path);
        path
    }

    fn sample_wallet(addr: &str) -> Wallet {
        Wallet {
            address: addr.to_string(),
            label: "label".to_string(),
            public_key: "pubkey".to_string(),
            encrypted_private_key: "encpriv".to_string(),
            balance: 42.0,
            created: 123456,
            is_locked: true,
            available_balance: 41.0,
        }
    }

    #[test]
    fn test_save_and_load_wallet() {
        let db = WalletDb::new(&temp_db());
        let w = sample_wallet("addr1");
        assert!(db.save_wallet(&w));
        let loaded = db.load_wallet("addr1").unwrap();
        assert_eq!(w, loaded);
    }

    #[test]
    fn test_list_wallets() {
        let db = WalletDb::new(&temp_db());
        let w1 = sample_wallet("a1");
        let w2 = sample_wallet("a2");
        db.save_wallet(&w1);
        db.save_wallet(&w2);
        let wallets = db.list_wallets();
        assert_eq!(wallets.len(), 2);
        assert!(wallets.iter().any(|w| w.address == "a1"));
        assert!(wallets.iter().any(|w| w.address == "a2"));
    }
}
