use std::{
    io::{Cursor, Read},
    ops::BitXorAssign,
    u128,
};

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

fn read_variable_lenth(data: &mut Cursor<&[u8]>) -> Result<Vec<u8>, String> {
    let length = read_variants(data)? as usize;
    let mut buf = vec![0; length];
    data.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

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
#[derive(Debug, PartialEq, Eq)]
struct WireTag {
    field_number: u128,
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
    }
}
