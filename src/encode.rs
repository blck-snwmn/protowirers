use anyhow::Result;
use std::convert::TryFrom;
use std::{
    io::{Cursor, Write},
    u128,
};

use crate::{decode::WireStruct, zigzag::encode64};

fn encode_variants(data: &mut Cursor<Vec<u8>>, input: u128) -> Result<()> {
    // 事前にbufferを確保すること
    let mut buf: Vec<u8> = Vec::new();
    let mut input = input;
    loop {
        if input == 0 {
            break;
        }
        // 下位7bitずつ読みすすめる
        let mut x: u8 = TryFrom::try_from(input & 0b01111111)?;
        input = input >> 7;
        if input != 0 {
            // MSB を１にする. その他は据え置き
            x = x | 0b10000000;
        }
        buf.push(x);
    }
    data.write_all(buf.as_slice())?;
    Ok(())
}

fn encode_length_delimited(data: &mut Cursor<Vec<u8>>, input: Vec<u8>) -> Result<()> {
    encode_variants(data, input.len() as u128)?;
    data.write_all(input.as_slice())?;
    Ok(())
}

fn encode_32bit(data: &mut Cursor<Vec<u8>>, input: [u8; 4]) -> Result<()> {
    data.write_all(&input)?;
    Ok(())
}
fn encode_64bit(data: &mut Cursor<Vec<u8>>, input: [u8; 8]) -> Result<()> {
    data.write_all(&input)?;
    Ok(())
}

// encode_tag decode wire's tag
fn encode_tag(data: &mut Cursor<Vec<u8>>, field_number: u128, field_type: u128) -> Result<()> {
    let input = (field_number << 3) + field_type;
    encode_variants(data, input)?;
    Ok(())
}

fn encode_struct(data: &mut Cursor<Vec<u8>>, input: WireStruct) -> Result<()> {
    encode_tag(data, input.field_number(), input.wire_type().type_number())?;
    match input.wire_type() {
        crate::decode::WireType::Varint(v) => {
            encode_variants(data, v)?;
        }
        crate::decode::WireType::Bit64(b) => {
            encode_64bit(data, b)?;
        }
        crate::decode::WireType::LengthDelimited(l) => {
            encode_length_delimited(data, l)?;
        }
        crate::decode::WireType::Bit32(b) => {
            encode_32bit(data, b)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::*;
    use std::io::Read;
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
    fn test_encode_32bit() {
        {
            let mut c = Cursor::new(Vec::new());
            assert_eq!(c.position(), 0);
            encode_32bit(&mut c, [0, 0, 0, 64]).unwrap();
            assert_eq!(c.position(), 4);
            assert_eq!(
                c.into_inner(),
                vec![0b00000000, 0b00000000, 0b00000000, 0b01000000]
            );
        }
    }

    #[test]
    fn test_encode_64bit() {
        {
            let mut c = Cursor::new(Vec::new());
            assert_eq!(c.position(), 0);
            encode_64bit(&mut c, [0, 0, 0, 0, 0, 0, 240, 63]).unwrap();
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
                WireType::Bit32([0b00000000, 0b00000000, 0b00000000, 0b01000000]),
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
                WireType::Bit64([
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ]),
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
                WireType::LengthDelimited(vec![
                    0b01111000, 0b11100011, 0b10000001, 0b10000010, 0b01111000, 0b11100011,
                    0b10000001, 0b10000010, 0b01111000, 0b11100011, 0b10000001, 0b10000010,
                ]),
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
            let ws = WireStruct::new(1000, WireType::Varint(10467));
            encode_struct(&mut c, ws).unwrap();
            assert_eq!(c.position(), 4);
            assert_eq!(
                c.into_inner(),
                vec![0b11000000, 0b00111110, 0b11100011, 0b01010001]
            );
        }
    }

    #[test]
    fn check() {
        {
            let bytes: Vec<u8> = Vec::new();
            let mut c = Cursor::new(bytes);

            println!("{:?}", c.write_all(&[1, 2, 3]));
            println!("{:?}", c.write_all(&[4, 5, 6]));
            println!("{:?}", c.write_all(&[7, 8, 9]));
            println!("{:?}", c.bytes());
        }
        {
            let mut c = Cursor::new(Vec::new());
            encode_variants(&mut c, 12323412).unwrap();

            let x: Vec<u8> = c.into_inner();
            println!("{:?}", x)
        }
        {
            let p = |x: u128| println!("{:#030b}", x);
            println!("--------------");
            let mut x = 12323412;
            p(x);
            x = x >> 7;
            p(x);
            x = x >> 7;
            p(x);
            x = x >> 7;
            p(x);
        }
        {
            let p = |x: u128| println!("{:#018b}", x);
            println!("--------------");
            let mut x = 65535;
            p(x);
            x = x >> 7;
            p(x);
            x = x >> 7;
            p(x);
            x = x >> 7;
            p(x);
        }
    }
}
