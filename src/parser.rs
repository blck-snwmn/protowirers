use anyhow::Result;
use std::convert::TryFrom;

use crate::zigzag;

/// parse_i64 parse variant' value as u32
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u32;
/// assert!(parse_u32(u128::MAX).is_err());
/// assert!(parse_u32((u32::MAX as u128) + 1).is_err());
/// assert_eq!(parse_u32(u32::MAX as u128).unwrap(), u32::MAX);
/// assert_eq!(parse_u32((u32::MAX - 1) as u128).unwrap(), u32::MAX - 1);
/// assert_eq!(parse_u32(0).unwrap(), 0);
/// ```
pub fn parse_u32(v: u128) -> Result<u32> {
    let u = TryFrom::try_from(v)?;
    Ok(u)
}

/// parse_i64 parse variant' value as u64
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u64;
/// assert!(parse_u64(u128::MAX).is_err());
/// assert!(parse_u64((u64::MAX as u128) + 1).is_err());
/// assert_eq!(parse_u64(u64::MAX as u128).unwrap(), u64::MAX);
/// assert_eq!(parse_u64((u64::MAX - 1) as u128).unwrap(), u64::MAX - 1);
/// assert_eq!(parse_u64(0).unwrap(), 0);
/// ```
pub fn parse_u64(v: u128) -> Result<u64> {
    let u = TryFrom::try_from(v)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i32 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i32;
/// assert!(parse_i32(u128::MAX).is_err());
/// assert!(parse_i32((u32::MAX as u128) + 1).is_err());
/// assert_eq!(parse_i32(u32::MAX as u128).unwrap(), i32::MIN);
/// assert_eq!(parse_i32((u32::MAX - 1) as u128).unwrap(), i32::MAX);
/// assert_eq!(parse_i32(0).unwrap(), 0);
/// assert_eq!(parse_i32(1).unwrap(), -1);
/// assert_eq!(parse_i32(2).unwrap(), 1);
/// ```
pub fn parse_i32(v: u128) -> Result<i32> {
    let decoded = zigzag::decode(v);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i64 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i64;
/// assert!(parse_i64(u128::MAX).is_err());
/// assert!(parse_i64((u64::MAX as u128) + 1).is_err());
/// assert_eq!(parse_i64(u64::MAX as u128).unwrap(), i64::MIN);
/// assert_eq!(parse_i64((u64::MAX - 1) as u128).unwrap(), i64::MAX);
/// assert_eq!(parse_i64(0).unwrap(), 0);
/// assert_eq!(parse_i64(1).unwrap(), -1);
/// assert_eq!(parse_i64(2).unwrap(), 1);
/// ```
pub fn parse_i64(v: u128) -> Result<i64> {
    let decoded = zigzag::decode(v);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

/// parse_i64 parse variant' value as String
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_string;
/// assert!(parse_string(vec![0xFF]).is_err());
/// assert_eq!(parse_string(vec![]).unwrap(), "");
/// assert_eq!(parse_string(vec![0x41]).unwrap(), "A");
/// ```
pub fn parse_string(v: Vec<u8>) -> Result<String> {
    let s = String::from_utf8(v)?;
    Ok(s)
}
