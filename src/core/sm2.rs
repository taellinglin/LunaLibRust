pub struct SM2;

use rand::RngCore;
use sha2::{Digest, Sha256};

impl SM2 {
    pub fn new() -> Self {
        SM2
    }
    pub fn generate_keypair(&self) -> (String, String) {
        // 64 hex chars private, 130 hex chars public (04 + 128)
        let mut priv_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut priv_bytes);
        let private_key = hex::encode(priv_bytes);
        let public_key = format!("04{:064x}{:064x}", priv_bytes[0], priv_bytes[1]); // dummy
        (private_key, public_key)
    }
    pub fn public_key_to_address(&self, public_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        format!("LUN_{}", &hex::encode(&hash)[..16])
    }
    pub fn derive_public_key(&self, private_key_hex: &str) -> String {
        // Dummy: just hash the private key
        let mut hasher = Sha256::new();
        hasher.update(private_key_hex.as_bytes());
        let hash = hasher.finalize();
        format!("04{}{}", hex::encode(&hash)[..32].to_string(), hex::encode(&hash)[32..].to_string())
    }
    pub fn sign(&self, data: &str, private_key_hex: &str) -> String {
        // Dummy: hash(data + priv)
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(private_key_hex.as_bytes());
        let hash = hasher.finalize();
        format!("{:0>128}", hex::encode(hash))
    }
    pub fn verify(&self, data: &str, signature: &str, _public_key_hex: &str) -> bool {
        // Dummy: always true if signature is 128 chars
        signature.len() == 128
    }
}
