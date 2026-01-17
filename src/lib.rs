pub mod core;
pub mod mining;
pub mod gtx;
pub mod storage;
pub mod transactions;
pub mod utils;
pub mod luna_lib;

/// Main library struct exposing all LunaLib functionality
pub struct LunaLib;

impl LunaLib {
    pub fn version() -> &'static str {
        "0.1.0"
    }
    // Add more methods as needed
}
