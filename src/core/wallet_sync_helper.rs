use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::wallet_manager::{WalletManager, Transaction, TransactionType, TransactionStatus, WalletBalance};

pub trait BlockchainSync: Send + Sync {
    fn scan_transactions_for_addresses(&self, addresses: &[String]) -> HashMap<String, Vec<Transaction>>;
}

pub trait MempoolSync: Send + Sync {
    fn get_pending_transactions_for_addresses(&self, addresses: &[String]) -> HashMap<String, Vec<Transaction>>;
}

pub struct WalletSyncHelper<B: BlockchainSync, M: MempoolSync> {
    pub wallet_manager: Arc<WalletManager>,
    pub blockchain: Arc<B>,
    pub mempool: Arc<M>,
    pub sync_thread: Option<thread::JoinHandle<()>>,
    pub stop_flag: Arc<Mutex<bool>>,
}

impl<B: BlockchainSync + 'static, M: MempoolSync + 'static> WalletSyncHelper<B, M> {
    pub fn new(wallet_manager: Arc<WalletManager>, blockchain: Arc<B>, mempool: Arc<M>) -> Self {
        WalletSyncHelper {
            wallet_manager,
            blockchain,
            mempool,
            sync_thread: None,
            stop_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn register_wallets(&self, addresses: &[String]) {
        self.wallet_manager.register_wallets(addresses);
    }

    pub fn sync_wallets_now(&self) {
        let addresses: Vec<String> = self.wallet_manager.get_all_wallet_states().keys().cloned().collect();
        let blockchain_txs = self.blockchain.scan_transactions_for_addresses(&addresses);
        let mempool_txs = self.mempool.get_pending_transactions_for_addresses(&addresses);
        self.wallet_manager.sync_wallets_from_sources(&blockchain_txs, &mempool_txs);
    }

    pub fn get_wallet_balance(&self, address: &str) -> Option<WalletBalance> {
        self.wallet_manager.get_wallet_state(address).map(|s| s.balance)
    }

    pub fn get_wallet_transactions(&self, address: &str, tx_type: Option<&str>) -> Vec<Transaction> {
        if let Some(state) = self.wallet_manager.get_wallet_state(address) {
            match tx_type {
                Some("confirmed") => state.confirmed_transactions,
                Some("pending") => state.pending_transactions,
                Some("all") | None => {
                    let mut all = state.confirmed_transactions;
                    all.extend(state.pending_transactions);
                    all
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn start_continuous_sync<F>(&mut self, poll_interval_secs: u64, on_update: F)
    where
        F: Fn() + Send + 'static,
    {
        let wallet_manager = Arc::clone(&self.wallet_manager);
        let blockchain = Arc::clone(&self.blockchain);
        let mempool = Arc::clone(&self.mempool);
        let stop_flag = Arc::clone(&self.stop_flag);
        self.sync_thread = Some(thread::spawn(move || {
            while !*stop_flag.lock().unwrap() {
                let addresses: Vec<String> = wallet_manager.get_all_wallet_states().keys().cloned().collect();
                let blockchain_txs = blockchain.scan_transactions_for_addresses(&addresses);
                let mempool_txs = mempool.get_pending_transactions_for_addresses(&addresses);
                wallet_manager.sync_wallets_from_sources(&blockchain_txs, &mempool_txs);
                on_update();
                thread::sleep(Duration::from_secs(poll_interval_secs));
            }
        }));
    }

    pub fn stop_continuous_sync(&mut self) {
        *self.stop_flag.lock().unwrap() = true;
        if let Some(handle) = self.sync_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::wallet_manager::{WalletManager, Transaction, TransactionType, TransactionStatus};
    use std::sync::Arc;

    struct DummyBlockchainManager;
    struct DummyMempoolManager;

    impl BlockchainSync for DummyBlockchainManager {
        fn scan_transactions_for_addresses(&self, addresses: &[String]) -> HashMap<String, Vec<Transaction>> {
            let mut map = HashMap::new();
            for addr in addresses {
                map.insert(addr.clone(), vec![Transaction {
                    hash: "h1".to_string(),
                    tx_type: TransactionType::Transfer,
                    from_address: "bob".to_string(),
                    to_address: addr.clone(),
                    amount: 100.0,
                    fee: 1.0,
                    timestamp: 0,
                    status: TransactionStatus::Confirmed,
                    block_height: Some(1),
                    confirmations: 10,
                    memo: String::new(),
                }]);
            }
            map
        }
    }

    impl MempoolSync for DummyMempoolManager {
        fn get_pending_transactions_for_addresses(&self, addresses: &[String]) -> HashMap<String, Vec<Transaction>> {
            let mut map = HashMap::new();
            for addr in addresses {
                map.insert(addr.clone(), vec![Transaction {
                    hash: "h2".to_string(),
                    tx_type: TransactionType::Transfer,
                    from_address: addr.clone(),
                    to_address: "bob".to_string(),
                    amount: 10.0,
                    fee: 0.1,
                    timestamp: 0,
                    status: TransactionStatus::Pending,
                    block_height: None,
                    confirmations: 0,
                    memo: String::new(),
                }]);
            }
            map
        }
    }

    #[test]
    fn test_sync_and_balance() {
        let wallet_manager = Arc::new(WalletManager::new());
        let blockchain = Arc::new(DummyBlockchainManager);
        let mempool = Arc::new(DummyMempoolManager);
        let helper = WalletSyncHelper::new(wallet_manager.clone(), blockchain.clone(), mempool.clone());
        let addresses = vec!["alice".to_string()];
        helper.register_wallets(&addresses);
        helper.sync_wallets_now();
        let bal = helper.get_wallet_balance("alice").unwrap();
        assert_eq!(bal.confirmed_balance, 100.0);
        assert_eq!(bal.pending_outgoing, 10.1);
        let txs = helper.get_wallet_transactions("alice", Some("all"));
        assert_eq!(txs.len(), 2);
    }
}
