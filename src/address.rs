use crypto::digest::Digest;
use crypto::sha1::Sha1;
use num;
use std::fmt;

pub const LENGTH: usize = 160;

fn compact_bytes(data: &[u8]) -> [u32; 5] {
    let mut compacted = [0; 5];
    for (compact, bytes) in compacted.iter_mut().zip(data.chunks(4).rev()) {
        *compact = bytes.iter()
            .rev()
            .enumerate()
            .fold(0u32, |sum, (i, &b)| sum + ((b as u32) << (8 * i)));
    }
    compacted
}

#[derive(Clone,Copy,Eq,Hash,PartialEq)]
pub struct Address {
    data: [u32; 5]
}

impl Address {
    pub fn for_content(content: &str) -> Address {
        let mut hasher = Sha1::new();
        hasher.input_str(content);
        let mut data = [0; 20];
        hasher.result(&mut data);
        Address {
            data: compact_bytes(&data)
        }
    }

    pub fn from_numeric(numeric: num::BigUint) -> Address {
        let data = numeric.to_bytes_be();
        Address {
            data: compact_bytes(data.as_slice())
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
            data: compact_bytes(&data)
        }
    }

    pub fn null() -> Address {
        Address {
            data: [0; 5]
        }
    }

    pub fn random(min: &num::BigUint, max: &num::BigUint) -> Address {
        use rand;
        use num::bigint::RandBigInt;

        let mut rng = rand::StdRng::new().unwrap();
        let numeric = rng.gen_biguint_range(min, max);
        Self::from_numeric(numeric)
    }

    pub fn as_numeric(&self) -> num::BigUint {
        num::BigUint::new(self.data.to_vec())
    }

    pub fn distance_from(&self, other: &Self) -> num::BigUint {
        use std::ops::BitXor;
        self.as_numeric().bitxor(other.as_numeric())
    }

    pub fn to_str(&self) -> String {
        format!("{:040x}", self.as_numeric())
    }
}

pub trait Addressable {
    fn addresss(&self) -> Address;
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address {{ {} }}", self.to_str())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.to_str())
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
