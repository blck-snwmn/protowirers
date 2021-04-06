// #[derive(Proto)]
struct Sample {
    // #[proto_def(field_num=1, p_type=int32)]
    s: u32,
    // #[proto_def(field_num=2, p_type=int32)]
    x: i64,
}
use anyhow::Result;
use protowirers::{decode, parser};
use std::io::Cursor;
impl Sample {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut c = Cursor::new(bytes);
        let result = decode::decode_wire_binary(&mut c)?;

        let mut s: u32 = 0;
        let mut x: i64 = 0;
        for sw in result {
            match (sw.field_number(), sw.wire_type()) {
                (1, decode::WireType::Varint(v)) => {
                    s = parser::parse_u32(*v)?;
                }
                (2, decode::WireType::Varint(v)) => {
                    x = parser::parse_i64(*v)?;
                }
                _ => (),
            }
        }
        Ok(Self { s, x })
    }
}
