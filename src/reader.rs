use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    u128,
};

// read_variants read base variants
fn read_variants(data: &mut Cursor<&[u8]>) -> Result<u128, String> {
    // iterate take_util とかでもできるよ
    let mut sum = 0;
    let mut loop_count = 0;
    loop {
        let mut buf = [0; 1];
        let result = data.read_exact(&mut buf);
        if let Err(_) = result {
            return Err("unexpected format. end come after MSB is 0".to_string());
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

// read_length_delimited read variable length byte.
// length to read is first variants
// this function used by `string`, `embedded messages`
fn read_length_delimited(data: &mut Cursor<&[u8]>) -> Result<Vec<u8>, String> {
    let length = read_variants(data)? as usize;
    let mut buf = vec![0; length];
    data.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn read_32bit(data: &mut Cursor<&[u8]>) -> Result<[u8; 4], String> {
    let mut buf = [0; 4];
    data.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}
fn read_64bit(data: &mut Cursor<&[u8]>) -> Result<[u8; 8], String> {
    let mut buf = [0; 8];
    data.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

// read_repeat read repeated elements
fn read_repeat(data: &mut Cursor<&[u8]>) -> Result<Vec<u128>, String> {
    let payload_size = read_variants(data)?;
    let start = data.position();

    let mut v = Vec::new();
    loop {
        let now_position = data.position();
        let red_size = (now_position - start) as u128;
        if payload_size == red_size {
            return Ok(v);
        }
        if payload_size < red_size {
            return Err("unexpected red size".to_string());
        }
        let value = read_variants(data)?;
        v.push(value);
    }
}

// read_tag read wire's tag
fn read_tag(data: &mut Cursor<&[u8]>) -> Result<(u128, u128), String> {
    let n = read_variants(data)?;
    let wt = n & 7;
    let field_number = n >> 3;
    Ok((field_number, wt))
}

fn read_struct(data: &mut Cursor<&[u8]>) -> Result<WireStruct, String> {
    let (field_num, wire_type) = read_tag(data)?;
    let wt = match wire_type {
        0 => Ok(WireType::Varint(read_variants(data)?)),
        1 => Ok(WireType::Bit64(read_64bit(data)?)),
        2 => Ok(WireType::LengthDelimited(read_length_delimited(data)?)),
        // 3=>WireType::StartGroup,
        // 4=>WireType::EndGroup,
        5 => Ok(WireType::Bit32(read_32bit(data)?)),
        _ => Err(format!("no expected type value. got={}", wire_type)),
    }?;
    Ok(WireStruct {
        field_number: field_num,
        wire_type: wt,
    })
}

// read_wire_binary read wire format. return Vec included red filed.
fn read_wire_binary(data: &mut Cursor<&[u8]>) -> Result<Vec<WireStruct>, String> {
    let mut v = Vec::new();
    // ここは、`data.get_ref().len();` でもよい。
    let end = data.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    data.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;

    while end > data.position() {
        v.push(read_struct(data)?);
    }
    Ok(v)
}

type FieldNumber = u128;

#[derive(Debug, PartialEq, Eq)]
struct WireStruct {
    field_number: FieldNumber,
    wire_type: WireType,
}

#[derive(Debug, PartialEq, Eq)]
enum WireType {
    Varint(u128),
    Bit64([u8; 8]),
    LengthDelimited(Vec<u8>),
    // StartGroup,
    // EndGroup,
    Bit32([u8; 4]),
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
    fn test_read_variants() {
        {
            let bytes: &[u8] = &[0b00000001];
            let mut c = Cursor::new(bytes);
            assert_eq!(c.position(), 0);
            let x = super::read_variants(&mut c);
            assert_eq!(x, Ok(1));
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b10101100, 0b00000010];
            let mut c = Cursor::new(bytes);
            assert_eq!(c.position(), 0);
            let x = super::read_variants(&mut c);
            assert_eq!(x, Ok(300));
            assert_eq!(c.position(), 2);
        }
    }
    #[test]
    fn test_read_tag() {
        {
            let bytes: &[u8] = &[0b00001000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_tag(&mut c).unwrap();

            let expected = (1, 0);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b00011010];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = read_tag(&mut c).unwrap();

            let expected = (3, 2);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b11000000, 0b0111110];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = read_tag(&mut c).unwrap();

            let expected = (1000, 0);
            assert_eq!(got, expected);
            assert_eq!(c.position(), 2);
        }
    }

    #[test]
    fn test_read_length_delimited() {
        {
            let bytes: &[u8] = &[0b00000010, 0b01111000, 0b01111000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_length_delimited(&mut c).unwrap();

            let expected = vec![0b01111000, 0b01111000];
            assert_eq!(got, expected);
            assert_eq!(c.position(), 3);
        }
    }

    #[test]
    fn test_read_repeat() {
        {
            let bytes: &[u8] = &[
                0b00000110, 0b00000001, 0b00000010, 0b11101000, 0b00000111, 0b00000100, 0b00000101,
            ];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_repeat(&mut c).unwrap();

            assert_eq!(got, vec![1, 2, 1000, 4, 5]);
            assert_eq!(c.position(), 7);
        }
    }

    #[test]
    fn test_read_32bit() {
        let bytes: &[u8] = &[0b00000000, 0b00000000, 0b00000000, 0b01000000];
        let mut c = Cursor::new(bytes);

        assert_eq!(c.position(), 0);

        let got = read_32bit(&mut c).unwrap();

        assert_eq!(got, [0, 0, 0, 64]);
        assert_eq!(c.position(), 4);
    }
    #[test]
    fn test_read_64bit() {
        let bytes: &[u8] = &[
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110000,
            0b00111111,
        ];
        let mut c = Cursor::new(bytes);

        assert_eq!(c.position(), 0);

        let got = read_64bit(&mut c).unwrap();

        assert_eq!(got, [0, 0, 0, 0, 0, 0, 240, 63]);
        assert_eq!(c.position(), 8);
    }

    #[test]
    fn test_read_struct() {
        {
            let bytes: &[u8] = &[0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_struct(&mut c).unwrap();

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

            let got = read_struct(&mut c).unwrap();

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

            let got = read_struct(&mut c).unwrap();

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

            let got = read_struct(&mut c).unwrap();

            let expected = WireStruct {
                field_number: 1000,
                wire_type: WireType::Varint(10467),
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 4);
        }
    }

    #[test]
    fn test_read_wire_binary() {
        {
            let bytes: &[u8] = &[0b01000101, 0b00000000, 0b00000000, 0b00000000, 0b01000000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_wire_binary(&mut c).unwrap();

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

            let got = read_wire_binary(&mut c).unwrap();

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
            assert!(read_wire_binary(&mut c).is_err());
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
