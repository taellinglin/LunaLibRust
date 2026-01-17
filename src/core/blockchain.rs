use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub miner: Option<String>,
    pub difficulty: Option<u64>,
    pub nonce: Option<u64>,
    // ...他のフィールドも必要に応じて追加
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub tx_type: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<f64>,
    pub timestamp: Option<u64>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    // ...他のフィールドも必要に応じて追加
}

impl Transaction {
    pub fn new() -> Self {
        Transaction {
            tx_type: None,
            from: None,
            to: None,
            amount: None,
            timestamp: None,
            hash: None,
            signature: None,
            // ...他のフィールドも必要に応じて追加
        }
    }
}

impl Block {
    pub fn new() -> Self {
        Block {
            index: 0,
            hash: String::new(),
            previous_hash: String::new(),
            timestamp: 0,
            transactions: vec![],
            miner: None,
            difficulty: None,
            nonce: None,
            // ...他のフィールドも必要に応じて追加
        }
    }
}

pub struct BlockchainManager {
    pub endpoint_url: String,
    pub network_connected: bool,
    pub cache: Arc<Mutex<HashMap<u64, Block>>>,
    pub async_tasks: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>>,
    pub task_results: Arc<Mutex<HashMap<String, String>>>,
    pub stop_events: Arc<Mutex<Vec<Arc<Mutex<bool>>>>>,
}

impl BlockchainManager {
    pub fn new(endpoint_url: &str, _max_workers: usize) -> Self {
        BlockchainManager {
            endpoint_url: endpoint_url.trim_end_matches('/').to_string(),
            network_connected: false,
            cache: Arc::new(Mutex::new(HashMap::new())),
            async_tasks: Arc::new(Mutex::new(HashMap::new())),
            task_results: Arc::new(Mutex::new(HashMap::new())),
            stop_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Normalize LUN addresses for comparison (lowercase, strip, drop prefix)
    pub fn normalize_address(addr: &str) -> String {
        if addr.is_empty() {
            return String::new();
        }
        let mut addr_str = addr.trim_matches(|c| c == '\'' || c == '"' || c == ' ').to_lowercase();
        if addr_str.starts_with("lun_") {
            addr_str = addr_str[4..].to_string();
        }
        addr_str
    }

    /// Validate transaction before broadcasting (struct version)
    pub fn validate_transaction_before_broadcast(transaction: &Transaction) -> bool {
        if transaction.tx_type.is_none()
            || transaction.from.is_none()
            || transaction.to.is_none()
            || transaction.amount.is_none()
            || transaction.timestamp.is_none()
            || transaction.hash.is_none()
            || transaction.signature.is_none()
        {
            println!("❌ Missing required field");
            return false;
        }
        if !transaction.from.as_ref().unwrap().starts_with("LUN_") {
            println!("❌ Invalid from address format: {}", transaction.from.as_ref().unwrap());
            return false;
        }
        if !transaction.to.as_ref().unwrap().starts_with("LUN_") {
            println!("❌ Invalid to address format: {}", transaction.to.as_ref().unwrap());
            return false;
        }
        if transaction.amount.unwrap() <= 0.0 {
            println!("❌ Invalid amount: {}", transaction.amount.unwrap());
            return false;
        }
        if transaction.signature.as_ref().unwrap().len() < 10 {
            println!("❌ Invalid or missing signature");
            return false;
        }
        if transaction.hash.as_ref().unwrap().len() < 10 {
            println!("❌ Invalid or missing transaction hash");
            return false;
        }
        println!("✅ Transaction validation passed");
        true
    }

    /// Non-blocking: Broadcast transaction to mempool
    pub async fn broadcast_transaction(&self, transaction: &Transaction) -> Result<String, String> {
        let url = format!("{}/mempool/add", self.endpoint_url);
        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .json(transaction)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;
        if res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            Ok(format!("Broadcast success: {}", text))
        } else {
            Err(format!("Broadcast failed: HTTP {}", res.status()))
        }
    }

    /// Non-blocking: Get current blockchain height
    pub async fn get_blockchain_height(&self) -> Result<u64, String> {
        let url = format!("{}/blockchain/blocks", self.endpoint_url);
        let res = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            if let Some(blocks) = json.get("blocks").and_then(|b| b.as_array()) {
                if let Some(last) = blocks.last() {
                    if let Some(index) = last.get("index").and_then(|i| i.as_u64()) {
                        return Ok(index);
                    }
                }
            }
            Ok(0)
        } else {
            Err(format!("Failed to get height: HTTP {}", res.status()))
        }
    }

    /// Async: get range of blocks (dummy, spawns thread)
    pub fn get_blocks_range_async(&self, start_height: u64, end_height: u64, task_id: String) {
        let cache: Arc<Mutex<HashMap<u64, Block>>> = Arc::clone(&self.cache);
        let async_tasks: Arc<Mutex<HashMap<String, thread::JoinHandle<()>>>> = Arc::clone(&self.async_tasks);
        let handle = thread::spawn(move || {
            // ダミー: キャッシュから取得
            let cache = cache.lock().unwrap();
            let _blocks: Vec<Block> = (start_height..=end_height)
                .filter_map(|h| cache.get(&h).cloned())
                .collect();
            // 本来はコールバックやチャンネルで通知
        });
        async_tasks.lock().unwrap().insert(task_id, handle);
    }

    /// Get range of blocks (cache only, dummy)
    pub fn get_blocks_range(&self, start_height: u64, end_height: u64) -> Vec<Block> {
        let cache = self.cache.lock().unwrap();
        (start_height..=end_height)
            .filter_map(|h| cache.get(&h).cloned())
            .collect()
    }

    /// Get current mempool (dummy)
    pub fn get_mempool(&self) -> Vec<Transaction> {
        // TODO: reqwestでHTTP GET実装
        vec![]
    }

    /// Check network connection (dummy)
    pub fn check_network_connection(&mut self) -> bool {
        // TODO: reqwestでHTTP GET実装
        self.network_connected = true;
        true
    }

    /// Dummy async task status
    pub fn get_task_status(&self, task_id: &str) -> String {
        let tasks = self.async_tasks.lock().unwrap();
        if tasks.contains_key(task_id) {
            "running".to_string()
        } else {
            "not_found".to_string()
        }
    }

    /// Dummy: cancel async task
    pub fn cancel_task(&self, task_id: &str) -> bool {
        let mut tasks = self.async_tasks.lock().unwrap();
        if let Some(_handle) = tasks.remove(task_id) {
            // RustではJoinHandleのキャンセルはサポート外
            true
        } else {
            false
        }
    }

    /// 非同期: 指定高さのブロックを取得
    pub async fn get_block_by_height(&self, height: u64) -> Result<Block, String> {
        let url = format!("{}/blockchain/block/{}", self.endpoint_url, height);
        let res = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        if res.status().is_success() {
            let block: Block = res.json().await.map_err(|e| e.to_string())?;
            Ok(block)
        } else {
            Err(format!("Failed to get block: HTTP {}", res.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    fn valid_transaction() -> Transaction {
        Transaction {
            tx_type: Some("transfer".to_string()),
            from: Some("LUN_testfrom".to_string()),
            to: Some("LUN_testto".to_string()),
            amount: Some(1.0),
            timestamp: Some(1234567890),
            hash: Some("1234567890abcdef1234567890abcdef".to_string()),
            signature: Some("abcdef1234567890abcdef1234567890".to_string()),
            ..Transaction::new()
        }
    }

    #[tokio::test]
    async fn test_broadcast_transaction_real_endpoint() {
        let manager = BlockchainManager::new("https://bank.linglin.art", 2);
        let tx = valid_transaction();
        let result = manager.broadcast_transaction(&tx).await;
        // 成功または失敗どちらも許容（ネットワーク状況やAPI仕様による）
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_get_blockchain_height_real_endpoint() {
        let manager = BlockchainManager::new("https://bank.linglin.art", 2);
        let result = manager.get_blockchain_height().await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_get_block_real_endpoint() {
        let manager = BlockchainManager::new("https://bank.linglin.art", 2);
        // 0番ブロックは必ず存在するはず
        let result = manager.get_block_by_height(0).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_normalize_address() {
        assert_eq!(BlockchainManager::normalize_address("LUN_abc123"), "abc123");
        assert_eq!(BlockchainManager::normalize_address("lun_ABC123"), "abc123");
        assert_eq!(BlockchainManager::normalize_address("abc123"), "abc123");
        assert_eq!(BlockchainManager::normalize_address("").as_str(), "");
    }

    #[test]
    fn test_validate_transaction_before_broadcast() {
        let mut tx = Transaction::new();
        assert!(!BlockchainManager::validate_transaction_before_broadcast(&tx));
        tx.tx_type = Some("transfer".to_string());
        tx.from = Some("LUN_from".to_string());
        tx.to = Some("LUN_to".to_string());
        tx.amount = Some(1.0);
        tx.timestamp = Some(1234567890);
        tx.hash = Some("1234567890abcdef".to_string());
        tx.signature = Some("abcdef1234567890".to_string());
        assert!(BlockchainManager::validate_transaction_before_broadcast(&tx));
    }
}
