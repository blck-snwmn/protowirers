use anyhow::Result;
use std::convert::TryFrom;

use crate::zigzag;

pub fn parse_u32(v: u128) -> Result<u32> {
    let u = TryFrom::try_from(v)?;
    Ok(u)
}

pub fn parse_u64(v: u128) -> Result<u64> {
    let u = TryFrom::try_from(v)?;
    Ok(u)
}

pub fn parse_i32(v: u128) -> Result<i32> {
    let decoded = zigzag::decode(v);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

pub fn parse_i64(v: u128) -> Result<i64> {
    let decoded = zigzag::decode(v);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

#[cfg(test)]
mod tests {
    use std::u128;

    use super::*;
    #[test]
    fn test_parse_u32() {
        assert!(parse_u32(u128::MAX).is_err());
        assert!(parse_u32((u32::MAX as u128) + 1).is_err());
        assert_eq!(parse_u32(u32::MAX as u128).unwrap(), u32::MAX);
        assert_eq!(parse_u32((u32::MAX - 1) as u128).unwrap(), u32::MAX - 1);
        assert_eq!(parse_u32(0).unwrap(), 0);
    }
    #[test]
    fn test_parse_u64() {
        assert!(parse_u64(u128::MAX).is_err());
        assert!(parse_u64((u64::MAX as u128) + 1).is_err());
        assert_eq!(parse_u64(u64::MAX as u128).unwrap(), u64::MAX);
        assert_eq!(parse_u64((u64::MAX - 1) as u128).unwrap(), u64::MAX - 1);
        assert_eq!(parse_u64(0).unwrap(), 0);
    }
    #[test]
    fn test_parse_i32() {
        assert!(parse_i32(u128::MAX).is_err());
        assert!(parse_i32((u32::MAX as u128) + 1).is_err());
        assert_eq!(parse_i32(u32::MAX as u128).unwrap(), i32::MIN);
        assert_eq!(parse_i32((u32::MAX - 1) as u128).unwrap(), i32::MAX);
        assert_eq!(parse_i32(0).unwrap(), 0);
        assert_eq!(parse_i32(1).unwrap(), -1);
        assert_eq!(parse_i32(2).unwrap(), 1);
    }
    #[test]
    fn test_parse_i64() {
        assert!(parse_i64(u128::MAX).is_err());
        assert!(parse_i64((u64::MAX as u128) + 1).is_err());
        assert_eq!(parse_i64(u64::MAX as u128).unwrap(), i64::MIN);
        assert_eq!(parse_i64((u64::MAX - 1) as u128).unwrap(), i64::MAX);
        assert_eq!(parse_i64(0).unwrap(), 0);
        assert_eq!(parse_i64(1).unwrap(), -1);
        assert_eq!(parse_i64(2).unwrap(), 1);
    }
}
