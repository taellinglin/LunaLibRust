# LunaLibRust

Rust implementation of LunaLib: a cryptocurrency wallet and mining system.

## Features
- Wallet management
- Mining operations
- Blockchain management
- Transaction handling

## Usage
Add this to your `Cargo.toml`:
```toml
dependency = "lunalib_rust"
```

## License
MIT OR Apache-2.0

## Quick Start Tutorial

Below are examples of how to initialize and use each major module in LunaLibRust.

### 1. Wallet Management
```rust
use lunalib_rust::luna_lib::create_wallet;

let wallet = create_wallet();
// Use wallet methods, e.g. wallet.get_address(), wallet.get_balance(), etc.
```

### 2. Mining Operations
```rust
use lunalib_rust::luna_lib::create_miner;

let miner = create_miner();
// Use miner methods, e.g. miner.start(), miner.stop(), etc.
```

### 3. Blockchain Management
```rust
use lunalib_rust::luna_lib::create_blockchain_manager;

let blockchain = create_blockchain_manager(Some("https://bank.linglin.art"));
// Use blockchain methods, e.g. blockchain.get_height(), blockchain.get_block(), etc.
```

### 4. Mempool Management
```rust
use lunalib_rust::luna_lib::create_mempool_manager;

let mempool = create_mempool_manager(None);
// Use mempool methods, e.g. mempool.add_transaction(), mempool.get_pending(), etc.
```

### 5. Transaction Handling
```rust
use lunalib_rust::luna_lib::get_transaction_manager;

let tx_manager = get_transaction_manager();
// Use tx_manager methods, e.g. tx_manager.create_transfer(), tx_manager.validate_transaction(), etc.
```

### 6. Version and Class Info
```rust
use lunalib_rust::luna_lib::LunaLib;

println!("LunaLib version: {}", LunaLib::get_version());
for (name, desc) in LunaLib::get_available_classes() {
    println!("{}: {}", name, desc);
}
```

---

For more details, see the documentation for each struct and method.