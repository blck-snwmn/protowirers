pub(crate) fn encode32(n: i128) -> u128 {
    ((n << 1) ^ (n >> 31)) as u128
}
pub(crate) fn encode64(n: i128) -> u128 {
    ((n << 1) ^ (n >> 63)) as u128
}
pub(crate) fn decode(n: u128) -> i128 {
    let r = (n >> 1) as i128;
    let l = (n & 1) as i128;
    (r ^ -l) as i128
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
