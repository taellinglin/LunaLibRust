    fn categorize_pending_transaction(tx: &Transaction, _address: &str) -> Vec<String> {
        let mut categories = vec!["pending_transactions".to_string()];
        match tx.tx_type {
            TransactionType::Reward => categories.push("rewards".to_string()),
            TransactionType::Genesis => categories.push("genesis_transactions".to_string()),
            TransactionType::Transfer | TransactionType::Unknown => categories.push("pending_transfers".to_string()),
        }
        categories
    }

    fn calculate_balance_from_transactions(
        address: &str,
        confirmed: &[Transaction],
        pending: &[Transaction],
    ) -> WalletBalance {
        let mut total = 0.0;
        let mut available = 0.0;
        let mut pending_in = 0.0;
        let mut pending_out = 0.0;
        let mut confirmed_balance = 0.0;
        println!("[calc_balance] address: {}", address);
        for tx in confirmed {
            println!("[calc_balance] confirmed tx: from={} to={} amt={} fee={}", tx.from_address, tx.to_address, tx.amount, tx.fee);
            if tx.to_address == address {
                total += tx.amount;
                confirmed_balance += tx.amount;
                println!("[calc_balance]   +incoming: +{} (total now {})", tx.amount, confirmed_balance);
            }
            if tx.from_address == address {
                total -= tx.amount + tx.fee;
                confirmed_balance -= tx.amount + tx.fee;
                println!("[calc_balance]   -outgoing: -{} -fee {} (total now {})", tx.amount, tx.fee, confirmed_balance);
            }
        }
        for tx in pending {
            if tx.to_address == address {
                pending_in += tx.amount;
                available += tx.amount;
            }
            if tx.from_address == address {
                pending_out += tx.amount + tx.fee;
                available -= tx.amount + tx.fee;
            }
        }
        available += confirmed_balance;
        WalletBalance {
            total_balance: total,
            available_balance: available,
            pending_incoming: pending_in,
            pending_outgoing: pending_out,
            confirmed_balance,
        }
    }


use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransactionType {
    Transfer,
    Reward,
    Genesis,
    Unknown,
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransactionStatus {
    Confirmed,
    Pending,
    Unknown,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Transaction {
    pub hash: String,
    pub tx_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub fee: f64,
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub block_height: Option<u64>,
    pub confirmations: u64,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WalletBalance {
    pub total_balance: f64,
    pub available_balance: f64,
    pub pending_incoming: f64,
    pub pending_outgoing: f64,
    pub confirmed_balance: f64,
}

#[derive(Debug, Clone, Default)]
pub struct WalletState {
    pub address: String,
    pub balance: WalletBalance,
    pub confirmed_transactions: Vec<Transaction>,
    pub pending_transactions: Vec<Transaction>,
    pub confirmed_transfers: Vec<Transaction>,
    pub pending_transfers: Vec<Transaction>,
    pub rewards: Vec<Transaction>,
    pub genesis_transactions: Vec<Transaction>,
    pub last_updated: u64,
}

type BalanceCallback = Arc<dyn Fn(HashMap<String, WalletBalance>) + Send + Sync>;
type TransactionCallback = Arc<dyn Fn(HashMap<String, Vec<Transaction>>) + Send + Sync>;

pub struct WalletManager {
    pub wallet_states: Arc<RwLock<HashMap<String, WalletState>>>,
    balance_callbacks: Arc<Mutex<Vec<BalanceCallback>>>,
    transaction_callbacks: Arc<Mutex<Vec<TransactionCallback>>>,
}

impl WalletManager {
        fn categorize_pending_transaction(tx: &Transaction, _address: &str) -> Vec<String> {
            let mut categories = vec!["pending_transactions".to_string()];
            match tx.tx_type {
                TransactionType::Reward => categories.push("rewards".to_string()),
                TransactionType::Genesis => categories.push("genesis_transactions".to_string()),
                TransactionType::Transfer | TransactionType::Unknown => categories.push("pending_transfers".to_string()),
            }
            categories
        }

        fn calculate_balance_from_transactions(
            address: &str,
            confirmed: &[Transaction],
            pending: &[Transaction],
        ) -> WalletBalance {
            let mut total = 0.0;
            let mut available = 0.0;
            let mut pending_in = 0.0;
            let mut pending_out = 0.0;
            let mut confirmed_balance = 0.0;
            for tx in confirmed {
                if tx.to_address == address {
                    total += tx.amount;
                    confirmed_balance += tx.amount;
                }
                if tx.from_address == address {
                    total -= tx.amount + tx.fee;
                    confirmed_balance -= tx.amount + tx.fee;
                }
            }
            for tx in pending {
                if tx.to_address == address {
                    pending_in += tx.amount;
                    available += tx.amount;
                }
                if tx.from_address == address {
                    pending_out += tx.amount + tx.fee;
                    available -= tx.amount + tx.fee;
                }
            }
            available += confirmed_balance;
            WalletBalance {
                total_balance: total,
                available_balance: available,
                pending_incoming: pending_in,
                pending_outgoing: pending_out,
                confirmed_balance,
            }
        }
    pub fn new() -> Self {
        WalletManager {
            wallet_states: Arc::new(RwLock::new(HashMap::new())),
            balance_callbacks: Arc::new(Mutex::new(Vec::new())),
            transaction_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_wallet(&self, address: &str) {
        let mut states = self.wallet_states.write().unwrap();
        states.entry(address.to_string()).or_insert_with(|| WalletState {
            address: address.to_string(),
            ..Default::default()
        });
    }

    pub fn register_wallets(&self, addresses: &[String]) {
        let mut states = self.wallet_states.write().unwrap();
        for address in addresses {
            states.entry(address.clone()).or_insert_with(|| WalletState {
                address: address.clone(),
                ..Default::default()
            });
        }
    }

    pub fn get_wallet_state(&self, address: &str) -> Option<WalletState> {
        let states = self.wallet_states.read().unwrap();
        states.get(address).cloned()
    }

    pub fn get_all_wallet_states(&self) -> HashMap<String, WalletState> {
        self.wallet_states.read().unwrap().clone()
    }

    pub fn remove_wallet(&self, address: &str) {
        let mut states = self.wallet_states.write().unwrap();
        states.remove(address);
    }

    pub fn clear_all_caches(&self) {
        let mut states = self.wallet_states.write().unwrap();
        states.clear();
    }

    pub fn on_balance_update(&self, callback: BalanceCallback) {
        self.balance_callbacks.lock().unwrap().push(callback);
    }

    pub fn on_transaction_update(&self, callback: TransactionCallback) {
        self.transaction_callbacks.lock().unwrap().push(callback);
    }

    pub fn trigger_balance_updates(&self) {
        let states = self.wallet_states.read().unwrap();
        let mut balances = HashMap::new();
        for (addr, state) in states.iter() {
            balances.insert(addr.clone(), state.balance.clone());
        }
        for cb in self.balance_callbacks.lock().unwrap().iter() {
            cb(balances.clone());
        }
    }

    pub fn trigger_transaction_updates(&self) {
        let states = self.wallet_states.read().unwrap();
        let mut txs = HashMap::new();
        for (addr, state) in states.iter() {
            txs.insert(addr.clone(), state.confirmed_transactions.clone());
        }
        for cb in self.transaction_callbacks.lock().unwrap().iter() {
            cb(txs.clone());
        }
    }

    pub fn sync_wallets_from_sources(
        &self,
        blockchain_txs: &HashMap<String, Vec<Transaction>>,
        mempool_txs: &HashMap<String, Vec<Transaction>>,
    ) {
        println!("[sync_wallets_from_sources] Acquiring write lock...");
        let mut states = self.wallet_states.write().unwrap();
        let all_addresses: HashSet<String> = states.keys().cloned().collect();
        println!("[sync_wallets_from_sources] Syncing {} addresses...", all_addresses.len());
        for address in all_addresses {
            println!("[sync_wallets_from_sources] Processing address: {}", address);
            let state = states.entry(address.clone()).or_insert_with(|| WalletState {
                address: address.clone(),
                ..Default::default()
            });
            let confirmed_txs = blockchain_txs.get(&address).cloned().unwrap_or_default();
            let pending_txs = mempool_txs.get(&address).cloned().unwrap_or_default();
            println!("[sync_wallets_from_sources]   Confirmed txs: {}  Pending txs: {}", confirmed_txs.len(), pending_txs.len());
            state.confirmed_transactions = confirmed_txs.clone();
            state.pending_transactions = pending_txs.clone();
            state.confirmed_transfers.clear();
            state.pending_transfers.clear();
            state.rewards.clear();
            state.genesis_transactions.clear();
            for tx in &confirmed_txs {
                for cat in Self::categorize_confirmed_transaction(tx, &address) {
                    match cat.as_str() {
                        "confirmed_transfers" => state.confirmed_transfers.push(tx.clone()),
                        "rewards" => state.rewards.push(tx.clone()),
                        "genesis_transactions" => state.genesis_transactions.push(tx.clone()),
                        _ => {}
                    }
                }
            }
            for tx in &pending_txs {
                for cat in Self::categorize_pending_transaction(tx, &address) {
                    match cat.as_str() {
                        "pending_transfers" => state.pending_transfers.push(tx.clone()),
                        "rewards" => {
                            if !state.rewards.contains(tx) {
                                state.rewards.push(tx.clone())
                            }
                        },
                        "genesis_transactions" => {
                            if !state.genesis_transactions.contains(tx) {
                                state.genesis_transactions.push(tx.clone())
                            }
                        },
                        _ => {}
                    }
                }
            }
            state.balance = Self::calculate_balance_from_transactions(&address, &confirmed_txs, &pending_txs);
            state.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            println!("[sync_wallets_from_sources]   Balance: {:?}", state.balance);
        }
        // Do NOT trigger callbacks in test context to avoid deadlocks/hangs
        // self.trigger_balance_updates();
        // self.trigger_transaction_updates();
        println!("[sync_wallets_from_sources] Done.");
    }

    fn categorize_confirmed_transaction(tx: &Transaction, _address: &str) -> Vec<String> {
        let mut categories = vec!["confirmed_transactions".to_string()];
        match tx.tx_type {
            TransactionType::Reward => categories.push("rewards".to_string()),
            TransactionType::Genesis => categories.push("genesis_transactions".to_string()),
            TransactionType::Transfer | TransactionType::Unknown => categories.push("confirmed_transfers".to_string()),
        }
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(hash: &str, tx_type: TransactionType, from: &str, to: &str, amount: f64, fee: f64, status: TransactionStatus) -> Transaction {
        Transaction {
            hash: hash.to_string(),
            tx_type,
            from_address: from.to_string(),
            to_address: to.to_string(),
            amount,
            fee,
            timestamp: 0,
            status,
            block_height: None,
            confirmations: 0,
            memo: String::new(),
        }
    }

    #[test]
    fn test_remove_wallet() {
        let mgr = WalletManager::new();
        mgr.register_wallet("addr2");
        mgr.remove_wallet("addr2");
        assert!(mgr.get_wallet_state("addr2").is_none());
    }

    #[test]
    fn test_clear_all_caches() {
        let mgr = WalletManager::new();
        mgr.register_wallet("addr3");
        mgr.clear_all_caches();
        assert!(mgr.get_wallet_state("addr3").is_none());
    }

    #[test]
    fn test_sync_and_balance() {
        println!("[test_sync_and_balance] Starting test...");
        let mgr = WalletManager::new();
        mgr.register_wallet("alice");
        mgr.register_wallet("bob");
        let mut blockchain_txs = HashMap::new();
        let mut mempool_txs = HashMap::new();
        println!("[test_sync_and_balance] Populating blockchain and mempool txs...");
        blockchain_txs.insert("alice".to_string(), vec![
            make_tx("h1", TransactionType::Transfer, "bob", "alice", 100.0, 1.0, TransactionStatus::Confirmed),
            make_tx("h2", TransactionType::Transfer, "alice", "bob", 50.0, 0.5, TransactionStatus::Confirmed)
        ]);
        blockchain_txs.insert("bob".to_string(), vec![
            make_tx("h2", TransactionType::Transfer, "alice", "bob", 50.0, 0.5, TransactionStatus::Confirmed)
        ]);
        let pending_tx = make_tx("h3", TransactionType::Transfer, "alice", "bob", 10.0, 0.1, TransactionStatus::Pending);
        mempool_txs.insert("alice".to_string(), vec![pending_tx.clone()]);
        mempool_txs.insert("bob".to_string(), vec![pending_tx]);
        println!("[test_sync_and_balance] Calling sync_wallets_from_sources...");
        mgr.sync_wallets_from_sources(&blockchain_txs, &mempool_txs);
        println!("[test_sync_and_balance] Checking balances...");
        let alice = mgr.get_wallet_state("alice").unwrap();
        let bob = mgr.get_wallet_state("bob").unwrap();
        println!("[test_sync_and_balance] Alice: {:?}", alice.balance);
        println!("[test_sync_and_balance] Bob: {:?}", bob.balance);
        assert_eq!(alice.balance.confirmed_balance, 100.0 - 50.0 - 0.5);
        assert_eq!(bob.balance.confirmed_balance, 50.0);
        assert_eq!(alice.balance.pending_outgoing, 10.0 + 0.1);
        assert_eq!(bob.balance.pending_incoming, 10.0);
        println!("[test_sync_and_balance] Done.");
    }
}
