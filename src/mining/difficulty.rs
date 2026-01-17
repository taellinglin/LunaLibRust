
#[derive(Debug, Clone, Copy)]
pub struct Difficulty {
    pub value: u32,
}

impl Difficulty {
    pub fn new(value: u32) -> Self {
        Difficulty { value }
    }

    /// Returns the target string (e.g. "0000" for difficulty 4)
    pub fn target_string(&self) -> String {
        "0".repeat(self.value as usize)
    }

    /// Checks if a hash meets the difficulty target
    pub fn is_valid_hash(&self, hash: &str) -> bool {
        hash.starts_with(&self.target_string())
    }

    /// Adjusts difficulty based on block time (simple example)
    pub fn adjust(&self, last_block_time: f64, target_time: f64) -> Difficulty {
        let mut new_value = self.value;
        if last_block_time < target_time {
            new_value += 1;
        } else if last_block_time > target_time && new_value > 1 {
            new_value -= 1;
        }
        Difficulty { value: new_value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_string() {
        let diff = Difficulty::new(3);
        assert_eq!(diff.target_string(), "000");
    }

    #[test]
    fn test_is_valid_hash() {
        let diff = Difficulty::new(2);
        assert!(diff.is_valid_hash("00abcdef"));
        assert!(!diff.is_valid_hash("10abcdef"));
    }

    #[test]
    fn test_adjust_up() {
        let diff = Difficulty::new(4);
        let new_diff = diff.adjust(5.0, 10.0);
        assert_eq!(new_diff.value, 5);
    }

    #[test]
    fn test_adjust_down() {
        let diff = Difficulty::new(4);
        let new_diff = diff.adjust(15.0, 10.0);
        assert_eq!(new_diff.value, 3);
    }
}
