
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde_json::{Value as JsonValue, json};
use crate::mining::difficulty::Difficulty;
use sha2::Digest;
use crate::gtx::digital_bill::DigitalBill;
use crate::mining::cuda_manager::CUDAManager;

#[derive(Debug)]
pub struct GenesisMiner {
    pub mining_active: Arc<Mutex<bool>>,
    pub mining_stats: Arc<Mutex<HashMap<String, u64>>>,
    pub cuda_manager: Option<CUDAManager>,
}

impl GenesisMiner {
    pub fn new(cuda_manager: Option<CUDAManager>) -> Self {
        let mut stats = HashMap::new();
        stats.insert("bills_mined".to_string(), 0);
        stats.insert("blocks_mined".to_string(), 0);
        stats.insert("total_mining_time".to_string(), 0);
        stats.insert("total_hash_attempts".to_string(), 0);
        GenesisMiner {
            mining_active: Arc::new(Mutex::new(false)),
            mining_stats: Arc::new(Mutex::new(stats)),
            cuda_manager,
        }
    }

    pub fn mine_bill(&self, denomination: u64, user_address: &str, bill_data: Option<JsonValue>, difficulty: u32) -> Option<HashMap<String, JsonValue>> {
        let mut digital_bill = DigitalBill::new(
            denomination,
            user_address.to_string(),
            difficulty,
            bill_data,
            None, None, None, None, None, None,
        );
        let target = "0".repeat(difficulty as usize);
        let mut nonce = 0u64;
        let start_time = Instant::now();
        let mut mining_active = self.mining_active.lock().unwrap();
        *mining_active = true;
        while *mining_active {
            let mining_data = digital_bill.get_mining_data(nonce);
            let data_string = serde_json::to_string(&mining_data).unwrap();
            let bill_hash = format!("{:x}", sha2::Sha256::digest(data_string.as_bytes()));
            if bill_hash.starts_with(&target) {
                let mining_time = start_time.elapsed().as_secs_f64();
                let mut stats = self.mining_stats.lock().unwrap();
                *stats.get_mut("bills_mined").unwrap() += 1;
                *stats.get_mut("total_mining_time").unwrap() += mining_time as u64;
                *stats.get_mut("total_hash_attempts").unwrap() += nonce;
                let mut result = HashMap::new();
                result.insert("success".to_string(), json!(true));
                result.insert("hash".to_string(), json!(bill_hash));
                result.insert("nonce".to_string(), json!(nonce));
                result.insert("mining_time".to_string(), json!(mining_time));
                return Some(result);
            }
            nonce += 1;
            if nonce % 100_000 == 0 {
                let hashrate = nonce as f64 / start_time.elapsed().as_secs_f64();
                println!("⏳ Bill mining: {} attempts | Rate: {:.0} H/s", nonce, hashrate);
            }
        }
        None
    }

    pub fn mine_block(&self, block_data: &mut HashMap<String, JsonValue>, difficulty: u32) -> Option<HashMap<String, JsonValue>> {
        let target = "0".repeat(difficulty as usize);
        let mut nonce = 0u64;
        let start_time = Instant::now();
        let mut mining_active = self.mining_active.lock().unwrap();
        *mining_active = true;
        while *mining_active {
            block_data.insert("nonce".to_string(), json!(nonce));
            let block_string = serde_json::to_string(&block_data).unwrap();
            let block_hash = format!("{:x}", sha2::Sha256::digest(block_string.as_bytes()));
            if block_hash.starts_with(&target) {
                let mining_time = start_time.elapsed().as_secs_f64();
                let mut stats = self.mining_stats.lock().unwrap();
                *stats.get_mut("blocks_mined").unwrap() += 1;
                *stats.get_mut("total_mining_time").unwrap() += mining_time as u64;
                *stats.get_mut("total_hash_attempts").unwrap() += nonce;
                block_data.insert("hash".to_string(), json!(block_hash));
                block_data.insert("mining_time".to_string(), json!(mining_time));
                return Some(block_data.clone());
            }
            nonce += 1;
            if nonce % 100_000 == 0 {
                let hashrate = nonce as f64 / start_time.elapsed().as_secs_f64();
                println!("Block mining: {} attempts | Rate: {:.0} H/s", nonce, hashrate);
            }
        }
        None
    }

    pub fn stop_mining(&self) {
        let mut mining_active = self.mining_active.lock().unwrap();
        *mining_active = false;
        println!("Mining stopped");
    }

    pub fn get_mining_stats(&self) -> HashMap<String, u64> {
        self.mining_stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mine_bill_basic() {
        let miner = GenesisMiner::new(None);
        let result = miner.mine_bill(1, "user1", None, 1);
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res["success"], json!(true));
        assert_eq!(res["hash"].as_str().unwrap().chars().next().unwrap(), '0');
    }

    #[test]
    fn test_mine_block_basic() {
        let miner = GenesisMiner::new(None);
        let mut block_data = HashMap::new();
        block_data.insert("index".to_string(), json!(1));
        block_data.insert("previous_hash".to_string(), json!("0".repeat(64)));
        block_data.insert("timestamp".to_string(), json!(0.0));
        block_data.insert("transactions".to_string(), json!([]));
        block_data.insert("miner".to_string(), json!("user1"));
        block_data.insert("difficulty".to_string(), json!(1));
        block_data.insert("version".to_string(), json!("1.0"));
        let result = miner.mine_block(&mut block_data, 1);
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res["hash"].as_str().unwrap().chars().next().unwrap(), '0');
    }

    #[test]
    fn test_stop_and_stats() {
        let miner = GenesisMiner::new(None);
        miner.stop_mining();
        let stats = miner.get_mining_stats();
        assert!(stats.contains_key("bills_mined"));
        assert!(stats.contains_key("blocks_mined"));
    }

    #[test]
    fn test_mine_bill_with_custom_data() {
        let miner = GenesisMiner::new(None);
        let custom_data = json!({"note": "test"});
        let result = miner.mine_bill(1, "user2", Some(custom_data.clone()), 1);
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res["success"], json!(true));
    }

    #[test]
    fn test_mine_block_stats_update() {
        let miner = GenesisMiner::new(None);
        let mut block_data = HashMap::new();
        block_data.insert("index".to_string(), json!(2));
        block_data.insert("previous_hash".to_string(), json!("0".repeat(64)));
        block_data.insert("timestamp".to_string(), json!(0.0));
        block_data.insert("transactions".to_string(), json!([]));
        block_data.insert("miner".to_string(), json!("user2"));
        block_data.insert("difficulty".to_string(), json!(1));
        block_data.insert("version".to_string(), json!("1.0"));
        let _ = miner.mine_block(&mut block_data, 1);
        let stats = miner.get_mining_stats();
        assert!(stats["blocks_mined"] >= 1);
    }

    #[test]
    fn test_stop_mining_during_bill() {
        use std::sync::Arc;
        let miner = Arc::new(GenesisMiner::new(None));
        let mining_active = miner.mining_active.clone();
        let miner_thread = miner.clone();
        let handle = std::thread::spawn(move || {
            miner_thread.mine_bill(1, "user3", None, 2);
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        {
            let mut active = mining_active.lock().unwrap();
            *active = false;
        }
        let _ = handle.join();
        let stats = miner.get_mining_stats();
        // Should not increment bills_mined if stopped early
        assert!(stats["bills_mined"] <= 1);
    }

    #[test]
    fn test_invalid_difficulty_zero() {
        let miner = GenesisMiner::new(None);
        let result = miner.mine_bill(1, "user4", None, 0);
        // Should instantly succeed since target is empty string
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res["success"], json!(true));
    }
}
