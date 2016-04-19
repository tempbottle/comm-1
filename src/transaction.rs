pub struct TransactionIdGenerator {
    next_id: u32
}

impl TransactionIdGenerator {
    pub fn new() -> TransactionIdGenerator {
        TransactionIdGenerator { next_id: 0 }
    }

    pub fn generate(&mut self) -> u32 {
        self.next_id = self.next_id.wrapping_add(1);
        self.next_id
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionIdGenerator;

    #[test]
    fn test_generate_id() {
        let mut generator = TransactionIdGenerator::new();
        assert_eq!(generator.generate(), 1);
        assert_eq!(generator.generate(), 2);
    }
}
