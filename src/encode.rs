use anyhow::Result;
use std::convert::TryFrom;
use std::{
    io::{Cursor, Write},
    u128,
};
fn encode_variants(data: &mut Cursor<Vec<u8>>, input: u128) -> Result<()> {
    // 事前にbufferを確保すること
    let mut buf: Vec<u8> = Vec::new();
    let mut input = input;
    loop {
        // 下位7bitずつ読みすすめる
        let mut x: u8 = TryFrom::try_from(input & 0b01111111)?;
        input = input >> 7;
        if input != 0 {
            // MSB を１にする. その他は据え置き
            x = x | 0b10000000;
        }
        buf.push(x);
        if input == 0 {
            break;
        }
    }
    data.write_all(buf.as_slice())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    }
}
