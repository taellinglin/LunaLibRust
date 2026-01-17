#[cfg(test)]
mod tests {
    use super::*;
    include!("wallet_tests.rs");
}

pub struct LunaWallet {
    pub address: String,
    pub public_key: String,
    pub encrypted_private_key: Vec<u8>,
    pub label: String,
    pub is_locked: bool,
    pub balance: f64,
    pub available_balance: f64,
    pub created: u64,
}


impl LunaWallet {
    pub fn new(address: String, public_key: String, encrypted_private_key: Vec<u8>, label: String, created: u64) -> Self {
        LunaWallet {
            address,
            public_key,
            encrypted_private_key,
            label,
            is_locked: true,
            balance: 0.0,
            available_balance: 0.0,
            created,
        }
    }
    // TODO: Implement create, unlock, export, import, info, balance, sign, verify, etc.
}
