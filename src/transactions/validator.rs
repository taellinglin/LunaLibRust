pub struct Validator;

use std::collections::HashSet;
use std::collections::HashMap;
use serde_json::Value;
use crate::transactions::security::TransactionSecurity;

#[derive(Debug)]
pub struct TransactionValidator {
    pub security: TransactionSecurity,
    pub recent_transactions: HashSet<String>,
    pub max_recent_size: usize,
}

impl TransactionValidator {
    pub fn new() -> Self {
        TransactionValidator {
            security: TransactionSecurity::new(false),
            recent_transactions: HashSet::new(),
            max_recent_size: 10_000,
        }
    }

    pub fn validate_transaction(&mut self, transaction: &HashMap<String, Value>) -> (bool, String) {
        let tx_hash = transaction.get("hash").and_then(|v| v.as_str()).unwrap_or("");
        if self.recent_transactions.contains(tx_hash) {
            return (false, "Duplicate transaction detected".to_string());
        }
        let (is_valid, message) = self.security.validate_transaction_security(transaction);
        if !is_valid {
            return (false, message);
        }
        self.add_to_recent(tx_hash);
        (true, message)
    }

    pub fn validate_transaction_batch(&mut self, transactions: &[HashMap<String, Value>]) -> (bool, Vec<String>) {
        let mut results = Vec::new();
        let mut all_valid = true;
        for tx in transactions {
            let (is_valid, message) = self.validate_transaction(tx);
            results.push(message.clone());
            if !is_valid {
                all_valid = false;
            }
        }
        (all_valid, results)
    }

    pub fn verify_transaction_inclusion(&self, transaction_hash: &str, _block_height: i64) -> bool {
        self.recent_transactions.contains(transaction_hash)
    }

    pub fn get_transaction_risk_level(&self, transaction: &HashMap<String, Value>) -> String {
        let amount = transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let security_score = self.security.calculate_security_score(transaction);
        if amount > 1_000_000.0 && security_score < 80 {
            "HIGH".to_string()
        } else if amount > 10_000.0 && security_score < 60 {
            "MEDIUM".to_string()
        } else if security_score < 40 {
            "LOW".to_string()
        } else {
            "VERY_LOW".to_string()
        }
    }

    fn add_to_recent(&mut self, tx_hash: &str) {
        self.recent_transactions.insert(tx_hash.to_string());
        if self.recent_transactions.len() > self.max_recent_size {
            let keep: HashSet<_> = self.recent_transactions.iter().cloned().take(self.max_recent_size).collect();
            self.recent_transactions = keep;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::security::TransactionSecurity;
    use serde_json::json;

    fn make_tx(hash: &str, amount: f64) -> HashMap<String, Value> {
        let mut tx = HashMap::new();
        tx.insert("type".to_string(), json!("transfer"));
        tx.insert("from".to_string(), json!("alice"));
        tx.insert("to".to_string(), json!("bob"));
        tx.insert("amount".to_string(), json!(amount));
        tx.insert("fee".to_string(), json!(0.001));
        tx.insert("timestamp".to_string(), json!(1234567890));
        // signature: 128 hex chars, starts with '04'
        let sig = format!("04{:0<126}", "a");
        tx.insert("signature".to_string(), json!(sig));
        tx.insert("public_key".to_string(), json!("04abcdef"));
        tx.insert("nonce".to_string(), json!(123));
        tx.insert("hash".to_string(), json!(hash));
        tx
    }

    #[test]
    fn test_duplicate_detection() {
        let mut validator = TransactionValidator::new();
        let tx1 = make_tx("h1", 10.0);
        let tx2 = make_tx("h1", 20.0);
        let (ok1, msg1) = validator.validate_transaction(&tx1);
        assert!(ok1, "{}", msg1);
        let (ok2, msg2) = validator.validate_transaction(&tx2);
        assert!(!ok2, "{}", msg2);
        assert_eq!(msg2, "Duplicate transaction detected");
    }

    #[test]
    fn test_batch_validation() {
        let mut validator = TransactionValidator::new();
        let txs = vec![make_tx("h2", 10.0), make_tx("h3", 20.0)];
        let (all_valid, results) = validator.validate_transaction_batch(&txs);
        assert!(all_valid, "Batch validation failed: {:?}", results);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_risk_level() {
        let mut validator = TransactionValidator::new();
        let tx = make_tx("h4", 2_000_000.0);
        // Lower security score by removing nonce/security_hash and weakening signature/public_key
        let mut tx_low_score = tx.clone();
        tx_low_score.remove("nonce");
        tx_low_score.remove("security_hash");
        tx_low_score.insert("signature".to_string(), serde_json::json!("bad_sig")); // <64 chars
        tx_low_score.insert("public_key".to_string(), serde_json::json!("abcdef")); // not start with '04'
        let level = validator.get_transaction_risk_level(&tx_low_score);
        assert_eq!(level, "HIGH");
        // With all security fields, should be VERY_LOW
        let level2 = validator.get_transaction_risk_level(&tx);
        assert_eq!(level2, "VERY_LOW");
    }

    #[test]
    fn test_inclusion() {
        let mut validator = TransactionValidator::new();
        let tx = make_tx("h5", 10.0);
        let (ok, msg) = validator.validate_transaction(&tx);
        assert!(ok, "{}", msg);
        assert!(validator.verify_transaction_inclusion("h5", 0));
    }
}
