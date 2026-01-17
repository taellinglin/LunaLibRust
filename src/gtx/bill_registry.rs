use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillInfo {
    pub bill_serial: String,
    pub denomination: i64,
    pub user_address: String,
    pub hash: String,
    pub mining_time: f64,
    pub difficulty: i64,
    pub luna_value: f64,
    pub timestamp: f64,
    pub verification_url: String,
    pub image_url: String,
    pub metadata: JsonValue,
    pub status: String,
}

pub struct BillRegistry {
    db_path: PathBuf,
}

impl BillRegistry {
    pub fn new(db_path: Option<PathBuf>) -> Self {
        let db_path = db_path.unwrap_or_else(|| {
            let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.push(".luna_wallet");
            std::fs::create_dir_all(&home).ok();
            home.push("bills.db");
            home
        });
        let reg = BillRegistry { db_path };
        reg.init_database().expect("Failed to init bill db");
        reg
    }

    fn init_database(&self) -> SqlResult<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bills (
                bill_serial TEXT PRIMARY KEY,
                denomination INTEGER,
                user_address TEXT,
                hash TEXT,
                mining_time REAL,
                difficulty INTEGER,
                luna_value REAL,
                timestamp REAL,
                verification_url TEXT,
                image_url TEXT,
                metadata TEXT,
                status TEXT DEFAULT 'active'
            )",
            [],
        )?;
        Ok(())
    }

    pub fn register_bill(&self, mut bill_info: BillInfo) -> SqlResult<()> {
        // Generate verification and image URLs
        bill_info.verification_url = format!("https://bank.linglin.art/verify/{}", bill_info.hash);
        bill_info.image_url = format!("https://bank.linglin.art/bills/{}.png", bill_info.bill_serial);
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO bills \
            (bill_serial, denomination, user_address, hash, mining_time, \
             difficulty, luna_value, timestamp, verification_url, image_url, metadata, status)\
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, COALESCE(?, 'active'))",
            params![
                bill_info.bill_serial,
                bill_info.denomination,
                bill_info.user_address,
                bill_info.hash,
                bill_info.mining_time,
                bill_info.difficulty,
                bill_info.luna_value,
                bill_info.timestamp,
                bill_info.verification_url,
                bill_info.image_url,
                serde_json::to_string(&bill_info.metadata).unwrap_or("{}".to_string()),
                bill_info.status
            ],
        )?;
        Ok(())
    }

    pub fn get_bill(&self, bill_serial: &str) -> SqlResult<Option<BillInfo>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT * FROM bills WHERE bill_serial = ?1")?;
        let mut rows = stmt.query(params![bill_serial])?;
        if let Some(row) = rows.next()? {
            let metadata_str: String = row.get(10)?;
            let metadata: JsonValue = serde_json::from_str(&metadata_str).unwrap_or(JsonValue::Null);
            Ok(Some(BillInfo {
                bill_serial: row.get(0)?,
                denomination: row.get(1)?,
                user_address: row.get(2)?,
                hash: row.get(3)?,
                mining_time: row.get(4)?,
                difficulty: row.get(5)?,
                luna_value: row.get(6)?,
                timestamp: row.get(7)?,
                verification_url: row.get(8)?,
                image_url: row.get(9)?,
                metadata,
                status: row.get(11)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_user_bills(&self, user_address: &str) -> SqlResult<Vec<BillInfo>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT * FROM bills WHERE user_address = ?1 ORDER BY timestamp DESC")?;
        let rows = stmt.query_map(params![user_address], |row| {
            let metadata_str: String = row.get(10)?;
            let metadata: JsonValue = serde_json::from_str(&metadata_str).unwrap_or(JsonValue::Null);
            Ok(BillInfo {
                bill_serial: row.get(0)?,
                denomination: row.get(1)?,
                user_address: row.get(2)?,
                hash: row.get(3)?,
                mining_time: row.get(4)?,
                difficulty: row.get(5)?,
                luna_value: row.get(6)?,
                timestamp: row.get(7)?,
                verification_url: row.get(8)?,
                image_url: row.get(9)?,
                metadata,
                status: row.get(11)?,
            })
        })?;
        let mut bills = Vec::new();
        for bill in rows {
            bills.push(bill?);
        }
        Ok(bills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;

    #[test]
    fn test_bill_registry_crud() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("bills.db");
        let reg = BillRegistry::new(Some(db_path.clone()));

        let bill = BillInfo {
            bill_serial: "B123".to_string(),
            denomination: 100,
            user_address: "user1".to_string(),
            hash: "abc123".to_string(),
            mining_time: 123.4,
            difficulty: 5,
            luna_value: 1.23,
            timestamp: 1234567890.0,
            verification_url: String::new(),
            image_url: String::new(),
            metadata: json!({"foo": "bar"}),
            status: "active".to_string(),
        };
        reg.register_bill(bill.clone()).unwrap();

        let fetched = reg.get_bill("B123").unwrap().unwrap();
        assert_eq!(fetched.bill_serial, "B123");
        assert_eq!(fetched.denomination, 100);
        assert_eq!(fetched.user_address, "user1");
        assert_eq!(fetched.hash, "abc123");
        assert_eq!(fetched.status, "active");
        assert_eq!(fetched.metadata["foo"], "bar");

        let bills = reg.get_user_bills("user1").unwrap();
        assert_eq!(bills.len(), 1);
        assert_eq!(bills[0].bill_serial, "B123");
    }
}
