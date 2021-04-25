use crate::wire::*;
use crate::zigzag;
use anyhow::Result;
use std::convert::TryFrom;
use thiserror::Error;

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

#[derive(Error, Debug)]
enum ParseError {
    #[error("unexpected type. got={got}, want={want}")]
    UnexpectTypeError { want: String, got: String },
}

pub trait Parser<Output> {
    type Type;
    fn parse(&self, ty: Self::Type) -> Result<Output>;
}

impl Parser<String> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<String> {
        if !matches!(ty, TypeLengthDelimited::WireString) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeLengthDelimited::WireString},
                got: format! {"{:?}", ty},
            })?;
        }
        let s = String::from_utf8(self.value.clone())?;
        Ok(s)
    }
}

impl Parser<Vec<i64>> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<Vec<i64>> {
        if !matches!(ty, TypeLengthDelimited::PackedRepeatedFields) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeLengthDelimited::PackedRepeatedFields},
                got: format! {"{:?}", ty},
            })?;
        }
        let x = self.value.iter().map(|vv| *vv as i64).collect();
        Ok(x)
    }
}

impl Parser<Vec<i32>> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<Vec<i32>> {
        if !matches!(ty, TypeLengthDelimited::PackedRepeatedFields) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeLengthDelimited::PackedRepeatedFields},
                got: format! {"{:?}", ty},
            })?;
        }
        let x = self.value.iter().map(|vv| *vv as i32).collect();
        Ok(x)
    }
}

impl Parser<i32> for WireDataVarint {
    type Type = TypeVairant;
    fn parse(&self, ty: Self::Type) -> Result<i32> {
        match ty {
            TypeVairant::Int32 => {
                let u = TryFrom::try_from(self.value)?;
                Ok(u)
            }
            TypeVairant::Sint32 => {
                let decoded = zigzag::decode(self.value);
                let u = TryFrom::try_from(decoded)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int32, TypeVairant::Sint32},
                got: format! {"{:?}", ty},
            })?,
        }
    }
}

impl Parser<i64> for WireDataVarint {
    type Type = TypeVairant;
    fn parse(&self, ty: Self::Type) -> Result<i64> {
        match ty {
            TypeVairant::Int64 => {
                let u = TryFrom::try_from(self.value)?;
                Ok(u)
            }
            TypeVairant::Sint64 => {
                let decoded = zigzag::decode(self.value);
                let u = TryFrom::try_from(decoded)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int64, TypeVairant::Sint64},
                got: format! {"{:?}", ty},
            })?,
        }
    }
}

impl Parser<u32> for WireDataVarint {
    type Type = TypeVairant;
    fn parse(&self, ty: Self::Type) -> Result<u32> {
        if !matches!(ty, TypeVairant::Uint32) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint32},
                got: format! {"{:?}", ty},
            })?;
        }
        let u = TryFrom::try_from(self.value)?;
        Ok(u)
    }
}

impl Parser<u64> for WireDataVarint {
    type Type = TypeVairant;
    fn parse(&self, ty: Self::Type) -> Result<u64> {
        if !matches!(ty, TypeVairant::Uint64) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint64},
                got: format! {"{:?}", ty},
            })?;
        }
        let u = TryFrom::try_from(self.value)?;
        Ok(u)
    }
}

impl<T: Proto> Parser<T> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<T> {
        if !matches!(ty, TypeLengthDelimited::EmbeddedMessages) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}", TypeLengthDelimited::EmbeddedMessages},
                got: format! {"{:?}", ty},
            })?;
        }
        let r = T::parse(self.value.as_slice())?;
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_u32() {
        assert!(
            Parser::<u32>::parse(&WireDataVarint::new(u128::MAX), TypeVairant::Uint32).is_err()
        );
        assert!(Parser::<u32>::parse(
            &WireDataVarint::new((u32::MAX as u128) + 1),
            TypeVairant::Uint32
        )
        .is_err());
        assert_eq!(
            Parser::<u32>::parse(&WireDataVarint::new(u32::MAX as u128), TypeVairant::Uint32)
                .unwrap(),
            u32::MAX
        );
        assert_eq!(
            Parser::<u32>::parse(
                &WireDataVarint::new((u32::MAX - 1) as u128),
                TypeVairant::Uint32
            )
            .unwrap(),
            u32::MAX - 1
        );
        assert_eq!(
            Parser::<u32>::parse(&WireDataVarint::new(0), TypeVairant::Uint32).unwrap(),
            0
        );
    }

    #[test]
    fn parse_u64() {
        assert!(
            Parser::<u64>::parse(&WireDataVarint::new(u128::MAX), TypeVairant::Uint64).is_err()
        );
        assert!(Parser::<u64>::parse(
            &WireDataVarint::new((u64::MAX as u128) + 1),
            TypeVairant::Uint64
        )
        .is_err());
        assert_eq!(
            Parser::<u64>::parse(&WireDataVarint::new(u64::MAX as u128), TypeVairant::Uint64)
                .unwrap(),
            u64::MAX
        );
        assert_eq!(
            Parser::<u64>::parse(
                &WireDataVarint::new((u64::MAX - 1) as u128),
                TypeVairant::Uint64
            )
            .unwrap(),
            u64::MAX - 1
        );
        assert_eq!(
            Parser::<u64>::parse(&WireDataVarint::new(0), TypeVairant::Uint64).unwrap(),
            0
        );
    }

    #[test]
    fn parse_i32() {
        assert!(
            Parser::<i32>::parse(&WireDataVarint::new(u128::MAX), TypeVairant::Sint32).is_err()
        );
        assert!(Parser::<i32>::parse(
            &WireDataVarint::new((u32::MAX as u128) + 1),
            TypeVairant::Sint32
        )
        .is_err());
        assert_eq!(
            Parser::<i32>::parse(&WireDataVarint::new(u32::MAX as u128), TypeVairant::Sint32)
                .unwrap(),
            i32::MIN
        );
        assert_eq!(
            Parser::<i32>::parse(
                &WireDataVarint::new((u32::MAX - 1) as u128),
                TypeVairant::Sint32
            )
            .unwrap(),
            i32::MAX
        );
        assert_eq!(
            Parser::<i32>::parse(&WireDataVarint::new(0), TypeVairant::Sint32).unwrap(),
            0
        );
        assert_eq!(
            Parser::<i32>::parse(&WireDataVarint::new(1), TypeVairant::Sint32).unwrap(),
            -1
        );
        assert_eq!(
            Parser::<i32>::parse(&WireDataVarint::new(2), TypeVairant::Sint32).unwrap(),
            1
        );
    }

    #[test]
    fn parse_i64() {
        assert!(
            Parser::<i64>::parse(&WireDataVarint::new(u128::MAX), TypeVairant::Sint64).is_err()
        );
        assert!(Parser::<i64>::parse(
            &WireDataVarint::new((u64::MAX as u128) + 1),
            TypeVairant::Sint64
        )
        .is_err());
        assert_eq!(
            Parser::<i64>::parse(&WireDataVarint::new(u64::MAX as u128), TypeVairant::Sint64)
                .unwrap(),
            i64::MIN
        );
        assert_eq!(
            Parser::<i64>::parse(
                &WireDataVarint::new((u64::MAX - 1) as u128),
                TypeVairant::Sint64
            )
            .unwrap(),
            i64::MAX
        );
        assert_eq!(
            Parser::<i64>::parse(&WireDataVarint::new(0), TypeVairant::Sint64).unwrap(),
            0
        );
        assert_eq!(
            Parser::<i64>::parse(&WireDataVarint::new(1), TypeVairant::Sint64).unwrap(),
            -1
        );
        assert_eq!(
            Parser::<i64>::parse(&WireDataVarint::new(2), TypeVairant::Sint64).unwrap(),
            1
        );
    }
}
