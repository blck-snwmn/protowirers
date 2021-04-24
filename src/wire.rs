use crate::zigzag::ZigZag;
use anyhow::Result;
use std::fmt::Display;

pub trait Proto {
    fn parse(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    fn bytes(&self) -> Result<Vec<u8>>;
}

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
        Self::new(field_number, WireType::Varint(WireTypeVarint::new(data)))
    }
    pub fn from_i32(field_number: FieldNumber, data: i32) -> Self {
        Self::from_in(field_number, data)
    }
    pub fn from_i64(field_number: FieldNumber, data: i64) -> Self {
        Self::from_in(field_number, data)
    }
    pub fn from_u32(field_number: FieldNumber, data: u32) -> Self {
        Self::new(
            field_number,
            WireType::Varint(WireTypeVarint::new(data as u128)),
        )
    }
    pub fn from_u64(field_number: FieldNumber, data: u64) -> Self {
        Self::new(
            field_number,
            WireType::Varint(WireTypeVarint::new(data as u128)),
        )
    }
    pub fn from_string(field_number: FieldNumber, data: String) -> Self {
        let data = Vec::from(data);
        Self::new(field_number, WireType::LengthDelimited(data))
    }
    pub fn from_vec(field_number: FieldNumber, data: String) -> Self {
        let data = Vec::from(data);
        Self::new(field_number, WireType::LengthDelimited(data))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WireType {
    Varint(WireTypeVarint),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireTypeVarint {
    pub value: u128,
}

impl WireTypeVarint {
    pub fn new(v: u128) -> Self {
        WireTypeVarint { value: v }
    }
}

impl Display for WireTypeVarint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Varint{}", self.value)
    }
}
pub enum TypeBit64 {
    Fixed64(u64), // 要確認
    Sfixed64(i64),
    Double(f64),
}
pub enum TypeLengthDelimited {
    WireString(String),
    Bytes,
    EmbeddedMessages(Box<dyn Proto>),
    PackedRepeatedFields,
}
pub enum TypeBit32 {
    Fixed32(u32),
    Sfixed32(i32),
    Float(f32),
}
