//! # LunaLibRust
//!
//! Rust implementation of LunaLib: a cryptocurrency wallet and mining system.
//!
//! ## Quick Start Example
//!
//! ```rust
//! use lunalib_rust::luna_lib::*;
//!
//! // Create a wallet
//! let wallet = create_wallet();
//! // Create a miner
//! let miner = create_miner();
//! // Create a blockchain manager
//! let blockchain = create_blockchain_manager(Some("https://bank.linglin.art"));
//! // Create a mempool manager
//! let mempool = create_mempool_manager(None);
//! // Get a transaction manager
//! let tx_manager = get_transaction_manager();
//! // Print version
//! println!("{}", LunaLib::get_version());
//! ```
//!
//! Main library entry point exposing all core functionality
//!
use crate::core::wallet::LunaWallet;
use crate::mining::miner::GenesisMiner;
use crate::gtx::genesis::GTXGenesis;
use crate::transactions::transactions::TransactionManager;
use crate::core::blockchain::BlockchainManager;
use crate::core::mempool::MempoolManager;

pub struct LunaLib;

impl LunaLib {
    /// Get the current library version
    pub fn get_version() -> &'static str {
        "1.0.0"
    }

    /// Get list of all available classes in the library
    pub fn get_available_classes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Wallet", "LunaWallet - Cryptocurrency wallet management"),
            ("Miner", "GenesisMiner - Mining operations"),
            ("GTX", "GTXGenesis - GTX token operations"),
            ("Transaction", "TransactionManager - Transaction handling"),
            ("Blockchain", "BlockchainManager - Blockchain operations with endpoint support"),
            ("Mempool", "MempoolManager - Memory Pool management and endpoint"),
        ]
    }
}

// Convenience constructors
pub fn create_wallet() -> LunaWallet {
    LunaWallet::new(
        "test_address".to_string(),
        "test_pubkey".to_string(),
        vec![],
        "test_label".to_string(),
        0,
    )
}

pub fn create_miner() -> GenesisMiner {
    GenesisMiner::new(None)
}

pub fn create_blockchain_manager(endpoint_url: Option<&str>) -> BlockchainManager {
    BlockchainManager::new(endpoint_url.unwrap_or("https://bank.linglin.art"), 1)
}

pub fn create_mempool_manager(_endpoint_url: Option<&str>) -> MempoolManager {
    MempoolManager::new()
}

pub fn get_transaction_manager() -> TransactionManager {
    TransactionManager::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(LunaLib::get_version(), "1.0.0");
    }

    #[test]
    fn test_available_classes() {
        let classes = LunaLib::get_available_classes();
        assert!(classes.iter().any(|(name, _)| *name == "Wallet"));
        assert!(classes.iter().any(|(name, _)| *name == "Miner"));
        assert!(classes.iter().any(|(name, _)| *name == "GTX"));
        assert!(classes.iter().any(|(name, _)| *name == "Transaction"));
        assert!(classes.iter().any(|(name, _)| *name == "Blockchain"));
        assert!(classes.iter().any(|(name, _)| *name == "Mempool"));
    }

    #[test]
    fn test_create_wallet() {
        let _wallet = create_wallet();
    }

    #[test]
    fn test_create_miner() {
        let _miner = create_miner();
    }

    #[test]
    fn test_create_blockchain_manager() {
        let _manager = create_blockchain_manager(None);
    }

    #[test]
    fn test_create_mempool_manager() {
        let _manager = create_mempool_manager(None);
    }

    #[test]
    fn test_get_transaction_manager() {
        let _tx_manager = get_transaction_manager();
    }
}
