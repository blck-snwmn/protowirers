use std::fmt::Display;

use crate::zigzag::ZigZag;

// alias　ではなく、タプル構造体にしたほうがよさそう
pub type FieldNumber = u128;

#[derive(Debug, PartialEq, Eq)]
pub struct WireStruct {
    field_number: FieldNumber,
    wire_type: WireType,
}

impl WireStruct {
    pub fn field_number(&self) -> FieldNumber {
        self.field_number
    }
    pub fn wire_type(&self) -> WireType {
        self.wire_type.clone()
    }
    pub fn new(field_number: FieldNumber, wire_type: WireType) -> Self {
        WireStruct {
            field_number,
            wire_type,
        }
    }
    fn from_in<T, U>(field_number: FieldNumber, data: T) -> Self
    where
        T: ZigZag<Output = U>,
        U: Into<u128>,
    {
        let data: u128 = data.encode().into();
        Self::new(field_number, WireType::Varint(data))
    }
    pub fn from_i32(field_number: FieldNumber, data: i32) -> Self {
        Self::from_in(field_number, data)
    }
    pub fn from_i64(field_number: FieldNumber, data: i64) -> Self {
        Self::from_in(field_number, data)
    }
    pub fn from_u32(field_number: FieldNumber, data: u32) -> Self {
        Self::new(field_number, WireType::Varint(data as u128))
    }
    pub fn from_u64(field_number: FieldNumber, data: u64) -> Self {
        Self::new(field_number, WireType::Varint(data as u128))
    }
    pub fn from_string(field_number: FieldNumber, data: String) -> Self {
        let data = Vec::from(data);
        Self::new(field_number, WireType::LengthDelimited(data))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WireType {
    Varint(u128),
    Bit64([u8; 8]),
    LengthDelimited(Vec<u8>),
    // StartGroup,
    // EndGroup,
    Bit32([u8; 4]),
}

impl WireType {
    pub fn type_number(&self) -> u128 {
        match &self {
            WireType::Varint(_) => 0,
            WireType::Bit64(_) => 1,
            WireType::LengthDelimited(_) => 2,
            WireType::Bit32(_) => 5,
        }
    }
}

impl Display for WireType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WireType::Varint(v) => write!(f, "Varint{}", v),
            WireType::Bit64(v) => write!(f, "Bit64{:?}", v),
            WireType::LengthDelimited(v) => write!(f, "LengthDelimited{:?}", v),
            WireType::Bit32(v) => write!(f, "Bit32{:?}", v),
        }
    }
}
