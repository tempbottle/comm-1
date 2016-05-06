pub type TransactionId = u32;

pub struct TransactionIdGenerator {
    next_id: TransactionId
}

impl TransactionIdGenerator {
    pub fn new() -> TransactionIdGenerator {
        TransactionIdGenerator { next_id: 0 }
    }

    pub fn generate(&mut self) -> TransactionId {
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
