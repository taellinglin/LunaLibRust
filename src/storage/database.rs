
use rusqlite::{params, Connection, Result};
use serde_json::{Value as JsonValue, json};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct WalletDatabase {
    pub db_path: PathBuf,
}

impl WalletDatabase {
    pub fn new(db_path: Option<PathBuf>) -> Self {
        let db_path = db_path.unwrap_or_else(|| {
            let mut home = dirs::home_dir().unwrap();
            home.push(".luna_wallet/wallets.db");
            home
        });
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let db = WalletDatabase { db_path };
        db.init_database();
        db
    }

    fn init_database(&self) {
        let conn = Connection::open(&self.db_path).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wallets (
                address TEXT PRIMARY KEY,
                label TEXT,
                public_key TEXT,
                encrypted_private_key TEXT,
                balance REAL DEFAULT 0.0,
                created REAL,
                last_accessed REAL,
                metadata TEXT
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                tx_hash TEXT PRIMARY KEY,
                wallet_address TEXT,
                tx_type TEXT,
                from_address TEXT,
                to_address TEXT,
                amount REAL,
                fee REAL,
                timestamp REAL,
                block_height INTEGER,
                status TEXT,
                memo TEXT,
                raw_data TEXT
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pending_transactions (
                tx_hash TEXT PRIMARY KEY,
                wallet_address TEXT,
                from_address TEXT,
                to_address TEXT,
                amount REAL,
                fee REAL,
                created_time REAL,
                status TEXT DEFAULT 'pending',
                retry_count INTEGER DEFAULT 0,
                last_retry REAL,
                raw_data TEXT
            )",
            [],
        ).unwrap();
    }

    pub fn save_wallet(&self, wallet_data: &JsonValue) -> bool {
        let conn = Connection::open(&self.db_path).unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let res = conn.execute(
            "INSERT OR REPLACE INTO wallets (address, label, public_key, encrypted_private_key, balance, created, last_accessed, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                wallet_data["address"].as_str().unwrap_or("") ,
                wallet_data.get("label").and_then(|v| v.as_str()).unwrap_or("") ,
                wallet_data.get("public_key").and_then(|v| v.as_str()).unwrap_or("") ,
                wallet_data.get("encrypted_private_key").and_then(|v| v.as_str()).unwrap_or("") ,
                wallet_data.get("balance").and_then(|v| v.as_f64()).unwrap_or(0.0) ,
                wallet_data.get("created").and_then(|v| v.as_f64()).unwrap_or(now) ,
                now ,
                wallet_data.get("metadata").map(|v| v.to_string()).unwrap_or("{}".to_string())
            ]
        );
        res.is_ok()
    }

    pub fn load_wallet(&self, address: &str) -> Option<JsonValue> {
        let conn = Connection::open(&self.db_path).unwrap();
        let mut stmt = conn.prepare("SELECT * FROM wallets WHERE address = ?").unwrap();
        let mut rows = stmt.query(params![address]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            let metadata_str: String = row.get(7).unwrap_or("{}".to_string());
            let metadata = serde_json::from_str(&metadata_str).unwrap_or(json!({}));
            Some(json!({
                "address": row.get::<_, String>(0).unwrap_or_default(),
                "label": row.get::<_, String>(1).unwrap_or_default(),
                "public_key": row.get::<_, String>(2).unwrap_or_default(),
                "encrypted_private_key": row.get::<_, String>(3).unwrap_or_default(),
                "balance": row.get::<_, f64>(4).unwrap_or(0.0),
                "created": row.get::<_, f64>(5).unwrap_or(0.0),
                "last_accessed": row.get::<_, f64>(6).unwrap_or(0.0),
                "metadata": metadata
            }))
        } else {
            None
        }
    }

    pub fn save_transaction(&self, transaction: &JsonValue, wallet_address: &str) -> bool {
        let conn = Connection::open(&self.db_path).unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let res = conn.execute(
            "INSERT OR REPLACE INTO transactions (tx_hash, wallet_address, tx_type, from_address, to_address, amount, fee, timestamp, block_height, status, memo, raw_data) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                transaction.get("hash").and_then(|v| v.as_str()).unwrap_or("") ,
                wallet_address ,
                transaction.get("type").and_then(|v| v.as_str()).unwrap_or("transfer") ,
                transaction.get("from").and_then(|v| v.as_str()).unwrap_or("") ,
                transaction.get("to").and_then(|v| v.as_str()).unwrap_or("") ,
                transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0) ,
                transaction.get("fee").and_then(|v| v.as_f64()).unwrap_or(0.0) ,
                transaction.get("timestamp").and_then(|v| v.as_f64()).unwrap_or(now) ,
                transaction.get("block_height").and_then(|v| v.as_i64()).unwrap_or(0) ,
                transaction.get("status").and_then(|v| v.as_str()).unwrap_or("confirmed") ,
                transaction.get("memo").and_then(|v| v.as_str()).unwrap_or("") ,
                transaction.to_string()
            ]
        );
        res.is_ok()
    }

    pub fn get_wallet_transactions(&self, wallet_address: &str, limit: usize) -> Vec<JsonValue> {
        let conn = Connection::open(&self.db_path).unwrap();
        let mut stmt = conn.prepare("SELECT raw_data FROM transactions WHERE wallet_address = ? ORDER BY timestamp DESC LIMIT ?").unwrap();
        let mut rows = stmt.query(params![wallet_address, limit as i64]).unwrap();
        let mut txs = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            let raw: String = row.get(0).unwrap_or("{}".to_string());
            if let Ok(tx) = serde_json::from_str(&raw) {
                txs.push(tx);
            }
        }
        txs
    }

    pub fn save_pending_transaction(&self, transaction: &JsonValue, wallet_address: &str) -> bool {
        let conn = Connection::open(&self.db_path).unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let res = conn.execute(
            "INSERT OR REPLACE INTO pending_transactions (tx_hash, wallet_address, from_address, to_address, amount, fee, created_time, status, raw_data) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                transaction.get("hash").and_then(|v| v.as_str()).unwrap_or("") ,
                wallet_address ,
                transaction.get("from").and_then(|v| v.as_str()).unwrap_or("") ,
                transaction.get("to").and_then(|v| v.as_str()).unwrap_or("") ,
                transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0) ,
                transaction.get("fee").and_then(|v| v.as_f64()).unwrap_or(0.0) ,
                now ,
                "pending" ,
                transaction.to_string()
            ]
        );
        res.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;

    #[test]
    fn test_wallet_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_wallets.db");
        let db = WalletDatabase::new(Some(db_path.clone()));
        let wallet = json!({
            "address": "addr1",
            "label": "main",
            "public_key": "pubkey",
            "encrypted_private_key": "privkey",
            "balance": 123.45,
            "created": 1234567890.0,
            "metadata": {"foo": "bar"}
        });
        assert!(db.save_wallet(&wallet));
        let loaded = db.load_wallet("addr1").unwrap();
        assert_eq!(loaded["address"], "addr1");
        assert_eq!(loaded["label"], "main");
        assert_eq!(loaded["public_key"], "pubkey");
        assert_eq!(loaded["encrypted_private_key"], "privkey");
        assert_eq!(loaded["balance"], 123.45);
        assert_eq!(loaded["metadata"]["foo"], "bar");
    }

    #[test]
    fn test_transaction_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_wallets.db");
        let db = WalletDatabase::new(Some(db_path.clone()));
        let wallet = json!({"address": "addr2"});
        db.save_wallet(&wallet);
        let tx = json!({
            "hash": "tx1",
            "type": "transfer",
            "from": "addr2",
            "to": "addr3",
            "amount": 10.0,
            "fee": 0.1,
            "block_height": 1,
            "status": "confirmed",
            "memo": "test"
        });
        assert!(db.save_transaction(&tx, "addr2"));
        let txs = db.get_wallet_transactions("addr2", 10);
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0]["hash"], "tx1");
        assert_eq!(txs[0]["amount"], 10.0);
        assert_eq!(txs[0]["memo"], "test");
    }

    #[test]
    fn test_pending_transaction() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_wallets.db");
        let db = WalletDatabase::new(Some(db_path.clone()));
        let tx = json!({
            "hash": "pending1",
            "from": "addr4",
            "to": "addr5",
            "amount": 5.0,
            "fee": 0.05
        });
        assert!(db.save_pending_transaction(&tx, "addr4"));
    }
}
