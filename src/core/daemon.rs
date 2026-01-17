
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct Daemon {
    pub is_running: bool,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub stats: Arc<Mutex<DaemonStats>>,
    // ...existing code...
}

#[derive(Default, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub registered_at: u64,
    pub last_seen: u64,
    pub capabilities: Vec<String>,
    pub url: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Default)]
pub struct DaemonStats {
    pub blocks_validated: u64,
    pub transactions_validated: u64,
    pub peers_registered: u64,
    pub start_time: u64,
}

impl Daemon {
    pub fn new() -> Self {
        Daemon {
            is_running: false,
            peers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(DaemonStats {
                start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                ..Default::default()
            })),
        }
    }

    pub fn start(&mut self) {
        if self.is_running { return; }
        self.is_running = true;
        // ...スレッド起動など...
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        // ...スレッド停止など...
    }

    pub fn register_peer(&self, peer: PeerInfo) -> bool {
        let mut peers = self.peers.lock().unwrap();
        if peers.contains_key(&peer.node_id) { return false; }
        peers.insert(peer.node_id.clone(), peer);
        let mut stats = self.stats.lock().unwrap();
        stats.peers_registered += 1;
        true
    }

    pub fn unregister_peer(&self, node_id: &str) -> bool {
        let mut peers = self.peers.lock().unwrap();
        peers.remove(node_id).is_some()
    }

    pub fn get_peer_list(&self) -> Vec<PeerInfo> {
        let peers = self.peers.lock().unwrap();
        peers.values().cloned().collect()
    }

    pub fn get_stats(&self) -> DaemonStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_registration() {
        let daemon = Daemon::new();
        let peer = PeerInfo {
            node_id: "node1".to_string(),
            registered_at: 1,
            last_seen: 1,
            capabilities: vec!["mining".to_string()],
            url: Some("http://localhost".to_string()),
            version: Some("0.1.0".to_string()),
        };
        assert!(daemon.register_peer(peer.clone()));
        assert!(!daemon.register_peer(peer.clone())); // duplicate
        let peers = daemon.get_peer_list();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, "node1");
        assert!(daemon.unregister_peer("node1"));
        assert!(!daemon.unregister_peer("node1"));
    }

    #[test]
    fn test_stats() {
        let daemon = Daemon::new();
        let stats = daemon.get_stats();
        assert_eq!(stats.blocks_validated, 0);
        assert_eq!(stats.transactions_validated, 0);
        assert_eq!(stats.peers_registered, 0);
    }
}
