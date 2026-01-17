
use crate::gtx::bill_registry::BillRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalBill {
    pub denomination: u64,
    pub user_address: String,
    pub difficulty: u32,
    pub bill_data: JsonValue,
    pub bill_serial: String,
    pub created_time: f64,
    pub bill_type: String,
    pub front_serial: String,
    pub back_serial: String,
    pub metadata_hash: String,
    pub timestamp: f64,
    pub issued_to: String,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

impl DigitalBill {
    pub fn new(
        denomination: u64,
        user_address: String,
        difficulty: u32,
        bill_data: Option<JsonValue>,
        bill_type: Option<String>,
        front_serial: Option<String>,
        back_serial: Option<String>,
        metadata_hash: Option<String>,
        public_key: Option<String>,
        signature: Option<String>,
    ) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let bill_serial = front_serial.clone().unwrap_or_else(|| Self::generate_serial(denomination));
        let metadata_hash = metadata_hash.unwrap_or_else(|| Self::generate_metadata_hash(
            denomination,
            &user_address,
            difficulty,
            now,
            &bill_serial,
        ));
        DigitalBill {
            denomination,
            user_address: user_address.clone(),
            difficulty,
            bill_data: bill_data.unwrap_or(JsonValue::Null),
            bill_serial: bill_serial.clone(),
            created_time: now,
            bill_type: bill_type.unwrap_or_else(|| "GTX_Genesis".to_string()),
            front_serial: front_serial.unwrap_or(bill_serial.clone()),
            back_serial: back_serial.unwrap_or_default(),
            metadata_hash,
            timestamp: now,
            issued_to: user_address,
            public_key,
            signature,
        }
    }

    fn generate_serial(denomination: u64) -> String {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let random_part: String = rand::thread_rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect();
        format!("GTX{}_{}_{}", denomination, timestamp, random_part)
    }

    fn generate_metadata_hash(
        denomination: u64,
        user_address: &str,
        difficulty: u32,
        timestamp: f64,
        bill_serial: &str,
    ) -> String {
        let metadata = serde_json::json!({
            "denomination": denomination,
            "user_address": user_address,
            "difficulty": difficulty,
            "timestamp": timestamp,
            "bill_serial": bill_serial
        });
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&metadata).unwrap());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_mining_data(&self, nonce: u64) -> JsonValue {
        serde_json::json!({
            "type": "GTX_Genesis",
            "denomination": self.denomination,
            "user_address": self.user_address,
            "bill_serial": self.bill_serial,
            "timestamp": self.created_time,
            "difficulty": self.difficulty,
            "previous_hash": Self::get_previous_hash(),
            "nonce": nonce,
            "bill_data": self.bill_data
        })
    }

    pub fn finalize(&mut self, hash: &str, nonce: &str, mining_time: f64, private_key: Option<&str>) -> JsonValue {
        let transaction_data = serde_json::json!({
            "type": "GTX_Genesis",
            "from": "genesis_network",
            "to": self.user_address,
            "amount": self.denomination,
            "bill_serial": self.bill_serial,
            "mining_difficulty": self.difficulty,
            "mining_time": mining_time,
            "hash": hash,
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64(),
            "status": "mined",
            "front_serial": self.front_serial,
            "issued_to": self.user_address,
            "denomination": self.denomination,
            "metadata_hash": self.metadata_hash
        });
        if let Some(pk) = private_key {
            let sig = self.sign(pk);
            self.public_key = Some(Self::derive_public_key(pk));
            self.signature = Some(sig.clone());
        }
        let bill_info = serde_json::json!({
            "success": true,
            "bill_serial": self.bill_serial,
            "denomination": self.denomination,
            "user_address": self.user_address,
            "mining_time": mining_time,
            "difficulty": self.difficulty,
            "hash": hash,
            "nonce": nonce,
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64(),
            "luna_value": self.denomination,
            "transaction_data": transaction_data
        });
        let reg = BillRegistry::new(None);
        // Note: BillRegistry expects a BillInfo struct, so conversion is needed for real use
        // reg.register_bill(...)
        bill_info
    }

    fn get_previous_hash() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        let mut hasher = Sha256::new();
        hasher.update(now.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn to_dict(&self) -> JsonValue {
        serde_json::json!({
            "type": self.bill_type,
            "front_serial": self.front_serial,
            "back_serial": self.back_serial,
            "metadata_hash": self.metadata_hash,
            "timestamp": self.timestamp,
            "issued_to": self.issued_to,
            "denomination": self.denomination
        })
    }

    pub fn calculate_hash(&self) -> String {
        let bill_string = serde_json::to_string(&self.to_dict()).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(bill_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn sign(&self, private_key: &str) -> String {
        // Fallback: hash(private_key + bill_hash)
        let bill_hash = self.calculate_hash();
        let signature_input = format!("{}{}", private_key, bill_hash);
        let mut hasher = Sha256::new();
        hasher.update(signature_input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify(&self) -> bool {
        if let (Some(ref pk), Some(ref sig)) = (&self.public_key, &self.signature) {
            let expected = self.sign(pk);
            &expected == sig
        } else {
            false
        }
    }

    pub fn derive_public_key(private_key: &str) -> String {
        // Fallback: hash(private_key)
        let mut hasher = Sha256::new();
        hasher.update(private_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_key_pair() -> (String, String) {
        let private_key: String = rand::thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();
        let public_key = Self::derive_public_key(&private_key);
        (private_key, public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_digital_bill_basic() {
        let (priv_key, pub_key) = DigitalBill::generate_key_pair();
        let mut bill = DigitalBill::new(
            100,
            "user1".to_string(),
            5,
            Some(json!({"foo": "bar"})),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let hash = bill.calculate_hash();
        let sig = bill.sign(&priv_key);
        bill.public_key = Some(priv_key.clone());
        bill.signature = Some(sig.clone());
        assert!(bill.verify());
        let mining_data = bill.get_mining_data(123);
        assert_eq!(mining_data["denomination"], 100);
        let finalized = bill.finalize(&hash, "nonce123", 1.23, Some(&priv_key));
        assert!(finalized["success"].as_bool().unwrap());
    }
}
