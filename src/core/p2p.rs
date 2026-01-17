
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub url: String,
}

pub struct P2P {
    pub primary_node: String,
    pub node_id: String,
    pub peer_url: String,
    pub peers: Arc<Mutex<Vec<PeerInfo>>>,
    pub is_running: bool,
}

impl P2P {
    pub fn new(primary_node: &str, node_id: &str, peer_url: &str) -> Self {
        P2P {
            primary_node: primary_node.to_string(),
            node_id: node_id.to_string(),
            peer_url: peer_url.to_string(),
            peers: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
        }
    }

    pub fn start(&mut self) {
        if self.is_running { return; }
        self.is_running = true;
        // 初期同期・ピア登録・ピアリスト取得（本来は非同期/スレッド）
        self.register_with_primary();
        // 本来はスレッドで定期的にupdate_peer_listや同期処理を行う
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        // 本来はスレッド停止処理
    }

    pub fn register_with_primary(&self) -> bool {
        // 本来はHTTP POSTでプライマリノードに自身を登録
        // ここではダミーでピアリストに自身を追加
        let mut peers = self.peers.lock().unwrap();
        if !peers.iter().any(|p| p.node_id == self.node_id) {
            peers.push(PeerInfo {
                node_id: self.node_id.clone(),
                url: self.peer_url.clone(),
            });
        }
        true
    }

    pub fn update_peer_list(&self, new_peers: Vec<PeerInfo>) {
        let mut peers = self.peers.lock().unwrap();
        // 自分自身を除外してピアリストを更新
        *peers = new_peers.into_iter().filter(|p| p.node_id != self.node_id).collect();
    }

    pub fn broadcast_block(&self, _block: &str) {
        // 本来は各ピアのURLにHTTP POSTでブロックを送信
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            // ここでHTTPリクエスト等を送る（省略）
            // 例: reqwest::blocking::post(format!("{}/api/blocks/new", peer.url), ...)
            // 今回はダミー
        }
    }

    pub fn broadcast_transaction(&self, _tx: &str) {
        // 本来は各ピアのURLにHTTP POSTでトランザクションを送信
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            // ここでHTTPリクエスト等を送る（省略）
            // 例: reqwest::blocking::post(format!("{}/api/transactions/new", peer.url), ...)
            // 今回はダミー
        }
    }

    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_peer_lifecycle() {
        let mut p2p = P2P::new("https://bank.linglin.art", "node1", "http://localhost:8080");
        assert!(!p2p.is_running);
        p2p.start();
        assert!(p2p.is_running);
        p2p.stop();
        assert!(!p2p.is_running);
    }
    #[test]
    fn test_peer_list_update() {
        let p2p = P2P::new("https://bank.linglin.art", "node1", "http://localhost:8080");
        let peers = vec![PeerInfo { node_id: "n2".to_string(), url: "http://n2".to_string() }];
        p2p.update_peer_list(peers.clone());
        let got = p2p.get_peers();
        assert_eq!(got, peers);
    }

    #[test]
    fn test_register_with_primary_adds_self() {
        let p2p = P2P::new("https://bank.linglin.art", "nodeX", "http://localhost:9000");
        // peers list should not contain self at first
        assert!(p2p.get_peers().is_empty());
        p2p.register_with_primary();
        let peers = p2p.get_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, "nodeX");
        assert_eq!(peers[0].url, "http://localhost:9000");
    }

    #[test]
    fn test_update_peer_list_excludes_self() {
        let p2p = P2P::new("https://bank.linglin.art", "me", "http://me");
        let peers = vec![
            PeerInfo { node_id: "me".to_string(), url: "http://me".to_string() },
            PeerInfo { node_id: "other".to_string(), url: "http://other".to_string() },
        ];
        p2p.update_peer_list(peers.clone());
        let got = p2p.get_peers();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].node_id, "other");
    }

    #[test]
    fn test_broadcast_block_and_transaction_no_panic() {
        let p2p = P2P::new("https://bank.linglin.art", "n", "http://n");
        // Should not panic even if no peers
        p2p.broadcast_block("blockdata");
        p2p.broadcast_transaction("txdata");
        // Add a peer and test again
        p2p.update_peer_list(vec![PeerInfo { node_id: "p".to_string(), url: "http://p".to_string() }]);
        p2p.broadcast_block("blockdata");
        p2p.broadcast_transaction("txdata");
    }

    #[test]
    fn test_multiple_peer_add_remove() {
        let p2p = P2P::new("https://bank.linglin.art", "main", "http://main");
        let mut peers = vec![];
        for i in 0..5 {
            peers.push(PeerInfo { node_id: format!("n{}", i), url: format!("http://n{}", i) });
        }
        p2p.update_peer_list(peers.clone());
        let got = p2p.get_peers();
        assert_eq!(got.len(), 5);
        // Remove all
        p2p.update_peer_list(vec![]);
        assert!(p2p.get_peers().is_empty());
    }
}
