use std::{
    io::{Cursor, Read},
    ops::BitXorAssign,
    u128,
};

fn read_variants(mut data: Cursor<&[u8]>) -> Result<u128, String> {
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

fn read_tag(mut data: Cursor<&[u8]>) -> Result<WireTag, String> {
    let mut buf = [0; 1];
    let result = data.read_exact(&mut buf);
    if let Err(_) = result {
        return Err("unexpected format. end come after MSB is 0".to_string());
    }
    let n = buf[0] as u128;
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
            let c = Cursor::new(bytes);
            let x = super::read_variants(c);
            assert_eq!(x, Ok(1));
        }
        {
            let bytes: &[u8] = &[0b10101100, 0b00000010];
            let c = Cursor::new(bytes);
            let x = super::read_variants(c);
            assert_eq!(x, Ok(300));
        }
    }
    #[test]
    fn test_read_tag() {
        {
            let bytes: &[u8] = &[0b00001000];
            let c = Cursor::new(bytes);
            let got = read_tag(c).unwrap();

            let expected = WireTag {
                field_number: 1,
                wire_type: WireType::Varint,
            };
            assert_eq!(got, expected);
        }
        {
            let bytes: &[u8] = &[0b00011010];
            let c = Cursor::new(bytes);
            let got = read_tag(c).unwrap();

            let expected = WireTag {
                field_number: 3,
                wire_type: WireType::LengthDelimited,
            };
            assert_eq!(got, expected);
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
