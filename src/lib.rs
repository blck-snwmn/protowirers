use std::io::{Cursor, Read};

fn read_variants(mut data: Cursor<&[u8]>) -> Result<u128, &str> {
    // iterate take_util とかでもできるよ
    let mut sum = 0;
    let mut loop_count = 0;
    loop {
        let mut buf = [0; 1];
        let result = data.read_exact(&mut buf);
        if let Err(_) = result {
            return Err("unexpected format. end come after MSB is 0");
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

#[cfg(test)]
mod tests {
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
    fn check() {
        let mut sum = 0;
        sum = 2 + sum;
        println!("sum={}", sum);
        sum = 0b00101100 + (sum << 7);
        println!("sum={}", sum);
    }
}
