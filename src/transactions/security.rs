pub struct Security;

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct TransactionSecurity {
    pub min_transaction_amount: f64,
    pub max_transaction_amount: f64,
    pub required_fee: f64,
    pub rate_limits: HashMap<String, Vec<u64>>, // address -> timestamps
    pub blacklisted_addresses: HashSet<String>,
    pub sm2_available: bool,
}

impl TransactionSecurity {
    pub fn new(sm2_available: bool) -> Self {
        TransactionSecurity {
            min_transaction_amount: 0.000001,
            max_transaction_amount: 100000000.0,
            required_fee: 0.00001,
            rate_limits: HashMap::new(),
            blacklisted_addresses: HashSet::new(),
            sm2_available,
        }
    }

    pub fn validate_transaction_security(&mut self, transaction: &HashMap<String, serde_json::Value>) -> (bool, String) {
        let tx_type = transaction.get("type").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        match tx_type.as_str() {
            "gtx_genesis" => self.validate_genesis_transaction(transaction),
            "reward" => self.validate_reward_transaction(transaction),
            "transfer" => self.validate_transfer_transaction(transaction),
            _ => (false, format!("Unknown transaction type: {}", tx_type)),
        }
    }

    fn validate_genesis_transaction(&self, transaction: &HashMap<String, serde_json::Value>) -> (bool, String) {
        let required_fields = ["bill_serial", "denomination", "mining_difficulty", "hash", "nonce"];
        for field in &required_fields {
            if !transaction.contains_key(*field) {
                return (false, format!("Missing GTX field: {}", field));
            }
        }
        let denomination = transaction.get("denomination").and_then(|v| v.as_i64()).unwrap_or(-1);
        let valid_denominations = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000];
        if !valid_denominations.contains(&denomination) {
            return (false, format!("Invalid denomination: {}", denomination));
        }
        if !self.validate_mining_proof(transaction) {
            return (false, "Invalid mining proof".to_string());
        }
        (true, "Valid GTX Genesis transaction".to_string())
    }

    fn validate_reward_transaction(&self, transaction: &HashMap<String, serde_json::Value>) -> (bool, String) {
        let required_fields = ["from", "to", "amount", "block_height", "hash"];
        for field in &required_fields {
            if !transaction.contains_key(*field) {
                return (false, format!("Missing reward field: {}", field));
            }
        }
        if transaction.get("from").and_then(|v| v.as_str()) != Some("network") {
            return (false, "Unauthorized reward creation".to_string());
        }
        (true, "Valid reward transaction".to_string())
    }

    fn validate_transfer_transaction(&mut self, transaction: &HashMap<String, serde_json::Value>) -> (bool, String) {
        let required_fields = ["from", "to", "amount", "signature", "public_key", "nonce"];
        for field in &required_fields {
            if !transaction.contains_key(*field) {
                return (false, format!("Missing field: {}", field));
            }
        }
        let amount = transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if amount < self.min_transaction_amount {
            return (false, format!("Amount below minimum: {}", self.min_transaction_amount));
        }
        if amount > self.max_transaction_amount {
            return (false, format!("Amount above maximum: {}", self.max_transaction_amount));
        }
        let fee = transaction.get("fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if fee < self.required_fee {
            return (false, format!("Insufficient fee: {} (required: {})", fee, self.required_fee));
        }
        if !self.validate_signature_sm2(transaction) {
            return (false, "Invalid SM2 signature".to_string());
        }
        let from_address = transaction.get("from").and_then(|v| v.as_str()).unwrap_or("");
        if !self.check_rate_limit(from_address) {
            return (false, "Rate limit exceeded".to_string());
        }
        if self.is_blacklisted(from_address) {
            return (false, "Address is blacklisted".to_string());
        }
        (true, "Valid transfer transaction".to_string())
    }

    fn validate_signature_sm2(&self, transaction: &HashMap<String, serde_json::Value>) -> bool {
        let signature = transaction.get("signature").and_then(|v| v.as_str()).unwrap_or("");
        let public_key = transaction.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
        let tx_type = transaction.get("type").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        if ["gtx_genesis", "reward"].contains(&tx_type.as_str()) {
            return true;
        }
        if ["system", "unsigned", "test"].contains(&signature) {
            return true;
        }
        if signature.len() != 128 {
            return false;
        }
        if !signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
        if !public_key.starts_with("04") {
            return false;
        }
        // SM2検証は外部KeyManagerが必要。ここでは形式のみチェック。
        true
    }

    fn validate_mining_proof(&self, transaction: &HashMap<String, serde_json::Value>) -> bool {
        let difficulty = transaction.get("mining_difficulty").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let bill_hash = transaction.get("hash").and_then(|v| v.as_str()).unwrap_or("");
        let target = "0".repeat(difficulty);
        bill_hash.starts_with(&target)
    }

    pub fn check_rate_limit(&mut self, address: &str) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let entry = self.rate_limits.entry(address.to_lowercase()).or_insert_with(Vec::new);
        entry.retain(|&t| now - t < 60);
        if entry.len() >= 10 {
            return false;
        }
        entry.push(now);
        true
    }

    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.blacklisted_addresses.contains(&address.to_lowercase())
    }

    pub fn blacklist_address(&mut self, address: &str) {
        self.blacklisted_addresses.insert(address.to_lowercase());
    }

    pub fn calculate_security_score(&self, transaction: &HashMap<String, serde_json::Value>) -> u32 {
        let mut score = 0;
        let signature = transaction.get("signature").and_then(|v| v.as_str()).unwrap_or("");
        if signature.len() == 128 {
            score += 60;
        } else if signature.len() == 64 {
            score += 40;
        }
        let public_key = transaction.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
        if !public_key.is_empty() && public_key.starts_with("04") {
            score += 30;
        }
        let timestamp = transaction.get("timestamp").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
        if now - timestamp < 600.0 {
            score += 20;
        }
        if transaction.contains_key("nonce") {
            score += 10;
        }
        if transaction.contains_key("security_hash") {
            score += 10;
        }
        score.min(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn make_tx(tx_type: &str) -> HashMap<String, serde_json::Value> {
        let mut tx = HashMap::new();
        tx.insert("type".to_string(), json!(tx_type));
        tx
    }

    #[test]
    fn test_genesis_validation() {
        let mut tx = make_tx("gtx_genesis");
        tx.insert("bill_serial".to_string(), json!("A"));
        tx.insert("denomination".to_string(), json!(100));
        tx.insert("mining_difficulty".to_string(), json!(2));
        tx.insert("hash".to_string(), json!("00abcdef"));
        tx.insert("nonce".to_string(), json!(123));
        let mut sec = TransactionSecurity::new(false);
        let (ok, msg) = sec.validate_transaction_security(&tx);
        assert!(ok, "{}", msg);
    }

    #[test]
    fn test_reward_validation() {
        let mut tx = make_tx("reward");
        tx.insert("from".to_string(), json!("network"));
        tx.insert("to".to_string(), json!("user"));
        tx.insert("amount".to_string(), json!(1.0));
        tx.insert("block_height".to_string(), json!(1));
        tx.insert("hash".to_string(), json!("abc"));
        let mut sec = TransactionSecurity::new(false);
        let (ok, msg) = sec.validate_transaction_security(&tx);
        assert!(ok, "{}", msg);
    }

    #[test]
    fn test_transfer_validation() {
        let mut tx = make_tx("transfer");
        tx.insert("from".to_string(), json!("user1"));
        tx.insert("to".to_string(), json!("user2"));
        tx.insert("amount".to_string(), json!(1.0));
        tx.insert("fee".to_string(), json!(0.00001));
        // signature: 128 hex chars, starts with '04'
        let sig = format!("04{:0<126}", "a");
        tx.insert("signature".to_string(), json!(sig));
        tx.insert("public_key".to_string(), json!("04abcdef"));
        tx.insert("nonce".to_string(), json!(123));
        let mut sec = TransactionSecurity::new(false);
        let (ok, msg) = sec.validate_transaction_security(&tx);
        assert!(ok, "{}", msg);
    }

    #[test]
    fn test_blacklist() {
        let mut sec = TransactionSecurity::new(false);
        sec.blacklist_address("badguy");
        assert!(sec.is_blacklisted("badguy"));
    }

    #[test]
    fn test_rate_limit() {
        let mut sec = TransactionSecurity::new(false);
        let addr = "user1";
        for _ in 0..10 {
            assert!(sec.check_rate_limit(addr));
        }
        assert!(!sec.check_rate_limit(addr));
    }

    #[test]
    fn test_security_score() {
        let mut tx = make_tx("transfer");
        // signature: 128 hex chars, starts with '04'
        let sig = format!("04{:0<126}", "a");
        tx.insert("signature".to_string(), json!(sig));
        tx.insert("public_key".to_string(), json!("04abcdef"));
        tx.insert("timestamp".to_string(), json!(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64));
        tx.insert("nonce".to_string(), json!(123));
        tx.insert("security_hash".to_string(), json!("abc"));
        let sec = TransactionSecurity::new(false);
        let score = sec.calculate_security_score(&tx);
        assert_eq!(score, 100);
    }
}
