use crate::{decode::decode_variants_slice, wire::*};
use crate::{encode::encode_repeat, zigzag};
use anyhow::Result;
use std::convert::TryFrom;
use thiserror::Error;

pub trait VariantToValue: Sized {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self>;
    fn to_variant(&self, ty: TypeVairant) -> Result<u128>;
}

impl VariantToValue for i32 {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self> {
        match ty {
            TypeVairant::Int32 => {
                // マイナス値の場合、i32 であっても i64 と同様のバイト数を消費する必要があるので、i64として処理させる
                let result = i64::from_valint(input, TypeVairant::Int64)?;
                let u = TryFrom::try_from(result)?;
                Ok(u)
            }
            TypeVairant::Sint32 => {
                let decoded = zigzag::decode(input);
                let u = TryFrom::try_from(decoded)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int32, TypeVairant::Sint32},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn to_variant(&self, ty: TypeVairant) -> Result<u128> {
        match ty {
            TypeVairant::Int32 => {
                let x: i64 = (*self).into();
                VariantToValue::to_variant(&x, TypeVairant::Int64)
            }
            TypeVairant::Sint32 => Ok(zigzag::encode(*self) as u128),
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int32, TypeVairant::Sint32},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}

impl VariantToValue for i64 {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self> {
        match ty {
            TypeVairant::Int64 => {
                if input > u64::MAX as u128 {
                    return Err(anyhow::anyhow!(
                        "unexpected value. this value is greater than {}(u64::MAX)",
                        u64::MAX
                    ));
                }
                Ok(input as i64)
            }
            TypeVairant::Sint64 => {
                let decoded = zigzag::decode(input);
                let u = TryFrom::try_from(decoded)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int64, TypeVairant::Sint64},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn to_variant(&self, ty: TypeVairant) -> Result<u128> {
        match ty {
            // u64に一度キャストすることにより、内部の保持しているバイト数を64bitに合わせる
            TypeVairant::Int64 => Ok(*self as u64 as u128),
            TypeVairant::Sint64 => Ok(zigzag::encode(*self) as u128),
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?} or {:?}",TypeVairant::Int64, TypeVairant::Sint64},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}

impl VariantToValue for u32 {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self> {
        match ty {
            TypeVairant::Uint32 => {
                let u = TryFrom::try_from(input)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint32},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn to_variant(&self, ty: TypeVairant) -> Result<u128> {
        match ty {
            TypeVairant::Uint32 => Ok(*self as u128),
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint32},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}

impl VariantToValue for u64 {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self> {
        match ty {
            TypeVairant::Uint64 => {
                let u = TryFrom::try_from(input)?;
                Ok(u)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint64},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn to_variant(&self, ty: TypeVairant) -> Result<u128> {
        match ty {
            TypeVairant::Uint64 => Ok(*self as u128),
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Uint64},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}

impl VariantToValue for bool {
    fn from_valint(input: u128, ty: TypeVairant) -> Result<Self> {
        match ty {
            TypeVairant::Bool => Ok(input != 0),
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Bool},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn to_variant(&self, ty: TypeVairant) -> Result<u128> {
        match ty {
            TypeVairant::Bool => {
                if *self {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeVairant::Bool},
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("unexpected type. got={got}, want={want}")]
    UnexpectTypeError { want: String, got: String },
}

pub trait Parser<Output>: Sized {
    type Type;
    fn parse(&self, ty: Self::Type) -> Result<Output>;
    // TODO rename
    fn from(input: Output, ty: Self::Type) -> Result<Self>;
}

impl Parser<String> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<String> {
        if !matches!(ty, TypeLengthDelimited::WireString) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeLengthDelimited::WireString},
                got: format! {"{:?}", ty},
            }
            .into());
        }
        let s = String::from_utf8(self.value.clone())?;
        Ok(s)
    }

    fn from(input: String, ty: Self::Type) -> Result<Self> {
        if !matches!(ty, TypeLengthDelimited::WireString) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}",TypeLengthDelimited::WireString},
                got: format! {"{:?}", ty},
            }
            .into());
        }
        Ok(Self {
            value: (input.into()),
        })
    }
}

impl<T: VariantToValue> Parser<Vec<T>> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<Vec<T>> {
        match ty {
            TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(v)) => {
                let x = decode_variants_slice(&self.value)?;
                let x = x
                    .iter()
                    .try_fold(Vec::with_capacity(x.len()), |mut acc, xx| {
                        T::from_valint(*xx, v).map(|x| {
                            acc.push(x);
                            acc
                        })
                    })?;
                Ok(x)
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: "TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant())"
                    .to_string(),
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }

    fn from(input: Vec<T>, ty: Self::Type) -> Result<Self> {
        match ty {
            TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(tv)) => {
                let input = input.iter().try_fold(Vec::new(), |mut acc, x| {
                    x.to_variant(tv).map(|x| {
                        acc.push(x);
                        acc
                    })
                })?;
                let mut v = Vec::new();
                encode_repeat(&mut v, input)?;
                Ok(Self { value: v })
            }
            _ => Err(ParseError::UnexpectTypeError {
                want: "TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant())"
                    .to_string(),
                got: format! {"{:?}", ty},
            }
            .into()),
        }
    }
}
impl<T: Proto> Parser<T> for WireDataLengthDelimited {
    type Type = TypeLengthDelimited;
    fn parse(&self, ty: Self::Type) -> Result<T> {
        if !matches!(ty, TypeLengthDelimited::EmbeddedMessages) {
            return Err(ParseError::UnexpectTypeError {
                want: format! {"{:?}", TypeLengthDelimited::EmbeddedMessages},
                got: format! {"{:?}", ty},
            }
            .into());
        }
        let r = T::parse(self.value.as_slice())?;
        Ok(r)
    }

    fn from(input: T, _: Self::Type) -> Result<Self> {
        Ok(Self {
            value: input.bytes()?,
        })
    }
}

impl<T: VariantToValue> Parser<T> for WireDataVarint {
    type Type = TypeVairant;
    fn parse(&self, ty: Self::Type) -> Result<T> {
        T::from_valint(self.value, ty)
    }

    fn from(input: T, ty: Self::Type) -> Result<Self> {
        Ok(Self {
            value: input.to_variant(ty)?,
        })
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

    #[test]
    fn parse_bool() {
        assert!(
            Parser::<bool>::parse(&WireDataVarint::new(u128::MAX), TypeVairant::Sint64).is_err()
        );
        assert!(Parser::<i64>::parse(
            &WireDataVarint::new((u64::MAX as u128) + 1),
            TypeVairant::Bool
        )
        .is_err());
        assert!(!Parser::<bool>::parse(&WireDataVarint::new(0), TypeVairant::Bool).unwrap());
        assert!(Parser::<bool>::parse(&WireDataVarint::new(1), TypeVairant::Bool).unwrap());
        assert!(Parser::<bool>::parse(&WireDataVarint::new(2), TypeVairant::Bool).unwrap());
    }

    #[test]
    fn parse_string() {
        assert!(Parser::<String>::parse(
            &WireDataLengthDelimited::new(vec![0xFF]),
            TypeLengthDelimited::WireString,
        )
        .is_err());
        assert_eq!(
            Parser::<String>::parse(
                &WireDataLengthDelimited::new(vec![]),
                TypeLengthDelimited::WireString,
            )
            .unwrap(),
            ""
        );
        assert_eq!(
            Parser::<String>::parse(
                &WireDataLengthDelimited::new(vec![0x41, 0x41, 0x41]),
                TypeLengthDelimited::WireString,
            )
            .unwrap(),
            "AAA"
        );
    }

    #[test]
    fn parse_vec() {
        {
            // i32
            assert_eq!(
                Parser::<Vec<i32>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                        0b11111111, 0b11111111, 0b11111111, 0b00000001,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Int32
                    ))
                )
                .unwrap(),
                vec![1, 2, 1000, 4, 5, -1]
            );
            assert!(Parser::<Vec<i32>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00001111,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Int32
                ))
            )
            .is_err());
        }
        {
            //i64
            assert_eq!(
                Parser::<Vec<i64>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                        0b11111111, 0b11111111, 0b11111111, 0b00000001,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Int64
                    ))
                )
                .unwrap(),
                vec![1, 2, 1000, 4, 5, -1]
            );
            assert!(Parser::<Vec<i64>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Int64
                ))
            )
            .is_err());
        }
        {
            assert_eq!(
                Parser::<Vec<u32>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Uint32
                    ))
                )
                .unwrap(),
                vec![1, 2, 1000, 4, 5]
            );
            assert!(Parser::<Vec<u32>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b10000001, 0b10000010, 0b11101000, 0b10000111, 0b10000100, 0b10000101,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Uint32
                ))
            )
            .is_err());
        }
        {
            //u64
            assert_eq!(
                Parser::<Vec<u64>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Uint64
                    ))
                )
                .unwrap(),
                vec![1, 2, 1000, 4, 5]
            );
            assert!(Parser::<Vec<u64>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001, 0b10000001,
                    0b10000001, 0b10000001, 0b10000001, 0b00000010, 0b11101000, 0b00000111,
                    0b00000100, 0b00000101,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Uint64
                ))
            )
            .is_err());
        }
        {
            // i64 for zigzag
            assert_eq!(
                Parser::<Vec<i64>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b11010000, 0b00001111, 0b11111110, 0b11111111, 0b11111111, 0b11111111,
                        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001,
                        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                        0b11111111, 0b11111111, 0b11111111, 0b00000001,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Sint64
                    ))
                )
                .unwrap(),
                vec![1000, i64::MAX, i64::MIN]
            );
            assert!(Parser::<Vec<i64>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b11010000, 0b00001111, 0b11111110, 0b11111111, 0b11111111, 0b11111111,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                    0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Sint64
                ))
            )
            .is_err());
        }
        {
            // i32 for zigzag
            assert_eq!(
                Parser::<Vec<i32>>::parse(
                    &WireDataLengthDelimited::new(vec![
                        0b11010000, 0b00001111, 0b11111110, 0b11111111, 0b11111111, 0b11111111,
                        0b00001111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00001111,
                    ]),
                    TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                        TypeVairant::Sint32
                    ))
                )
                .unwrap(),
                vec![1000, i32::MAX, i32::MIN]
            );
            assert!(Parser::<Vec<i32>>::parse(
                &WireDataLengthDelimited::new(vec![
                    0b11010000, 0b00001111, 0b11111110, 0b11111111, 0b11111111, 0b11111111,
                    0b00001111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                    0b00001111,
                ]),
                TypeLengthDelimited::PackedRepeatedFields(AllowedPakcedType::Variant(
                    TypeVairant::Sint32
                ))
            )
            .is_err(),);
        }
    }
}
