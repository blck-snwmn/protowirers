use anyhow::Result;
use std::convert::TryFrom;
use std::io::{Cursor, Write};

use crate::wire::WireStruct;

// encode_variants decode varint format
fn encode_variants<T: std::io::Write>(data: &mut T, input: u128) -> Result<()> {
    let mut buf: Vec<u8> = Vec::with_capacity(calc_capacity(input));
    let mut input = input;
    loop {
        if input == 0 {
            break;
        }
        // 下位7bitずつ読みすすめる
        let mut x: u8 = TryFrom::try_from(input & 0b01111111)?;
        input >>= 7;
        if input != 0 {
            // MSB を１にする. その他は据え置き
            x |= 0b10000000;
        }
        buf.push(x);
    }
    data.write_all(buf.as_slice())?;
    Ok(())
}

// calc_capacity return capacity of buffer
fn calc_capacity(input: u128) -> usize {
    let mut input = input;
    let mut capacity = 0;
    while input != 0 {
        input >>= 7;
        capacity += 1;
    }
    capacity
}

pub(crate) fn encode_repeat<T: std::io::Write>(data: &mut T, input: Vec<u128>) -> Result<()> {
    for i in input {
        encode_variants(data, i)?;
    }
    Ok(())
}

fn encode_length_delimited(data: &mut Cursor<Vec<u8>>, input: Vec<u8>) -> Result<()> {
    encode_variants(data, input.len() as u128)?;
    data.write_all(input.as_slice())?;
    Ok(())
}

pub fn encode<const BYTE_SIZE: usize>(
    data: &mut Cursor<Vec<u8>>,
    input: [u8; BYTE_SIZE],
) -> Result<()> {
    data.write_all(&input)?;
    Ok(())
}

// encode_tag decode wire's tag
fn encode_tag(data: &mut Cursor<Vec<u8>>, field_number: u128, field_type: u128) -> Result<()> {
    let input = (field_number << 3) + field_type;
    encode_variants(data, input)?;
    Ok(())
}

// TODO フィールドの値がないときにもtagを書き込んでそうなので、直す
fn encode_struct(data: &mut Cursor<Vec<u8>>, input: WireStruct) -> Result<()> {
    encode_tag(data, input.field_number(), input.wire_type().type_number())?;
    match input.wire_type() {
        crate::wire::WireData::Varint(v) => {
            encode_variants(data, v.value)?;
        }
        crate::wire::WireData::Bit64(b) => {
            encode(data, b.value)?;
        }
        crate::wire::WireData::LengthDelimited(l) => {
            encode_length_delimited(data, l.value)?;
        }
        crate::wire::WireData::Bit32(b) => {
            encode(data, b.value)?;
        }
    }
    Ok(())
}

// encode_wire_binary decode wire format. return Vec included red filed.
pub fn encode_wire_binary(data: &mut Cursor<Vec<u8>>, inputs: Vec<WireStruct>) -> Result<()> {
    for input in inputs {
        if input.is_empty() {
            continue;
        }
        encode_struct(data, input)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        WireData, WireDataBit32, WireDataBit64, WireDataLengthDelimited, WireDataVarint,
    };
    #[test]
    fn test_encode_variants() {
        {
            let mut c = Cursor::new(Vec::new());
            encode_variants(&mut c, 1).unwrap();
            assert_eq!(c.position(), 1);

            let x: Vec<u8> = c.into_inner();
            assert_eq!(x, vec![0b00000001]);
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_variants(&mut c, 300).unwrap();
            // assert_eq!(c.position(), 2);

            let x: Vec<u8> = c.into_inner();
            assert_eq!(x, vec![0b10101100, 0b00000010]);
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_variants(&mut c, 12323412).unwrap();
            // assert_eq!(c.position(), 2);

            let x: Vec<u8> = c.into_inner();
            assert_eq!(x, vec![0b11010100, 0b10010100, 0b11110000, 0b00000101]);
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_variants(&mut c, 0).unwrap();
            // assert_eq!(c.position(), 2);

            let x: Vec<u8> = c.into_inner();
            assert_eq!(x, vec![]);
        }
        {
            {
                let x: i64 = -6423;
                let mut c = Cursor::new(Vec::new());
                encode_variants(&mut c, x as u128).unwrap();

                let x: Vec<u8> = c.into_inner();
                println!("{:?}", x);
            }
            {
                let x: i32 = -6423;
                let mut c = Cursor::new(Vec::new());
                encode_variants(&mut c, x as u64 as u128).unwrap();

                let x: Vec<u8> = c.into_inner();
                println!("{:?}", x);
            }
        }
    }

    #[test]
    fn test_encode_repeat() {
        {
            let mut v = Vec::new();
            encode_repeat(&mut v, vec![]).unwrap();
            assert_eq!(v, vec![]);
        }
        {
            let mut v = Vec::new();
            encode_repeat(&mut v, vec![1]).unwrap();
            assert_eq!(v, vec![0b00000001]);
        }
        {
            let mut v = Vec::new();
            encode_repeat(&mut v, vec![1, 300]).unwrap();
            assert_eq!(v, vec![0b00000001, 0b10101100, 0b00000010]);
        }
        {
            let mut v = Vec::new();
            encode_repeat(&mut v, vec![1, 300, 12323412]).unwrap();
            assert_eq!(
                v,
                vec![
                    0b00000001, 0b10101100, 0b00000010, 0b11010100, 0b10010100, 0b11110000,
                    0b00000101
                ]
            );
        }
    }

    #[test]
    fn test_encode_length_delimited() {
        {
            let mut c = Cursor::new(Vec::new());
            assert_eq!(c.position(), 0);
            encode_length_delimited(&mut c, vec![0b01111000, 0b01111000]).unwrap();
            assert_eq!(c.position(), 3);
            assert_eq!(c.into_inner(), vec![0b00000010, 0b01111000, 0b01111000]);
        }
    }

    #[test]
    fn test_encode() {
        {
            let mut c = Cursor::new(Vec::new());
            assert_eq!(c.position(), 0);
            encode(&mut c, [0, 0, 0, 64]).unwrap();
            assert_eq!(c.position(), 4);
            assert_eq!(
                c.into_inner(),
                vec![0b00000000, 0b00000000, 0b00000000, 0b01000000]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            assert_eq!(c.position(), 0);
            encode(&mut c, [0, 0, 0, 0, 0, 0, 240, 63]).unwrap();
            assert_eq!(c.position(), 8);
            assert_eq!(
                c.into_inner(),
                vec![
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ]
            );
        }
    }

    #[test]
    fn test_encode_tag() {
        {
            let mut c = Cursor::new(Vec::new());
            encode_tag(&mut c, 1, 0).unwrap();
            assert_eq!(c.position(), 1);
            assert_eq!(c.into_inner(), vec![0b00001000]);
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_tag(&mut c, 3, 2).unwrap();
            assert_eq!(c.position(), 1);
            assert_eq!(c.into_inner(), vec![0b00011010]);
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_tag(&mut c, 1000, 0).unwrap();
            assert_eq!(c.position(), 2);
            assert_eq!(c.into_inner(), vec![0b11000000, 0b0111110]);
        }
    }

    #[test]
    fn test_encode_struct() {
        {
            let mut c = Cursor::new(Vec::new());
            let ws = WireStruct::new(
                8,
                WireData::Bit32(WireDataBit32::new([
                    0b00000000, 0b00000000, 0b00000000, 0b01000000,
                ])),
            );
            encode_struct(&mut c, ws).unwrap();
            assert_eq!(c.position(), 5);
            assert_eq!(
                c.into_inner(),
                vec![0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let ws = WireStruct::new(
                1,
                WireData::Bit64(WireDataBit64::new([
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ])),
            );
            encode_struct(&mut c, ws).unwrap();
            assert_eq!(c.position(), 9);
            assert_eq!(
                c.into_inner(),
                vec![
                    0b00001001, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b00000000, 0b11110000, 0b00111111,
                ]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let ws = WireStruct::new(
                4,
                WireData::LengthDelimited(WireDataLengthDelimited::new(vec![
                    0b01111000, 0b11100011, 0b10000001, 0b10000010, 0b01111000, 0b11100011,
                    0b10000001, 0b10000010, 0b01111000, 0b11100011, 0b10000001, 0b10000010,
                ])),
            );
            encode_struct(&mut c, ws).unwrap();
            assert_eq!(c.position(), 14);
            assert_eq!(
                c.into_inner(),
                vec![
                    0b00100010, 0b00001100, 0b01111000, 0b11100011, 0b10000001, 0b10000010,
                    0b01111000, 0b11100011, 0b10000001, 0b10000010, 0b01111000, 0b11100011,
                    0b10000001, 0b10000010,
                ]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let ws = WireStruct::new(1000, WireData::Varint(WireDataVarint::new(10467)));
            encode_struct(&mut c, ws).unwrap();
            assert_eq!(c.position(), 4);
            assert_eq!(
                c.into_inner(),
                vec![0b11000000, 0b00111110, 0b11100011, 0b01010001]
            );
        }
    }

    #[test]
    fn test_encode_wire_binary() {
        {
            let mut c = Cursor::new(Vec::new());
            let ws = WireStruct::new(1000, WireData::Varint(WireDataVarint::new(10467)));
            encode_wire_binary(&mut c, vec![ws]).unwrap();
            assert_eq!(c.position(), 4);
            assert_eq!(
                c.into_inner(),
                vec![0b11000000, 0b00111110, 0b11100011, 0b01010001]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let wss = vec![WireStruct::new(
                8,
                WireData::Bit32(WireDataBit32::new([
                    0b00000000, 0b00000000, 0b00000000, 0b01000000,
                ])),
            )];
            encode_wire_binary(&mut c, wss).unwrap();
            assert_eq!(c.position(), 5);
            assert_eq!(
                c.into_inner(),
                vec![0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let wss = vec![
                WireStruct::new(
                    8,
                    WireData::Bit32(WireDataBit32::new([
                        0b00000000, 0b00000000, 0b00000000, 0b01000000,
                    ])),
                ),
                WireStruct::new(
                    1,
                    WireData::Bit64(WireDataBit64::new([
                        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                        0b11110000, 0b00111111,
                    ])),
                ),
            ];
            encode_wire_binary(&mut c, wss).unwrap();
            assert_eq!(c.position(), 14);
            assert_eq!(
                c.into_inner(),
                vec![
                    0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000, 0b00001001,
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ]
            );
        }
        {
            let mut c = Cursor::new(Vec::new());
            let wss = vec![
                WireStruct::new(
                    8,
                    WireData::Bit32(WireDataBit32::new([
                        0b00000000, 0b00000000, 0b00000000, 0b01000000,
                    ])),
                ),
                WireStruct::new(
                    1,
                    WireData::Bit64(WireDataBit64::new([
                        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                        0b11110000, 0b00111111,
                    ])),
                ),
            ];
            encode_wire_binary(&mut c, wss).unwrap();
            assert_eq!(c.position(), 14);
            assert_eq!(
                c.into_inner(),
                vec![
                    0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000, 0b00001001,
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ]
            );
        }
    }
    #[test]
    fn test_calc_capacity() {
        {
            let input = 0b1;
            let cap = calc_capacity(input);
            assert_eq!(cap, 1);
        }
        {
            let input = 0b1111111;
            let cap = calc_capacity(input);
            assert_eq!(cap, 1);
        }
        {
            let input = 0b10000000;
            let cap = calc_capacity(input);
            assert_eq!(cap, 2);
        }
    }

    #[test]
    fn bit_lenght() {
        {
            let input: u64 = 0b11110000_00111111_u64;
            dbg!(input);
            let zeros = input.leading_zeros();
            assert_eq!(zeros, 48);
            let l = u64::BITS - zeros;
            assert_eq!(l, 16);
        }
    }

    // #[test]
    // fn check() {
    //     {
    //         let bytes: Vec<u8> = Vec::new();
    //         let mut c = Cursor::new(bytes);

    //         println!("{:?}", c.write_all(&[1, 2, 3]));
    //         println!("{:?}", c.write_all(&[4, 5, 6]));
    //         println!("{:?}", c.write_all(&[7, 8, 9]));
    //         println!("{:?}", c.bytes());
    //     }
    //     {
    //         let mut c = Cursor::new(Vec::new());
    //         encode_variants(&mut c, 12323412).unwrap();

    //         let x: Vec<u8> = c.into_inner();
    //         println!("{:?}", x)
    //     }
    //     {
    //         let p = |x: u128| println!("{:#030b}", x);
    //         println!("--------------");
    //         let mut x = 12323412;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //     }
    //     {
    //         let p = |x: u128| println!("{:#018b}", x);
    //         println!("--------------");
    //         let mut x = 65535;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //         x = x >> 7;
    //         p(x);
    //     }
    // }
}
