// Basic tests for LunaWallet struct
use super::LunaWallet;

#[test]
fn test_wallet_creation() {
    let wallet = LunaWallet::new(
        "LUN_testaddress".to_string(),
        "testpubkey".to_string(),
        vec![1,2,3,4],
        "Test Wallet".to_string(),
        1234567890,
    );
    assert_eq!(wallet.address, "LUN_testaddress");
    assert_eq!(wallet.public_key, "testpubkey");
    assert_eq!(wallet.label, "Test Wallet");
    assert!(wallet.is_locked);
    assert_eq!(wallet.balance, 0.0);
    assert_eq!(wallet.available_balance, 0.0);
    assert_eq!(wallet.created, 1234567890);
}
