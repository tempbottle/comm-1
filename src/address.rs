use crypto::digest::Digest;
use crypto::sha1::Sha1;
use num;
use std::fmt;

pub const LENGTH: usize = 160;

#[derive(Clone,Copy,Eq,Hash,PartialEq)]
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

    pub fn from_numeric(numeric: num::BigUint) -> Address {
        let bytes = numeric.to_bytes_be();
        let mut data = [0; 20];
        for (place, byte) in data.iter_mut().rev().zip(bytes.iter().rev()).rev() {
            *place = *byte
        }
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

    pub fn null() -> Address {
        let data = [0; 20];
        Address { data: data }
    }

    pub fn as_numeric(&self) -> num::BigUint {
        // TODO: Expensive
        num::BigUint::from_bytes_be(&self.data)
    }

    pub fn distance_from(&self, other: &Self) -> num::BigUint {
        use std::ops::BitXor;
        self.as_numeric().bitxor(other.as_numeric())
    }

    pub fn to_str(&self) -> String {
        use rustc_serialize::hex::ToHex;
        self.data.to_hex()
    }
}

pub trait Addressable {
    fn get_address(&self) -> Address;
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address {{ {} }}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use num;
    use super::{Address, LENGTH};

    #[test]
    fn test_for_content() {
        let address = Address::for_content("some string");
        assert_eq!(address.to_str(), "8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
    }

    #[test]
    fn test_from_numeric() {
        use num::bigint::ToBigUint;

        let numeric = 0x0000000000000000000000000000000000000001.to_biguint().unwrap();
        let address = Address::from_numeric(numeric);
        assert_eq!(address.as_numeric(), 0x0000000000000000000000000000000000000001.to_biguint().unwrap());
        assert_eq!(address.to_str(), "0000000000000000000000000000000000000001");

        let numeric = num::pow(2.to_biguint().unwrap(), 158) -
            num::pow(2.to_biguint().unwrap(), 157) -
            num::pow(2.to_biguint().unwrap(), 156);
        let address = Address::from_numeric(numeric.clone());
        assert_eq!(address.as_numeric(), numeric);
        assert_eq!(address.to_str(), "1000000000000000000000000000000000000000");

        let numeric = num::pow(2.to_biguint().unwrap(), LENGTH) - 1.to_biguint().unwrap();
        let address = Address::from_numeric(numeric);
        assert_eq!(address.as_numeric(), num::pow(2.to_biguint().unwrap(), LENGTH) - 1.to_biguint().unwrap());
        assert_eq!(address.to_str(), "ffffffffffffffffffffffffffffffffffffffff");
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
