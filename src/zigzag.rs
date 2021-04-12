use std::{
    mem::{size_of, size_of_val},
    ops::{Shl, Shr},
};

pub(crate) fn encode32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}
pub(crate) fn encode64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

pub(crate) fn encode<T: ZigZag>(n: T) -> T::Output {
    n.encode()
}

pub(crate) fn decode(n: u128) -> i128 {
    let r = (n >> 1) as i128;
    let l = (n & 1) as i128;
    (r ^ -l) as i128
}

pub trait ZigZag {
    type Output;
    fn encode(&self) -> Self::Output;
}

impl ZigZag for i32 {
    type Output = u32;
    fn encode(&self) -> Self::Output {
        ((self << 1) ^ (self >> 31)) as u32
    }
}

impl ZigZag for i64 {
    type Output = u64;
    fn encode(&self) -> Self::Output {
        ((self << 1) ^ (self >> 63)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode32() {
        assert_eq!(encode32(0), 0);
        assert_eq!(encode32(-1), 1);
        assert_eq!(encode32(1), 2);
        assert_eq!(encode32(-2), 3);
        assert_eq!(encode32(2), 4);
        assert_eq!(encode32(2147483647), 4294967294);
        assert_eq!(encode32(-2147483648), 4294967295);
    }
    #[test]
    fn test_encode64() {
        assert_eq!(encode64(0), 0);
        assert_eq!(encode64(-1), 1);
        assert_eq!(encode64(1), 2);
        assert_eq!(encode64(-2), 3);
        assert_eq!(encode64(2), 4);
        assert_eq!(encode64(2147483647), 4294967294);
        assert_eq!(encode64(-9223372036854775808), 18446744073709551615);
        assert_eq!(encode64(9223372036854775807), 18446744073709551614);
    }

    #[test]
    fn test_encode() {
        assert_eq!(encode(0), 0 as u32);
        assert_eq!(encode(-1), 1 as u32);
        assert_eq!(encode(1), 2 as u32);
        assert_eq!(encode(-2), 3 as u32);
        assert_eq!(encode(2), 4 as u32);
        assert_eq!(encode(2147483647), 4294967294 as u32);
        assert_eq!(encode(-2147483648), 4294967295 as u32);
        assert_eq!(encode(2147483647 as i32), 4294967294);
        assert_eq!(encode(-2147483648 as i32), 4294967295);
        assert_eq!(encode(-9223372036854775808 as i64), 18446744073709551615);
        assert_eq!(encode(9223372036854775807 as i64), 18446744073709551614);
    }
    #[test]
    fn test_decode_zigzag() {
        assert_eq!(decode(0), 0);
        assert_eq!(decode(1), -1);
        assert_eq!(decode(2), 1);
        assert_eq!(decode(3), -2);
        assert_eq!(decode(4), 2);
        assert_eq!(decode(4294967294), 2147483647);
        assert_eq!(decode(4294967295), -2147483648);
    }
}
