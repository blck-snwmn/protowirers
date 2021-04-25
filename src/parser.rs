use crate::wire::{WireDataLengthDelimited, WireDataVarint};
use crate::zigzag;
use anyhow::Result;
use std::convert::TryFrom;

/// parse_i64 parse variant' value as u32
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u32;
/// # use protowirers::wire::WireDataVarint;
/// assert!(parse_u32(WireDataVarint::new(u128::MAX)).is_err());
/// assert!(parse_u32(WireDataVarint::new((u32::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_u32(WireDataVarint::new(u32::MAX as u128)).unwrap(), u32::MAX);
/// assert_eq!(parse_u32(WireDataVarint::new((u32::MAX - 1) as u128)).unwrap(), u32::MAX - 1);
/// assert_eq!(parse_u32(WireDataVarint::new(0)).unwrap(), 0);
/// ```
pub fn parse_u32(v: WireDataVarint) -> Result<u32> {
    let u = TryFrom::try_from(v.value)?;
    Ok(u)
}

/// parse_i64 parse variant' value as u64
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_u64;
/// # use protowirers::wire::WireDataVarint;
/// assert!(parse_u64(WireDataVarint::new(u128::MAX)).is_err());
/// assert!(parse_u64(WireDataVarint::new((u64::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_u64(WireDataVarint::new(u64::MAX as u128)).unwrap(), u64::MAX);
/// assert_eq!(parse_u64(WireDataVarint::new((u64::MAX - 1) as u128)).unwrap(), u64::MAX - 1);
/// assert_eq!(parse_u64(WireDataVarint::new(0)).unwrap(), 0);
/// ```
pub fn parse_u64(v: WireDataVarint) -> Result<u64> {
    let u = TryFrom::try_from(v.value)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i32 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i32;
/// # use protowirers::wire::WireDataVarint;
/// assert!(parse_i32(WireDataVarint::new(u128::MAX)).is_err());
/// assert!(parse_i32(WireDataVarint::new((u32::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_i32(WireDataVarint::new(u32::MAX as u128)).unwrap(), i32::MIN);
/// assert_eq!(parse_i32(WireDataVarint::new((u32::MAX - 1) as u128)).unwrap(), i32::MAX);
/// assert_eq!(parse_i32(WireDataVarint::new(0)).unwrap(), 0);
/// assert_eq!(parse_i32(WireDataVarint::new(1)).unwrap(), -1);
/// assert_eq!(parse_i32(WireDataVarint::new(2)).unwrap(), 1);
/// ```
pub fn parse_i32(v: WireDataVarint) -> Result<i32> {
    let decoded = zigzag::decode(v.value);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

/// parse_i64 parse variant' value as i64 using zigzag
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_i64;
/// # use protowirers::wire::WireDataVarint;
/// assert!(parse_i64(WireDataVarint::new(u128::MAX)).is_err());
/// assert!(parse_i64(WireDataVarint::new((u64::MAX as u128) + 1)).is_err());
/// assert_eq!(parse_i64(WireDataVarint::new(u64::MAX as u128)).unwrap(), i64::MIN);
/// assert_eq!(parse_i64(WireDataVarint::new((u64::MAX - 1) as u128)).unwrap(), i64::MAX);
/// assert_eq!(parse_i64(WireDataVarint::new(0)).unwrap(), 0);
/// assert_eq!(parse_i64(WireDataVarint::new(1)).unwrap(), -1);
/// assert_eq!(parse_i64(WireDataVarint::new(2)).unwrap(), 1);
/// ```
pub fn parse_i64(v: WireDataVarint) -> Result<i64> {
    let decoded = zigzag::decode(v.value);
    let u = TryFrom::try_from(decoded)?;
    Ok(u)
}

/// parse_i64 parse variant' value as String
///
/// # Example
/// ```rust
/// # use protowirers::parser::parse_string;
/// # use protowirers::wire::WireDataLengthDelimited;
/// assert!(parse_string(WireDataLengthDelimited::new(vec![0xFF])).is_err());
/// assert_eq!(parse_string(WireDataLengthDelimited::new(vec![])).unwrap(), "");
/// assert_eq!(parse_string(WireDataLengthDelimited::new(vec![0x41])).unwrap(), "A");
/// ```
pub fn parse_string(v: WireDataLengthDelimited) -> Result<String> {
    let s = String::from_utf8(v.value)?;
    Ok(s)
}

pub fn parse_vec_i64(v: Vec<u8>) -> Result<Vec<i64>> {
    let x = v.iter().map(|vv| *vv as i64).collect();
    Ok(x)
}

pub fn parse_vec_i32(v: Vec<u8>) -> Result<Vec<i32>> {
    let x = v.iter().map(|vv| *vv as i32).collect();
    Ok(x)
}

impl From<WireDataLengthDelimited> for Vec<i32> {
    fn from(v: WireDataLengthDelimited) -> Self {
        v.value.iter().map(|vv| *vv as i32).collect()
    }
}

pub trait From<T>: Sized {
    fn from(_: T) -> Self;
}

// pub fn parse_length_delimited<T>() -> Result<T> {}

trait Parser {
    fn parse<T>(v: WireDataLengthDelimited) -> Result<T>;
}
