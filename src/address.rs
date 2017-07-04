use crypto::digest::Digest;
use crypto::sha1::Sha1;
use num;
use std::fmt;
use rustc_serialize::hex;

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

/// An `Address` can be used to address anything that is `Addressable`. `Node`s, for example, have
/// an `Address`. Every `client::messages::TextMessage` has an `Address` as well.
#[derive(Clone,Copy,Eq,Hash,PartialEq)]
pub struct Address {
    data: [u32; 5]
}

impl Address {

    /// Hashes `content` into an `Address`. Current implementation is to take the SHA1 digest of
    /// `content`, but this is subject to change and should not be depended on.
    ///
    /// # Example
    ///
    /// ```
    /// let addr = address::Address::for_content("alep");
    /// assert_eq!(addr, address::Address::from_str("44751799925b964a00bae3863cc4236f9bb8d519"));
    /// ```
    pub fn for_content(content: &str) -> Address {
        let mut hasher = Sha1::new();
        hasher.input_str(content);
        let mut data = [0; 20];
        hasher.result(&mut data);
        Address {
            data: compact_bytes(&data)
        }
    }

    /// Creates an `Address` from its numeric representation. Useful for randomly generating
    /// `Address`es within a range.
    pub fn from_numeric(numeric: num::BigUint) -> Address {
        let data = numeric.to_bytes_be();
        Address {
            data: compact_bytes(data.as_slice())
        }
    }

    /// Creates a `Address` from its hexidecimal string representation. `string` must be
    /// hexidecimal or an `Err` will be returned.
    pub fn from_str(string: &str) -> Result<Address, hex::FromHexError> {
        use rustc_serialize::hex::FromHex;
        string.from_hex().map(|bytes| {
            let mut data = [0u8; 20];
            for (place, byte) in data.iter_mut().zip(bytes.iter()) {
                *place = *byte;
            }
            Address {
                data: compact_bytes(&data)
            }
        })
    }

    /// The null `Address`. Use to address "nothing." No `Node`, message, or any `Addressable`
    /// thing should ever reside at the null address. It's useful for bootstrapping a
    /// `network::Network` when one does not know the address of any peers, but has connection
    /// details to them such as an IP address and port.
    pub fn null() -> Address {
        Address {
            data: [0; 5]
        }
    }

    /// Randomly generates an `Address` within the address space between `min` and `max`. Useful
    /// for refreshing a `NodeBucket` by performing a find node operation on a random address
    /// within its range.
    pub fn random(min: &num::BigUint, max: &num::BigUint) -> Address {
        use rand;
        use num::bigint::RandBigInt;

        let mut rng = rand::StdRng::new().unwrap();
        let numeric = rng.gen_biguint_range(min, max);
        Self::from_numeric(numeric)
    }

    /// The numeric representation of an `Address`. Useful for partitioning the `Node`s in a
    /// `NodeBucket` into two new buckets.
    pub fn as_numeric(&self) -> num::BigUint {
        num::BigUint::new(self.data.to_vec())
    }

    /// The address space distance of `self` from `other`. Computed as the XOR of their numeric
    /// representations.
    pub fn distance_from(&self, other: &Self) -> num::BigUint {
        use std::ops::BitXor;
        self.as_numeric().bitxor(other.as_numeric())
    }

    /// The string representation of an `Address`. Useful for displaying, exporting outside of
    /// Rust, serializing into a protobuf, etc.
    ///
    /// TODO: this may be poorly named, breaking the `to_string` convention set by
    /// `&str::to_string`. It's probably wiser to `impl ToString`.
    pub fn to_str(&self) -> String {
        format!("{:040x}", self.as_numeric())
    }
}

/// Anything that needs to be uniquely addressed can implement `Addressable`.
pub trait Addressable {

    /// The `Address` where `self` resides.
    fn address(&self) -> Address;
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
        let address = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e").unwrap();
        assert_eq!(address.to_str(), "8b45e4bd1c6acb88bebf6407d16205f567e62a3e");
    }

    #[test]
    fn test_as_numeric() {
        use num::bigint::ToBigUint;

        let address = Address::from_str("000000000000000000000000000000000000000f").unwrap();
        assert_eq!(address.as_numeric(), 15u8.to_biguint().unwrap());
        let address = Address::from_str("00000000000000000000000000000000000000f0").unwrap();
        assert_eq!(address.as_numeric(), 240u8.to_biguint().unwrap());
    }

    #[test]
    fn test_equal() {
        let a = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e").unwrap();
        let b = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_not_equal() {
        let a = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3e").unwrap();
        let b = Address::from_str("8b45e4bd1c6acb88bebf6407d16205f567e62a3f").unwrap();
        assert!(a != b);
    }
}
