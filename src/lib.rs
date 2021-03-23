use std::{
    io::{Cursor, Read},
    u128,
};

mod zigzag;

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

// read_variable_lenth read variable length byte.
// length to read is first variants
// this function used by `string`, `embedded messages`
fn read_variable_lenth(data: &mut Cursor<&[u8]>) -> Result<Vec<u8>, String> {
    let length = read_variants(data)? as usize;
    let mut buf = vec![0; length];
    data.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

// read_zigzag read zigzag encoding data
fn read_zigzag(data: &mut Cursor<&[u8]>) -> Result<i128, String> {
    let v = read_variants(data)?;
    Ok(zigzag::decode(v))
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
fn read_tag(data: &mut Cursor<&[u8]>) -> Result<WireTag, String> {
    let n = read_variants(data)?;
    let wtb = n & 7;
    let field_number = n >> 3;
    let wt = WireType::new(wtb);
    if wt.is_none() {
        return Err(format!("no supurted type. got={}", wtb));
    }
    let wt = wt.unwrap();
    Ok(WireTag {
        field_number: field_number,
        wire_type: wt,
    })
}
#[derive(Debug, PartialEq, Eq)]
struct WireStruct {
    tag: WireTag,
    value: u128,
}

type FieldNumber = u128;
#[derive(Debug, PartialEq, Eq)]
struct WireTag {
    field_number: FieldNumber,
    wire_type: WireType,
}
#[derive(Debug, PartialEq, Eq)]
enum WireType {
    Varint,
    Bit64,
    LengthDelimited,
    StartGroup,
    EndGroup,
    Bit32,
}

impl WireType {
    fn new(u: u128) -> Option<Self> {
        match u {
            0 => Some(WireType::Varint),
            1 => Some(WireType::Bit64),
            2 => Some(WireType::LengthDelimited),
            3 => Some(WireType::StartGroup),
            4 => Some(WireType::EndGroup),
            5 => Some(WireType::Bit32),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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

            let expected = WireTag {
                field_number: 1,
                wire_type: WireType::Varint,
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b00011010];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = read_tag(&mut c).unwrap();

            let expected = WireTag {
                field_number: 3,
                wire_type: WireType::LengthDelimited,
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 1);
        }
        {
            let bytes: &[u8] = &[0b11000000, 0b0111110];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);
            let got = read_tag(&mut c).unwrap();

            let expected = WireTag {
                field_number: 1000,
                wire_type: WireType::Varint,
            };
            assert_eq!(got, expected);
            assert_eq!(c.position(), 2);
        }
    }

    #[test]
    fn test_read_variable_lenth() {
        {
            let bytes: &[u8] = &[0b00000010, 0b01111000, 0b01111000];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_variable_lenth(&mut c).unwrap();

            let expected = vec![0b01111000, 0b01111000];
            assert_eq!(got, expected);
            assert_eq!(c.position(), 3);
        }
    }

    #[test]
    fn test_read_zigzag() {
        {
            let bytes: &[u8] = &[0b10011111, 0b10011100, 0b00000001];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_zigzag(&mut c).unwrap();

            assert_eq!(got, -10000);
            assert_eq!(c.position(), 3);
        }
        {
            let bytes: &[u8] = &[0b10100011, 0b00010011];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_zigzag(&mut c).unwrap();

            assert_eq!(got, -1234);
            assert_eq!(c.position(), 2);
        }
        {
            let bytes: &[u8] = &[0b11100100, 0b01010001];
            let mut c = Cursor::new(bytes);

            assert_eq!(c.position(), 0);

            let got = read_zigzag(&mut c).unwrap();

            assert_eq!(got, 5234);
            assert_eq!(c.position(), 2);
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
        {}
    }
}
