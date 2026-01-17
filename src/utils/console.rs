use std::fmt;
use std::io::{self, Write};

pub enum ConsoleColor {
    Cyan,
    Yellow,
    Red,
    Green,
    Magenta,
    Reset,
}

impl ConsoleColor {
    fn to_ansi_code(&self) -> &'static str {
        match self {
            ConsoleColor::Cyan => "\x1b[36m",
            ConsoleColor::Yellow => "\x1b[33m",
            ConsoleColor::Red => "\x1b[31m",
            ConsoleColor::Green => "\x1b[32m",
            ConsoleColor::Magenta => "\x1b[35m",
            ConsoleColor::Reset => "\x1b[0m",
        }
    }
}

fn print_colored(msg: impl fmt::Display, color: ConsoleColor) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}{}{}\n", color.to_ansi_code(), msg, ConsoleColor::Reset.to_ansi_code());
}

pub struct Console;

impl Console {
    pub fn new() -> Self {
        Console
    }
    // TODO: Implement console utilities
}

pub fn print_info(msg: impl fmt::Display) {
    print_colored(msg, ConsoleColor::Cyan);
}

pub fn print_warn(msg: impl fmt::Display) {
    print_colored(msg, ConsoleColor::Yellow);
}

pub fn print_error(msg: impl fmt::Display) {
    print_colored(msg, ConsoleColor::Red);
}

pub fn print_success(msg: impl fmt::Display) {
    print_colored(msg, ConsoleColor::Green);
}

pub fn print_debug(msg: impl fmt::Display) {
    print_colored(msg, ConsoleColor::Magenta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_info() {
        print_info("Info message");
    }

    #[test]
    fn test_print_warn() {
        print_warn("Warning message");
    }

    #[test]
    fn test_print_error() {
        print_error("Error message");
    }

    #[test]
    fn test_print_success() {
        print_success("Success message");
    }

    #[test]
    fn test_print_debug() {
        print_debug("Debug message");
    }
}
