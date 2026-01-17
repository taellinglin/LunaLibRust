
use base64::{engine::general_purpose, Engine as _};
use ring::pbkdf2;
use ring::digest;
use ring::hmac;
use rand::RngCore;
use serde_json::{Value as JsonValue, json};
use std::num::NonZeroU32;
use std::collections::HashMap;

const SALT: &[u8] = b"luna_wallet_salt";
const PBKDF2_ITER: u32 = 100_000;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 16;
const MAC_LEN: usize = 32;

#[derive(Debug, Clone)]
pub struct EncryptionManager {
    pub salt: Vec<u8>,
}

impl EncryptionManager {
    pub fn new() -> Self {
        EncryptionManager { salt: SALT.to_vec() }
    }

    fn derive_key(&self, password: &str) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITER).unwrap(),
            &self.salt,
            password.as_bytes(),
            &mut key,
        );
        key
    }

    fn keystream(&self, key: &[u8], nonce: &[u8], length: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(length);
        let mut counter = 0u32;
        while output.len() < length {
            let mut counter_bytes = [0u8; 4];
            counter_bytes.copy_from_slice(&counter.to_be_bytes());
            let mut mac = hmac::Key::new(hmac::HMAC_SHA256, key);
            let digest = hmac::sign(&mac, &[nonce, &counter_bytes].concat());
            output.extend_from_slice(digest.as_ref());
            counter += 1;
        }
        output.truncate(length);
        output
    }

    fn encrypt_bytes(&self, plaintext: &[u8], password: &str) -> String {
        let key = self.derive_key(password);
        let mut nonce = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce);
        let stream = self.keystream(&key, &nonce, plaintext.len());
        let ciphertext: Vec<u8> = plaintext.iter().zip(stream.iter()).map(|(a, b)| a ^ b).collect();
        let mut mac_key = hmac::Key::new(hmac::HMAC_SHA256, &key);
        let mac = hmac::sign(&mac_key, &[nonce.as_ref(), &ciphertext].concat());
        let mut token = b"EL1".to_vec();
        token.extend_from_slice(&nonce);
        token.extend_from_slice(&ciphertext);
        token.extend_from_slice(mac.as_ref());
        general_purpose::URL_SAFE_NO_PAD.encode(token)
    }

    fn decrypt_bytes(&self, token: &str, password: &str) -> Result<Vec<u8>, String> {
        let key = self.derive_key(password);
        let raw = general_purpose::URL_SAFE_NO_PAD.decode(token).map_err(|e| format!("base64 decode: {e}"))?;
        if !raw.starts_with(b"EL1") {
            return Err("Unsupported encryption format".to_string());
        }
        let nonce = &raw[3..19];
        let mac = &raw[raw.len()-MAC_LEN..];
        let ciphertext = &raw[19..raw.len()-MAC_LEN];
        let mut mac_key = hmac::Key::new(hmac::HMAC_SHA256, &key);
        let expected_mac = hmac::sign(&mac_key, &[nonce, ciphertext].concat());
        if mac != expected_mac.as_ref() {
            return Err("Invalid password or corrupted data".to_string());
        }
        let stream = self.keystream(&key, nonce, ciphertext.len());
        Ok(ciphertext.iter().zip(stream.iter()).map(|(a, b)| a ^ b).collect())
    }

    pub fn encrypt_wallet(&self, wallet_data: &mut JsonValue, password: &str) -> JsonValue {
        if let Some(private_key) = wallet_data.get("private_key").and_then(|v| v.as_str()) {
            let encrypted_private = self.encrypt_bytes(private_key.as_bytes(), password);
            wallet_data["encrypted_private_key"] = json!(encrypted_private);
            wallet_data.as_object_mut().unwrap().remove("private_key");
        }
        let wallet_json = serde_json::to_string(wallet_data).unwrap();
        let encrypted_wallet = self.encrypt_bytes(wallet_json.as_bytes(), password);
        json!({
            "encrypted_data": encrypted_wallet,
            "version": "1.0",
            "salt": general_purpose::STANDARD.encode(&self.salt)
        })
    }

    pub fn decrypt_wallet(&self, encrypted_data: &JsonValue, password: &str) -> Option<JsonValue> {
        let encrypted_wallet = encrypted_data.get("encrypted_data").and_then(|v| v.as_str())?;
        let decrypted_bytes = self.decrypt_bytes(encrypted_wallet, password).ok()?;
        let mut wallet_data: JsonValue = serde_json::from_slice(&decrypted_bytes).ok()?;
        if let Some(encrypted_private) = wallet_data.get("encrypted_private_key").and_then(|v| v.as_str()) {
            if let Ok(private_bytes) = self.decrypt_bytes(encrypted_private, password) {
                wallet_data["private_key"] = json!(String::from_utf8_lossy(&private_bytes));
                wallet_data.as_object_mut().unwrap().remove("encrypted_private_key");
            }
        }
        Some(wallet_data)
    }

    pub fn verify_password(&self, encrypted_data: &JsonValue, password: &str) -> bool {
        let token = encrypted_data.get("encrypted_data").and_then(|v| v.as_str());
        if token.is_none() { return false; }
        self.decrypt_bytes(token.unwrap(), password).is_ok()
    }

    pub fn encrypt_data(&self, data: &str, password: &str) -> String {
        self.encrypt_bytes(data.as_bytes(), password)
    }

    pub fn decrypt_data(&self, encrypted_data: &str, password: &str) -> Option<String> {
        self.decrypt_bytes(encrypted_data, password).ok().and_then(|v| String::from_utf8(v).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encrypt_decrypt_wallet() {
        let mut wallet = json!({
            "address": "addr1",
            "private_key": "mysecretkey",
            "balance": 100.0
        });
        let manager = EncryptionManager::new();
        let encrypted = manager.encrypt_wallet(&mut wallet, "password123");
        assert!(encrypted["encrypted_data"].is_string());
        let decrypted = manager.decrypt_wallet(&encrypted, "password123").unwrap();
        assert_eq!(decrypted["address"], "addr1");
        assert_eq!(decrypted["private_key"], "mysecretkey");
        assert_eq!(decrypted["balance"], 100.0);
    }

    #[test]
    fn test_verify_password() {
        let mut wallet = json!({"address": "addr2", "private_key": "key2"});
        let manager = EncryptionManager::new();
        let encrypted = manager.encrypt_wallet(&mut wallet, "pw");
        assert!(manager.verify_password(&encrypted, "pw"));
        assert!(!manager.verify_password(&encrypted, "wrongpw"));
    }

    #[test]
    fn test_encrypt_decrypt_data() {
        let manager = EncryptionManager::new();
        let encrypted = manager.encrypt_data("hello world", "pw");
        let decrypted = manager.decrypt_data(&encrypted, "pw").unwrap();
        assert_eq!(decrypted, "hello world");
    }
}
