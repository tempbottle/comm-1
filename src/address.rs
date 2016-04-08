use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::iter::repeat;
use num;

pub const LENGTH: usize = 160;

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
pub struct Address {
    data: [u8; 20]
}

impl Address {
    pub fn for_content(content: &str) -> Address {
        let mut hasher = Sha1::new();
        hasher.input_str(content);
        let mut data = [0; 20];
        hasher.result(&mut data);
        Address {
            data: data
        }
    }

    pub fn from_str(string: &str) -> Address {
        use rustc_serialize::hex::FromHex;
        let bytes = string.from_hex().unwrap();
        let mut data = [0u8; 20];
        for (place, byte) in data.iter_mut().zip(bytes.iter()) {
            *place = *byte;
        }
        Address {
            data: data
        }
    }

    pub fn as_numeric(&self) -> num::BigUint {
        num::BigUint::from_bytes_be(&self.data)
    }

    pub fn to_str(&self) -> String {
        use rustc_serialize::hex::ToHex;
        self.data.to_hex()
    }
}

pub trait Addressable {
    fn address(&self) -> Address;
}

#[cfg(test)]
mod tests {
    use super::Address;

    #[test]
    fn test_for_content() {
        let address = Address::for_content("some string");
        assert_eq!(address.to_str(), "8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
    }

    #[test]
    fn test_from_str() {
        let address = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
        assert_eq!(address.to_str(), "8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
    }

    #[test]
    fn test_as_numeric() {
        use num::bigint::ToBigUint;

        let address = Address::from_str("000000000000000000000000000000000000000f");
        assert_eq!(address.as_numeric(), 15u8.to_biguint().unwrap());
        let address = Address::from_str("00000000000000000000000000000000000000f0");
        assert_eq!(address.as_numeric(), 240u8.to_biguint().unwrap());
    }

    #[test]
    fn test_equal() {
        let a = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
        let b = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
        assert_eq!(a, b);
    }

    #[test]
    fn test_not_equal() {
        let a = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
        let b = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3f");
        assert!(a != b);
    }
}
