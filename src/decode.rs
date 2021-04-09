use std::{
    fmt::Display,
    io::{Cursor, Read, Seek, SeekFrom},
    u128,
};

use anyhow::Result;
use thiserror::Error;
#[derive(Error, Debug)]
enum DecodeError {
    #[error("unexpected format. end come after MSB is 0")]
    UnexpectFormatError,
    #[error("unexpected red size. got={0}, want={1}")]
    UnexpectRepeatSizeError(u128, u128),
    #[error("no expected type value. got={0}")]
    UnexpectedWireTypeValueError(u128),
}

// decode_variants decode base variants
fn decode_variants(data: &mut Cursor<&[u8]>) -> Result<u128> {
    // iterate take_util とかでもできるよ
    let mut sum = 0;
    let mut loop_count = 0;
    loop {
        let mut buf = [0; 1];
        let result = data.read_exact(&mut buf);
        if let Err(_) = result {
            return Err(DecodeError::UnexpectFormatError)?;
        }
        // MSB は後続のバイトが続くかどうかの判定に使われる
        // 1 の場合、後続が続く
        let top = buf[0] & 0b10000000;
        let buf: u128 = (buf[0] & 0b01111111) as u128;
        // little endian
        let buf = buf << (7 * loop_count);
        sum = buf + sum;
        loop_count = loop_count + 1;
        if top != 0b10000000 {
            return Ok(sum);
        }
    }
}

// decode_length_delimited decode variable length byte.
// length to decode is first variants
// this function used by `string`, `embedded messages`
fn decode_length_delimited(data: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let length = decode_variants(data)? as usize;
    let mut buf = vec![0; length];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

fn decode_nbit<const SIZE: usize>(data: &mut Cursor<&[u8]>) -> Result<[u8; SIZE]> {
    let mut buf = [0; SIZE];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

fn decode_32bit(data: &mut Cursor<&[u8]>) -> Result<[u8; 4]> {
    decode_nbit(data)
}
fn decode_64bit(data: &mut Cursor<&[u8]>) -> Result<[u8; 8]> {
    decode_nbit(data)
}

// decode_repeat decode repeated elements
fn decode_repeat(data: &mut Cursor<&[u8]>) -> Result<Vec<u128>> {
    let payload_size = decode_variants(data)?;
    let start = data.position();

    let mut v = Vec::new();
    loop {
        let now_position = data.position();
        let red_size = (now_position - start) as u128;
        if payload_size == red_size {
            return Ok(v);
        }
        if payload_size < red_size {
            return Err(DecodeError::UnexpectRepeatSizeError(payload_size, red_size))?;
        }
        let value = decode_variants(data)?;
        v.push(value);
    }
}

// decode_tag decode wire's tag
fn decode_tag(data: &mut Cursor<&[u8]>) -> Result<(u128, u128)> {
    let n = decode_variants(data)?;
    let wt = n & 7;
    let field_number = n >> 3;
    Ok((field_number, wt))
}

fn decode_struct(data: &mut Cursor<&[u8]>) -> Result<WireStruct> {
    let (field_num, wire_type) = decode_tag(data)?;
    let wt = match wire_type {
        0 => Ok(WireType::Varint(decode_variants(data)?)),
        1 => Ok(WireType::Bit64(decode_64bit(data)?)),
        2 => Ok(WireType::LengthDelimited(decode_length_delimited(data)?)),
        // 3=>WireType::StartGroup,
        // 4=>WireType::EndGroup,
        5 => Ok(WireType::Bit32(decode_32bit(data)?)),
        _ => Err(DecodeError::UnexpectedWireTypeValueError(wire_type)),
    }?;
    Ok(WireStruct {
        field_number: field_num,
        wire_type: wt,
    })
}

// decode_wire_binary decode wire format. return Vec included red filed.
pub fn decode_wire_binary(data: &mut Cursor<&[u8]>) -> Result<Vec<WireStruct>> {
    let mut v = Vec::new();
    // ここは、`data.get_ref().len();` でもよい。
    let end = data.seek(SeekFrom::End(0))?;
    data.seek(SeekFrom::Start(0))?;

    while end > data.position() {
        v.push(decode_struct(data)?);
    }
    Ok(v)
}

// TODO 別ファイルへ移動
type FieldNumber = u128;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_decode_variants() {
        {
            let bytes: &[u8] = &[0b00000001];
            let mut c = Cursor::new(bytes);
            assert_eq!(c.position(), 0);
            let x = super::decode_variants(&mut c).unwrap();
            assert_eq!(x, 1);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b10101100, 0b00000010];
            let mut c = Cursor::new(bytes);
            assert_eq!(c.position(), 0);
            let x = super::decode_variants(&mut c).unwrap();
            assert_eq!(x, 300);
            assert_eq!(c.position(), 2);
        }
    }
    #[test]
    fn test_decode_tag() {
        {
            let bytes: &[u8] = &[0b00001000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_tag(&mut c).unwrap();

            let expected = (1, 0);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b00011010];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = decode_tag(&mut c).unwrap();

            let expected = (3, 2);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b11000000, 0b0111110];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = decode_tag(&mut c).unwrap();

            let expected = (1000, 0);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 2);
        }
    }

    #[test]
    fn test_decode_length_delimited() {
        {
            let bytes: &[u8] = &[0b00000010, 0b01111000, 0b01111000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_length_delimited(&mut c).unwrap();

            let expected = vec![0b01111000, 0b01111000];
            assert_eq!(got, expected);
            assert_eq!(c.position(), 3);
        }
    }

    #[test]
    fn test_decode_repeat() {
        {
            let bytes: &[u8] = &[
                0b00000110, 0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
            ];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_repeat(&mut c).unwrap();

            assert_eq!(got, vec![1, 2, 1000, 4, 5]);
            assert_eq!(c.position(), 7);
        }
    }

    #[test]
    fn test_decode_32bit() {
        let bytes: &[u8] = &[0b00000000, 0b00000000, 0b00000000, 0b01000000];
        let mut c = Cursor::new(bytes);

        assert_eq!(c.position(), 0);

        let got = decode_32bit(&mut c).unwrap();

        assert_eq!(got, [0, 0, 0, 64]);
        assert_eq!(c.position(), 4);
    }
    #[test]
    fn test_decode_64bit() {
        let bytes: &[u8] = &[
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110000,
            0b00111111,
        ];
        let mut c = Cursor::new(bytes);

        assert_eq!(c.position(), 0);

        let got = decode_64bit(&mut c).unwrap();

        assert_eq!(got, [0, 0, 0, 0, 0, 0, 240, 63]);
        assert_eq!(c.position(), 8);
    }

    #[test]
    fn test_decode_struct() {
        {
            let bytes: &[u8] = &[0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_struct(&mut c).unwrap();

            let expected = WireStruct {
                field_number: 8,
                wire_type: WireType::Bit32([0b00000000, 0b00000000, 0b00000000, 0b01000000]),
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 5);
        }
        {
            let bytes: &[u8] = &[
                0b00001001, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                0b11110000, 0b00111111,
            ];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_struct(&mut c).unwrap();

            let expected = WireStruct {
                field_number: 1,
                wire_type: WireType::Bit64([
                    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                    0b11110000, 0b00111111,
                ]),
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 9);
        }
        {
            let bytes: &[u8] = &[
                0b00100010, 0b00001100, 0b01111000, 0b11100011, 0b10000001, 0b10000010, 0b01111000,
                0b11100011, 0b10000001, 0b10000010, 0b01111000, 0b11100011, 0b10000001, 0b10000010,
            ];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_struct(&mut c).unwrap();

            let expected = WireStruct {
                field_number: 4,
                wire_type: WireType::LengthDelimited(vec![
                    0b01111000, 0b11100011, 0b10000001, 0b10000010, 0b01111000, 0b11100011,
                    0b10000001, 0b10000010, 0b01111000, 0b11100011, 0b10000001, 0b10000010,
                ]),
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 14);
        }
        {
            let bytes: &[u8] = &[0b11000000, 0b00111110, 0b11100011, 0b01010001];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_struct(&mut c).unwrap();

            let expected = WireStruct {
                field_number: 1000,
                wire_type: WireType::Varint(10467),
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 4);
        }
    }

    #[test]
    fn test_decode_wire_binary() {
        {
            let bytes: &[u8] = &[0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_wire_binary(&mut c).unwrap();

            let expected = vec![WireStruct {
                field_number: 8,
                wire_type: WireType::Bit32([0b00000000, 0b00000000, 0b00000000, 0b01000000]),
            }];
            assert_eq!(got, expected);
            assert_eq!(c.position(), 5);
        }
        {
            let bytes: &[u8] = &[
                0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000, 0b00001001, 0b00000000,
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110000, 0b00111111,
            ];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = decode_wire_binary(&mut c).unwrap();

            let expected = vec![
                WireStruct {
                    field_number: 8,
                    wire_type: WireType::Bit32([0b00000000, 0b00000000, 0b00000000, 0b01000000]),
                },
                WireStruct {
                    field_number: 1,
                    wire_type: WireType::Bit64([
                        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                        0b11110000, 0b00111111,
                    ]),
                },
            ];
            assert_eq!(got, expected);
            assert_eq!(c.position(), 14);
        }
        {
            let bytes: &[u8] = &[
                0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000, 0b00001001, 0b00000000,
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110000,
            ];
            let mut c = Cursor::new(bytes);
            assert!(decode_wire_binary(&mut c).is_err());
        }
    }

    #[test]
    fn check() {
        {
            let mut sum = 0;
            sum = 2 + sum;
            println!("sum={}", sum);
            sum = 0b00101100 + (sum << 7);
            println!("sum={}", sum);
        }
        {
            println!("{:08b}", 0b10000001 & 7);
            println!("{:08b}", 0b10000101 & 7);
        }
        {
            let encode = |n: i32| {
                println!(
                    "{:32}:{:032b}, {:032b}, {:032b}={}",
                    n,
                    n,
                    n << 1,
                    n >> 31,
                    (n << 1) ^ (n >> 31)
                )
            };
            let n = 1;
            encode(n);
            let n = -1;
            encode(n);
            let n = -2;
            encode(n);
            let n = 2;
            encode(n);
            let n = 3;
            encode(n);
            let n = -3;
            encode(n);
        }
        {
            let decode = |n: i32| {
                println!(
                    "{:32}:{:032b}, {:032b}, {:032b}={}",
                    n,
                    n,
                    n >> 1,
                    -(n & 1),
                    (n >> 1) ^ -(n & 1)
                )
            };
            let n = 1;
            decode(n);
            let n = 2;
            decode(n);
            let n = 3;
            decode(n);
            let n = 4;
            decode(n);
            let n = 5;
            decode(n);
            let n = 6;
            decode(n);
        }
        {
            let bytes: &[u8] = &[0b11000000, 0b00111110, 0b11100011, 0b01010001];
            let mut c = Cursor::new(bytes);

            let mut buf = [0; 6];

            println!("{:?}", c.get_ref().len());
            println!("empty={}", buf.is_empty());
            println!("start={}", c.position());
            let r = c.read_exact(&mut buf);
            println!("result={:?}", r);
            println!("1st end={}", c.position());
            let r = c.read_exact(&mut buf);
            println!("result={:?}", r);
            println!("end={}", c.position());
            println!("write={:?}", buf);
            println!("empty={}", buf.is_empty());
            println!("{:?}", c.seek(SeekFrom::End(0)));
        }
    }
}
