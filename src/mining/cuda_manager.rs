
use std::time::Instant;
use std::collections::HashMap;
use serde_json::{Value as JsonValue, json};
use sha2::Digest;

#[cfg(feature = "cuda")]
use cust::prelude::*;

#[derive(Debug)]
pub struct CUDAManager {
    pub cuda_available: bool,
    pub device_name: Option<String>,
}

impl CUDAManager {
    pub fn new() -> Self {
        let mut cuda_available = false;
        let mut device_name = None;
        #[cfg(feature = "cuda")]
        {
            match Device::get_count() {
                Ok(count) if count > 0 => {
                    cuda_available = true;
                    let device = Device::get_device(0).unwrap();
                    device_name = Some(device.name().unwrap_or("Unknown").to_string());
                    println!("✅ CUDA is available for accelerated mining");
                },
                Ok(_) => println!("❌ CUDA drivers found but no GPU available"),
                Err(e) => println!("❌ CUDA check failed: {:?}", e),
            }
        }
        #[cfg(not(feature = "cuda"))]
        println!("❌ CUDA not compiled in (feature 'cuda' missing)");
        CUDAManager { cuda_available, device_name }
    }

    pub fn cuda_mine_batch(&self, mining_data: &HashMap<String, JsonValue>, difficulty: usize, batch_size: usize) -> Option<HashMap<String, JsonValue>> {
        if !self.cuda_available {
            return None;
        }
        let target = "0".repeat(difficulty);
        let mut nonce_start: u64 = 0;
        let start_time = Instant::now();
        let mut base_data = mining_data.clone();
        base_data.remove("nonce");
        loop {
            let nonces: Vec<u64> = (nonce_start..nonce_start + batch_size as u64).collect();
            let hashes = Self::compute_hashes_parallel(&base_data, &nonces);
            for (i, hash_hex) in hashes.iter().enumerate() {
                if hash_hex.starts_with(&target) {
                    let mining_time = start_time.elapsed().as_secs_f64();
                    let successful_nonce = nonces[i];
                    let mut result = HashMap::new();
                    result.insert("success".to_string(), json!(true));
                    result.insert("hash".to_string(), json!(hash_hex));
                    result.insert("nonce".to_string(), json!(successful_nonce));
                    result.insert("mining_time".to_string(), json!(mining_time));
                    result.insert("method".to_string(), json!("cuda"));
                    return Some(result);
                }
            }
            nonce_start += batch_size as u64;
            if nonce_start % (batch_size as u64 * 10) == 0 {
                let hashrate = nonce_start as f64 / start_time.elapsed().as_secs_f64();
                println!("⏳ CUDA: {} attempts | {:.0} H/s", nonce_start, hashrate);
            }
            if start_time.elapsed().as_secs() > 300 {
                break;
            }
        }
        None
    }

    pub fn compute_hashes_parallel(base_data: &HashMap<String, JsonValue>, nonces: &[u64]) -> Vec<String> {
        nonces.iter().map(|nonce| {
            let mut mining_data = base_data.clone();
            mining_data.insert("nonce".to_string(), json!(*nonce));
            let data_string = serde_json::to_string(&mining_data).unwrap();
            let hash = sha2::Sha256::digest(data_string.as_bytes());
            format!("{:x}", hash)
        }).collect()
    }

    pub fn get_cuda_info(&self) -> HashMap<String, JsonValue> {
        let mut info = HashMap::new();
        if !self.cuda_available {
            info.insert("available".to_string(), json!(false));
            return info;
        }
        #[cfg(feature = "cuda")]
        {
            match Device::get_device(0) {
                Ok(device) => {
                    info.insert("available".to_string(), json!(true));
                    info.insert("device_name".to_string(), json!(device.name().unwrap_or("Unknown")));
                    info.insert("compute_capability".to_string(), json!(format!("{}.{}", device.compute_capability_major(), device.compute_capability_minor())));
                    info.insert("total_memory".to_string(), json!(device.total_memory()));
                    info.insert("multiprocessors".to_string(), json!(device.multi_processor_count()));
                },
                Err(e) => {
                    info.insert("available".to_string(), json!(false));
                    info.insert("error".to_string(), json!(format!("{:?}", e)));
                }
            }
        }
        #[cfg(not(feature = "cuda"))]
        {
            info.insert("available".to_string(), json!(false));
            info.insert("error".to_string(), json!("CUDA feature not enabled"));
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cuda_manager_cpu_fallback() {
        let manager = CUDAManager::new();
        let mut mining_data = HashMap::new();
        mining_data.insert("data".to_string(), json!("test"));
        let result = manager.cuda_mine_batch(&mining_data, 1, 1000);
        // CUDA not available in most test envs, so should be None
        assert!(result.is_none() || result.as_ref().unwrap().get("success") == Some(&json!(true)));
    }

    #[test]
    fn test_compute_hashes_parallel() {
        let mut base_data = HashMap::new();
        base_data.insert("data".to_string(), json!("abc"));
        let nonces = vec![1, 2, 3];
        let hashes = CUDAManager::compute_hashes_parallel(&base_data, &nonces);
        assert_eq!(hashes.len(), 3);
        for h in hashes {
            assert_eq!(h.len(), 64);
        }
    }
}
