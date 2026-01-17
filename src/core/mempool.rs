
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MempoolManager {
    pub local_mempool: Arc<Mutex<HashMap<String, Transaction>>>,
    pub confirmed_transactions: Arc<Mutex<HashSet<String>>>,
    pub max_mempool_size: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub tx_type: String,
}

impl MempoolManager {
    pub fn new() -> Self {
        MempoolManager {
            local_mempool: Arc::new(Mutex::new(HashMap::new())),
            confirmed_transactions: Arc::new(Mutex::new(HashSet::new())),
            max_mempool_size: 10000,
        }
    }

    pub fn add_transaction(&self, tx: Transaction) -> bool {
        let mut mempool = self.local_mempool.lock().unwrap();
        let confirmed = self.confirmed_transactions.lock().unwrap();
        if mempool.contains_key(&tx.hash) || confirmed.contains(&tx.hash) {
            return false;
        }
        if !self.validate_transaction_basic(&tx) {
            return false;
        }
        if mempool.len() >= self.max_mempool_size {
            return false;
        }
        mempool.insert(tx.hash.clone(), tx);
        true
    }

    pub fn remove_transaction(&self, tx_hash: &str) {
        let mut mempool = self.local_mempool.lock().unwrap();
        let mut confirmed = self.confirmed_transactions.lock().unwrap();
        mempool.remove(tx_hash);
        confirmed.insert(tx_hash.to_string());
    }

    pub fn get_transaction(&self, tx_hash: &str) -> Option<Transaction> {
        let mempool = self.local_mempool.lock().unwrap();
        mempool.get(tx_hash).cloned()
    }

    pub fn get_pending_transactions(&self) -> Vec<Transaction> {
        let mempool = self.local_mempool.lock().unwrap();
        mempool.values().cloned().collect()
    }

    pub fn is_transaction_pending(&self, tx_hash: &str) -> bool {
        let mempool = self.local_mempool.lock().unwrap();
        mempool.contains_key(tx_hash)
    }

    pub fn is_transaction_confirmed(&self, tx_hash: &str) -> bool {
        let confirmed = self.confirmed_transactions.lock().unwrap();
        confirmed.contains(tx_hash)
    }

    pub fn get_mempool_size(&self) -> usize {
        let mempool = self.local_mempool.lock().unwrap();
        mempool.len()
    }

    pub fn clear_mempool(&self) {
        let mut mempool = self.local_mempool.lock().unwrap();
        mempool.clear();
    }

    pub fn validate_transaction_basic(&self, tx: &Transaction) -> bool {
        if tx.hash.is_empty() || tx.from.is_empty() || tx.to.is_empty() || tx.amount <= 0.0 || tx.timestamp == 0 || tx.tx_type.is_empty() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_tx(hash: &str) -> Transaction {
        Transaction {
            hash: hash.to_string(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 1.0,
            timestamp: 123456,
            tx_type: "transaction".to_string(),
        }
    }
    #[test]
    fn test_add_and_get_transaction() {
        let mempool = MempoolManager::new();
        let tx = sample_tx("tx1");
        assert!(mempool.add_transaction(tx.clone()));
        assert_eq!(mempool.get_transaction("tx1"), Some(tx.clone()));
        assert!(!mempool.add_transaction(tx.clone())); // duplicate
    }
    #[test]
    fn test_remove_and_confirmed() {
        let mempool = MempoolManager::new();
        let tx = sample_tx("tx2");
        mempool.add_transaction(tx.clone());
        mempool.remove_transaction("tx2");
        assert!(!mempool.is_transaction_pending("tx2"));
        assert!(mempool.is_transaction_confirmed("tx2"));
    }
    #[test]
    fn test_get_pending_transactions() {
        let mempool = MempoolManager::new();
        mempool.add_transaction(sample_tx("tx3"));
        mempool.add_transaction(sample_tx("tx4"));
        let txs = mempool.get_pending_transactions();
        assert_eq!(txs.len(), 2);
    }
    #[test]
    fn test_clear_mempool() {
        let mempool = MempoolManager::new();
        mempool.add_transaction(sample_tx("tx5"));
        mempool.clear_mempool();
        assert_eq!(mempool.get_mempool_size(), 0);
    }
}
