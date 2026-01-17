// src/cli.rs
use clap::{Arg, Command};
use crate::luna_lib::LunaLib;

pub fn main() {
    let matches = Command::new("LunaLib Cryptocurrency Wallet")
        .arg(Arg::new("version")
            .long("version")
            .help("Show version")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    if matches.get_flag("version") {
        println!("LunaLib v{}", LunaLib::get_version());
    } else {
        println!("LunaLib - Use 'luna-wallet --help' for options");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_version_flag() {
        // Simulate --version argument
        let args = vec!["test-bin", "--version"];
        env::set_var("CLAP_TEST_ARGS", args.join(" "));
        // Should print version (mock LunaLib)
        // This is a smoke test; output is not captured
        main();
    }

    #[test]
    fn test_no_flag() {
        let args = vec!["test-bin"];
        env::set_var("CLAP_TEST_ARGS", args.join(" "));
        main();
    }
}
