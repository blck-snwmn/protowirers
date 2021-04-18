use crate::{
    decode::{WireStruct, WireType},
    zigzag::{self, ZigZag},
};

pub fn to_wire_struct<T: Into<u128>>(field_num: u128, data: T) -> WireStruct {
    let data: u128 = data.into();
    let t = WireType::Varint(data);
    WireStruct::new(field_num, t)
}

pub fn to_wire_struct_from_signed<T, U>(field_num: u128, data: T) -> WireStruct
where
    T: ZigZag<Output = U>,
    U: Into<u128>,
{
    let data = zigzag::encode(data);
    to_wire_struct(field_num, data)
}
