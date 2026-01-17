pub struct Crypto;

use crate::core::sm2::SM2;
use sha2::{Digest, Sha256};

impl Crypto {
    pub fn new() -> Self {
        Crypto
    }
    pub fn generate_keypair(&self) -> (String, String, String) {
        let sm2 = SM2::new();
        let (private_key, public_key) = sm2.generate_keypair();
        let address = sm2.public_key_to_address(&public_key);
        (private_key, public_key, address)
    }
    pub fn generate_private_key(&self) -> String {
        let (private_key, _, _) = self.generate_keypair();
        private_key
    }
    pub fn derive_public_key(&self, private_key_hex: &str) -> String {
        let sm2 = SM2::new();
        sm2.derive_public_key(private_key_hex)
    }
    pub fn derive_address(&self, public_key_hex: &str) -> String {
        let sm2 = SM2::new();
        sm2.public_key_to_address(public_key_hex)
    }
    pub fn sign_data(&self, data: &str, private_key_hex: &str) -> String {
        let sm2 = SM2::new();
        sm2.sign(data, private_key_hex)
    }
    pub fn verify_signature(&self, data: &str, signature: &str, public_key_hex: &str) -> bool {
        let sm2 = SM2::new();
        sm2.verify(data, signature, public_key_hex)
    }
    pub fn validate_key_pair(&self, private_key_hex: &str, public_key_hex: &str) -> bool {
        let test_data = "SM2 key validation test";
        let signature = self.sign_data(test_data, private_key_hex);
        self.verify_signature(test_data, &signature, public_key_hex)
    }
    pub fn get_key_info(&self, private_key_hex: Option<&str>, public_key_hex: Option<&str>) -> serde_json::Value {
        let mut info = serde_json::json!({
            "crypto_standard": "SM2 (GB/T 32918)",
            "curve": "SM2 P-256",
            "key_size_bits": 256
        });
        if let Some(privk) = private_key_hex {
            info["private_key_length"] = serde_json::json!(privk.len());
            info["private_key_prefix"] = serde_json::json!(&privk[..8.min(privk.len())]);
        }
        if let Some(pubk) = public_key_hex {
            info["public_key_length"] = serde_json::json!(pubk.len());
            info["public_key_format"] = serde_json::json!(if pubk.starts_with("04") { "uncompressed" } else { "unknown" });
            info["address"] = serde_json::json!(self.derive_address(pubk));
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let crypto = Crypto::new();
        let (privk, pubk, addr) = crypto.generate_keypair();
        assert_eq!(privk.len(), 64);
        assert!(pubk.starts_with("04"));
        assert!(addr.starts_with("LUN_"));
    }

    #[test]
    fn test_sign_and_verify() {
        let crypto = Crypto::new();
        let (privk, pubk, _) = crypto.generate_keypair();
        let msg = "Hello, SM2 cryptography!";
        let sig = crypto.sign_data(msg, &privk);
        assert!(crypto.verify_signature(msg, &sig, &pubk));
    }

    #[test]
    fn test_derive_address() {
        let crypto = Crypto::new();
        let (_, pubk, addr) = crypto.generate_keypair();
        let derived = crypto.derive_address(&pubk);
        assert_eq!(addr, derived);
    }

    #[test]
    fn test_validate_key_pair() {
        let crypto = Crypto::new();
        let (privk, pubk, _) = crypto.generate_keypair();
        assert!(crypto.validate_key_pair(&privk, &pubk));
    }

    #[test]
    fn test_get_key_info() {
        let crypto = Crypto::new();
        let (privk, pubk, _) = crypto.generate_keypair();
        let info = crypto.get_key_info(Some(&privk), Some(&pubk));
        assert_eq!(info["crypto_standard"], "SM2 (GB/T 32918)");
        assert_eq!(info["curve"], "SM2 P-256");
        assert_eq!(info["key_size_bits"], 256);
    }
}


