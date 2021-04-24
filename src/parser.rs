use crate::wire::WireTypeVarint;
use crate::zigzag;
use anyhow::Result;
use std::convert::TryFrom;

/// parse_i64 parse variant' value as u32
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u32;
/// # use protowirers::wire::WireTypeVarint;
/// assert!(parse_u32(WireTypeVarint::new(u128::MAX)).is_err());
/// assert!(parse_u32(WireTypeVarint::new((u32::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_u32(WireTypeVarint::new(u32::MAX as u128)).unwrap(), u32::MAX);
/// assert_eq!(parse_u32(WireTypeVarint::new((u32::MAX - 1) as u128)).unwrap(), u32::MAX - 1);
/// assert_eq!(parse_u32(WireTypeVarint::new(0)).unwrap(), 0);
/// ```
pub fn parse_u32(v: WireTypeVarint) -> Result<u32> {
    let u = TryFrom::try_from(v.value)?;
    Ok(u)
}

/// parse_i64 parse variant' value as u64
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u64;
/// # use protowirers::wire::WireTypeVarint;
/// assert!(parse_u64(WireTypeVarint::new(u128::MAX)).is_err());
/// assert!(parse_u64(WireTypeVarint::new((u64::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_u64(WireTypeVarint::new(u64::MAX as u128)).unwrap(), u64::MAX);
/// assert_eq!(parse_u64(WireTypeVarint::new((u64::MAX - 1) as u128)).unwrap(), u64::MAX - 1);
/// assert_eq!(parse_u64(WireTypeVarint::new(0)).unwrap(), 0);
/// ```
pub fn parse_u64(v: WireTypeVarint) -> Result<u64> {
    let u = TryFrom::try_from(v.value)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i32 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i32;
/// # use protowirers::wire::WireTypeVarint;
/// assert!(parse_i32(WireTypeVarint::new(u128::MAX)).is_err());
/// assert!(parse_i32(WireTypeVarint::new((u32::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_i32(WireTypeVarint::new(u32::MAX as u128)).unwrap(), i32::MIN);
/// assert_eq!(parse_i32(WireTypeVarint::new((u32::MAX - 1) as u128)).unwrap(), i32::MAX);
/// assert_eq!(parse_i32(WireTypeVarint::new(0)).unwrap(), 0);
/// assert_eq!(parse_i32(WireTypeVarint::new(1)).unwrap(), -1);
/// assert_eq!(parse_i32(WireTypeVarint::new(2)).unwrap(), 1);
/// ```
pub fn parse_i32(v: WireTypeVarint) -> Result<i32> {
    let decoded = zigzag::decode(v.value);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i64 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i64;
/// # use protowirers::wire::WireTypeVarint;
/// assert!(parse_i64(WireTypeVarint::new(u128::MAX)).is_err());
/// assert!(parse_i64(WireTypeVarint::new((u64::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_i64(WireTypeVarint::new(u64::MAX as u128)).unwrap(), i64::MIN);
/// assert_eq!(parse_i64(WireTypeVarint::new((u64::MAX - 1) as u128)).unwrap(), i64::MAX);
/// assert_eq!(parse_i64(WireTypeVarint::new(0)).unwrap(), 0);
/// assert_eq!(parse_i64(WireTypeVarint::new(1)).unwrap(), -1);
/// assert_eq!(parse_i64(WireTypeVarint::new(2)).unwrap(), 1);
/// ```
pub fn parse_i64(v: WireTypeVarint) -> Result<i64> {
    let decoded = zigzag::decode(v.value);
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

pub fn parse_vec_i64(v: Vec<u8>) -> Result<Vec<i64>> {
    let x = v.iter().map(|vv| *vv as i64).collect();
    Ok(x)
}

pub fn parse_vec_i32(v: Vec<u8>) -> Result<Vec<i64>> {
    let x = v.iter().map(|vv| *vv as i64).collect();
    Ok(x)
}
