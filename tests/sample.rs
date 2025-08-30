// #[derive(Proto)]
struct Sample {
    // #[proto_def(field_num=1, p_type=int32)]
    s: u32,
    // #[proto_def(field_num=2, p_type=sint64)]
    x: i64,
}
use protowirers::Result;
use protowirers::parser::Parser;
use protowirers::wire::*;
use protowirers::{decode, encode};
use std::io::Cursor;
impl Sample {
    #[allow(dead_code)]
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut c = Cursor::new(bytes);
        let result = decode::decode_wire_binary(&mut c)?;

        let mut s: u32 = 0;
        let mut x: i64 = 0;

        // parser 内で Hashmap<u128, Vec<>> にし、関数呼び出しのみでセットする
        for sw in result {
            match (sw.field_number(), sw.wire_type()) {
                (1, WireData::Varint(v)) => {
                    s = v.parse(TypeVairant::Uint32)?;
                }
                (2, WireData::Varint(v)) => {
                    x = v.parse(TypeVairant::Uint64)?;
                }
                _ => (),
            }
        }
        Ok(Self { s, x })
    }

    #[allow(dead_code)]
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let inputs = vec![
            WireStruct::from_u32(1, self.s),
            WireStruct::from_i64(2, self.x),
        ];
        let mut c = Cursor::new(Vec::new());
        encode::encode_wire_binary(&mut c, inputs)?;
        Ok(c.into_inner())
    }
}
