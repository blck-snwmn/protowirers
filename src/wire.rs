use crate::zigzag::ZigZag;
use crate::Result;
use std::fmt::Display;

pub trait Proto {
    fn parse(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
    fn bytes(&self) -> Result<Vec<u8>>;
}

// alias　ではなく、タプル構造体にしたほうがよさそう
pub type FieldNumber = u128;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireStruct {
    field_number: FieldNumber,
    wire_type: WireData,
}

impl WireStruct {
    pub fn field_number(&self) -> FieldNumber {
        self.field_number
    }
    // TODO ここは参照を返せば良さそう？clone実装してもよさそう
    pub fn wire_type(&self) -> WireData {
        self.wire_type.clone()
    }
    // is_empty は WireStruct の 値がゼロかどうか確認します。
    // ゼロの場合、encode時に書き出されません
    pub fn is_empty(&self) -> bool {
        match &self.wire_type {
            WireData::Varint(v) => v.value == 0,
            WireData::Bit64(b) => b.value.iter().all(|v| *v == 0),
            WireData::LengthDelimited(l) => l.value.is_empty(),
            WireData::Bit32(b) => b.value.iter().all(|v| *v == 0),
        }
    }
    pub const fn new(field_number: FieldNumber, wire_type: WireData) -> Self {
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
        Self::new(field_number, WireData::Varint(WireDataVarint::new(data)))
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
            WireData::Varint(WireDataVarint::new(data as u128)),
        )
    }
    pub fn from_u64(field_number: FieldNumber, data: u64) -> Self {
        Self::new(
            field_number,
            WireData::Varint(WireDataVarint::new(data as u128)),
        )
    }
    pub fn from_string(field_number: FieldNumber, data: String) -> Self {
        let data = Vec::from(data);
        Self::new(
            field_number,
            WireData::LengthDelimited(WireDataLengthDelimited::new(data)),
        )
    }
    pub fn from_vec(field_number: FieldNumber, data: String) -> Self {
        let data = Vec::from(data);
        Self::new(
            field_number,
            WireData::LengthDelimited(WireDataLengthDelimited::new(data)),
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WireData {
    Varint(WireDataVarint),
    Bit64(WireDataBit64),
    LengthDelimited(WireDataLengthDelimited),
    // StartGroup,
    // EndGroup,
    Bit32(WireDataBit32),
}

impl WireData {
    pub fn type_number(&self) -> u128 {
        match &self {
            WireData::Varint(_) => 0,
            WireData::Bit64(_) => 1,
            WireData::LengthDelimited(_) => 2,
            WireData::Bit32(_) => 5,
        }
    }
}

impl Display for WireData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            WireData::Varint(v) => write!(f, "Varint{}", v),
            WireData::Bit64(v) => write!(f, "Bit64{:?}", v),
            WireData::LengthDelimited(v) => write!(f, "LengthDelimited{:?}", v),
            WireData::Bit32(v) => write!(f, "Bit32{:?}", v),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireDataVarint {
    pub value: u128,
}

impl WireDataVarint {
    pub const fn new(v: u128) -> Self {
        // TODO 暫定でInt64をセット
        WireDataVarint { value: v }
    }
}

impl Display for WireDataVarint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Varint{{{}}}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireDataLengthDelimited {
    pub value: Vec<u8>,
}

impl WireDataLengthDelimited {
    pub fn new(v: Vec<u8>) -> Self {
        // TODO 暫定でWireStringをセット
        WireDataLengthDelimited { value: v }
    }
}
impl Display for WireDataLengthDelimited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LengthDelimited{{{:?}}}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireDataBit64 {
    pub value: [u8; 8],
}
impl WireDataBit64 {
    pub const fn new(v: [u8; 8]) -> Self {
        // TODO 暫定でWireStringをセット
        WireDataBit64 { value: v }
    }
}
impl Display for WireDataBit64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bit64{{{:?}}}", self.value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WireDataBit32 {
    pub value: [u8; 4],
}
impl WireDataBit32 {
    pub const fn new(v: [u8; 4]) -> Self {
        // TODO 暫定でWireStringをセット
        WireDataBit32 { value: v }
    }
}
impl Display for WireDataBit32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bit32{{{:?}}}", self.value)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeVairant {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeBit64 {
    Fixed64,
    Sfixed64,
    Double,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeLengthDelimited {
    WireString,
    Bytes,
    EmbeddedMessages,
    PackedRepeatedFields(AllowedPakcedType),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AllowedPakcedType {
    Variant(TypeVairant),
    Bit64(TypeBit64),
    Bit32(TypeBit32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeBit32 {
    Fixed32,
    Sfixed32,
    Float,
}
