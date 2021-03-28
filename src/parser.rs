use anyhow::Result;
use std::convert::TryFrom;

pub fn parse_u32(v: u128) -> Result<u32> {
    let u = TryFrom::try_from(v)?;
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
}
