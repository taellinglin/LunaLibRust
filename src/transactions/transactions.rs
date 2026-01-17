
use std::collections::HashMap;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

#[derive(Debug, Default)]
pub struct TransactionSecurity;

impl TransactionSecurity {
    pub fn validate_transaction(&self, transaction: &HashMap<String, Value>) -> (bool, String) {
        let required_fields = ["type", "from", "to", "amount", "timestamp", "hash"];
        for field in &required_fields {
            if !transaction.contains_key(*field) {
                return (false, format!("Missing required field: {}", field));
            }
        }
        let amount = transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let tx_type = transaction.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if amount <= 0.0 && tx_type != "reward" {
            return (false, "Amount must be positive".to_string());
        }
        (true, "Valid".to_string())
    }

    pub fn assess_risk(&self, transaction: &HashMap<String, Value>) -> (String, String) {
        let amount = transaction.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let tx_type = transaction.get("type").and_then(|v| v.as_str()).unwrap_or("transfer");
        if ["gtx_genesis", "reward"].contains(&tx_type) {
            return ("very_low".to_string(), "System transaction".to_string());
        }
        if amount > 1_000_000.0 {
            ("high".to_string(), "Very large transaction".to_string())
        } else if amount > 100_000.0 {
            ("medium".to_string(), "Large transaction".to_string())
        } else {
            ("low".to_string(), "Normal transaction".to_string())
        }
    }
}

#[derive(Debug, Default)]
pub struct FeeCalculator {
    pub fee_config: HashMap<String, f64>,
}

impl FeeCalculator {
    pub fn new() -> Self {
        let mut fee_config = HashMap::new();
        fee_config.insert("transfer".to_string(), 0.001);
        fee_config.insert("reward".to_string(), 0.0);
        fee_config.insert("gtx_genesis".to_string(), 0.0);
        FeeCalculator { fee_config }
    }
    pub fn get_fee(&self, transaction_type: &str) -> f64 {
        *self.fee_config.get(transaction_type).unwrap_or(&0.001)
    }
}

#[derive(Debug)]
pub struct TransactionManager {
    pub security: TransactionSecurity,
    pub fee_calculator: FeeCalculator,
}

impl TransactionManager {
    pub fn new() -> Self {
        TransactionManager {
            security: TransactionSecurity,
            fee_calculator: FeeCalculator::new(),
        }
    }

    pub fn create_transaction(
        &self,
        from_address: &str,
        to_address: &str,
        amount: f64,
        memo: &str,
        transaction_type: &str,
    ) -> HashMap<String, Value> {
        let fee = self.fee_calculator.get_fee(transaction_type);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut tx = HashMap::new();
        tx.insert("type".to_string(), Value::String(transaction_type.to_string()));
        tx.insert("from".to_string(), Value::String(from_address.to_string()));
        tx.insert("to".to_string(), Value::String(to_address.to_string()));
        tx.insert("amount".to_string(), Value::from(amount));
        tx.insert("fee".to_string(), Value::from(fee));
        tx.insert("timestamp".to_string(), Value::from(timestamp));
        tx.insert("memo".to_string(), Value::String(memo.to_string()));
        tx.insert("version".to_string(), Value::String("2.0".to_string()));
        // 署名・公開鍵は未実装
        tx.insert("signature".to_string(), Value::String("unsigned".to_string()));
        tx.insert("public_key".to_string(), Value::String("unsigned".to_string()));
        tx.insert("hash".to_string(), Value::String(Self::calculate_transaction_hash(&tx)));
        tx
    }

    pub fn create_gtx_transaction(&self, bill_info: &HashMap<String, Value>) -> HashMap<String, Value> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut tx = HashMap::new();
        tx.insert("type".to_string(), Value::String("gtx_genesis".to_string()));
        tx.insert("from".to_string(), Value::String("mining".to_string()));
        tx.insert("to".to_string(), bill_info.get("owner_address").cloned().unwrap_or(Value::String("unknown".to_string())));
        tx.insert("amount".to_string(), bill_info.get("denomination").cloned().unwrap_or(Value::from(0.0)));
        tx.insert("fee".to_string(), Value::from(0.0));
        tx.insert("timestamp".to_string(), Value::from(timestamp));
        tx.insert("bill_serial".to_string(), bill_info.get("serial").cloned().unwrap_or(Value::String("".to_string())));
        tx.insert("mining_difficulty".to_string(), bill_info.get("difficulty").cloned().unwrap_or(Value::from(0)));
        tx.insert("signature".to_string(), Value::String("system".to_string()));
        tx.insert("public_key".to_string(), Value::String("system".to_string()));
        tx.insert("version".to_string(), Value::String("2.0".to_string()));
        tx.insert("hash".to_string(), Value::String(Self::calculate_transaction_hash(&tx)));
        tx
    }

    pub fn create_reward_transaction(&self, to_address: &str, amount: f64, block_height: i64) -> HashMap<String, Value> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut tx = HashMap::new();
        tx.insert("type".to_string(), Value::String("reward".to_string()));
        tx.insert("from".to_string(), Value::String("network".to_string()));
        tx.insert("to".to_string(), Value::String(to_address.to_string()));
        tx.insert("amount".to_string(), Value::from(amount));
        tx.insert("fee".to_string(), Value::from(0.0));
        tx.insert("block_height".to_string(), Value::from(block_height));
        tx.insert("timestamp".to_string(), Value::from(timestamp));
        tx.insert("signature".to_string(), Value::String("system".to_string()));
        tx.insert("public_key".to_string(), Value::String("system".to_string()));
        tx.insert("version".to_string(), Value::String("2.0".to_string()));
        tx.insert("hash".to_string(), Value::String(Self::generate_reward_hash(to_address, amount, block_height)));
        tx
    }

    pub fn calculate_transaction_hash(tx: &HashMap<String, Value>) -> String {
        let mut tx_copy = tx.clone();
        tx_copy.remove("hash");
        let json = serde_json::to_string(&tx_copy).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_reward_hash(to_address: &str, amount: f64, block_height: i64) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let data = format!("reward_{}_{}_{}_{}", to_address, amount, block_height, now);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transfer() {
        let mgr = TransactionManager::new();
        let tx = mgr.create_transaction("alice", "bob", 123.45, "memo", "transfer");
        assert_eq!(tx.get("type").unwrap().as_str().unwrap(), "transfer");
        assert_eq!(tx.get("from").unwrap().as_str().unwrap(), "alice");
        assert_eq!(tx.get("to").unwrap().as_str().unwrap(), "bob");
        assert_eq!(tx.get("amount").unwrap().as_f64().unwrap(), 123.45);
        assert_eq!(tx.get("fee").unwrap().as_f64().unwrap(), 0.001);
        assert_eq!(tx.get("signature").unwrap().as_str().unwrap(), "unsigned");
        assert!(tx.get("hash").unwrap().as_str().unwrap().len() == 64);
    }

    #[test]
    fn test_create_gtx_transaction() {
        let mgr = TransactionManager::new();
        let mut bill_info = HashMap::new();
        bill_info.insert("owner_address".to_string(), Value::String("miner1".to_string()));
        bill_info.insert("denomination".to_string(), Value::from(100.0));
        bill_info.insert("serial".to_string(), Value::String("S123".to_string()));
        bill_info.insert("difficulty".to_string(), Value::from(2));
        let tx = mgr.create_gtx_transaction(&bill_info);
        assert_eq!(tx.get("type").unwrap().as_str().unwrap(), "gtx_genesis");
        assert_eq!(tx.get("from").unwrap().as_str().unwrap(), "mining");
        assert_eq!(tx.get("to").unwrap().as_str().unwrap(), "miner1");
        assert_eq!(tx.get("amount").unwrap().as_f64().unwrap(), 100.0);
        assert_eq!(tx.get("signature").unwrap().as_str().unwrap(), "system");
        assert!(tx.get("hash").unwrap().as_str().unwrap().len() == 64);
    }

    #[test]
    fn test_create_reward_transaction() {
        let mgr = TransactionManager::new();
        let tx = mgr.create_reward_transaction("bob", 50.0, 42);
        assert_eq!(tx.get("type").unwrap().as_str().unwrap(), "reward");
        assert_eq!(tx.get("from").unwrap().as_str().unwrap(), "network");
        assert_eq!(tx.get("to").unwrap().as_str().unwrap(), "bob");
        assert_eq!(tx.get("amount").unwrap().as_f64().unwrap(), 50.0);
        assert_eq!(tx.get("block_height").unwrap().as_i64().unwrap(), 42);
        assert_eq!(tx.get("signature").unwrap().as_str().unwrap(), "system");
        assert!(tx.get("hash").unwrap().as_str().unwrap().len() == 64);
    }

    #[test]
    fn test_validate_transaction() {
        let mgr = TransactionManager::new();
        let tx = mgr.create_transaction("alice", "bob", 1.0, "memo", "transfer");
        let (ok, msg) = mgr.security.validate_transaction(&tx);
        assert!(ok, "{}", msg);
    }

    #[test]
    fn test_assess_risk() {
        let mgr = TransactionManager::new();
        let tx = mgr.create_transaction("alice", "bob", 1_000_001.0, "memo", "transfer");
        let (level, reason) = mgr.security.assess_risk(&tx);
        assert_eq!(level, "high");
        assert_eq!(reason, "Very large transaction");
    }
}
