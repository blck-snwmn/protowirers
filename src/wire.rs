use std::fmt::Display;

// TODO 別ファイルへ移動
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
