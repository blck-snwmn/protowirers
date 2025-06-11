use criterion::{black_box, criterion_group, criterion_main, Criterion};
use protowirers::wire::*;
use std::io::Cursor;

fn create_simple_struct() -> WireStruct {
    WireStruct::new(1, WireData::Varint(WireDataVarint::new(150)))
}

fn create_complex_struct() -> Vec<WireStruct> {
    vec![
        WireStruct::new(1, WireData::Varint(WireDataVarint::new(123456789))),
        WireStruct::new(
            2,
            WireData::LengthDelimited(WireDataLengthDelimited::new(
                b"Hello, protowirers benchmark!".to_vec(),
            )),
        ),
        WireStruct::new(
            3,
            WireData::Bit64(WireDataBit64::new([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            ])),
        ),
        WireStruct::new(
            4,
            WireData::Bit32(WireDataBit32::new([0x01, 0x02, 0x03, 0x04])),
        ),
    ]
}

fn encode_single_varint(c: &mut Criterion) {
    c.bench_function("encode_single_varint", |b| {
        let ws = create_simple_struct();
        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(16));
            let _ =
                protowirers::encode::encode_wire_binary(&mut cursor, black_box(vec![ws.clone()]));
            cursor.into_inner()
        })
    });
}

fn encode_complex_message(c: &mut Criterion) {
    c.bench_function("encode_complex_message", |b| {
        let wss = create_complex_struct();
        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(128));
            let _ = protowirers::encode::encode_wire_binary(&mut cursor, black_box(wss.clone()));
            cursor.into_inner()
        })
    });
}

fn decode_single_varint(c: &mut Criterion) {
    let ws = create_simple_struct();
    let mut cursor = Cursor::new(Vec::with_capacity(16));
    let _ = protowirers::encode::encode_wire_binary(&mut cursor, vec![ws]);
    let encoded = cursor.into_inner();

    c.bench_function("decode_single_varint", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&encoded as &[u8]));
            let _ = protowirers::decode::decode_wire_binary(&mut cursor);
        })
    });
}

fn decode_complex_message(c: &mut Criterion) {
    let wss = create_complex_struct();
    let mut cursor = Cursor::new(Vec::with_capacity(128));
    let _ = protowirers::encode::encode_wire_binary(&mut cursor, wss);
    let encoded = cursor.into_inner();

    c.bench_function("decode_complex_message", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&encoded as &[u8]));
            let _ = protowirers::decode::decode_wire_binary(&mut cursor);
        })
    });
}

fn roundtrip_single_varint(c: &mut Criterion) {
    c.bench_function("roundtrip_single_varint", |b| {
        let ws = create_simple_struct();
        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(16));
            let _ =
                protowirers::encode::encode_wire_binary(&mut cursor, black_box(vec![ws.clone()]));
            let encoded = cursor.into_inner();
            let mut cursor = Cursor::new(black_box(&encoded as &[u8]));
            let _ = protowirers::decode::decode_wire_binary(&mut cursor);
        })
    });
}

fn roundtrip_complex_message(c: &mut Criterion) {
    c.bench_function("roundtrip_complex_message", |b| {
        let wss = create_complex_struct();
        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(128));
            let _ = protowirers::encode::encode_wire_binary(&mut cursor, black_box(wss.clone()));
            let encoded = cursor.into_inner();
            let mut cursor = Cursor::new(black_box(&encoded as &[u8]));
            let _ = protowirers::decode::decode_wire_binary(&mut cursor);
        })
    });
}

// 大量のVarintエンコード/デコードのベンチマーク
fn encode_many_varints(c: &mut Criterion) {
    c.bench_function("encode_many_varints", |b| {
        let varints: Vec<WireStruct> = (0..1000)
            .map(|i| WireStruct::new(i as u128, WireData::Varint(WireDataVarint::new(i as u128))))
            .collect();

        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(8192));
            let _ =
                protowirers::encode::encode_wire_binary(&mut cursor, black_box(varints.clone()));
            cursor.into_inner()
        })
    });
}

// 長い文字列のエンコード/デコードのベンチマーク
fn encode_long_string(c: &mut Criterion) {
    c.bench_function("encode_long_string", |b| {
        let long_string = "a".repeat(1000);
        let ws = WireStruct::new(
            1,
            WireData::LengthDelimited(WireDataLengthDelimited::new(long_string.into_bytes())),
        );

        b.iter(|| {
            let mut cursor = Cursor::new(Vec::with_capacity(1024));
            let _ =
                protowirers::encode::encode_wire_binary(&mut cursor, black_box(vec![ws.clone()]));
            cursor.into_inner()
        })
    });
}

criterion_group!(
    benches,
    encode_single_varint,
    encode_complex_message,
    decode_single_varint,
    decode_complex_message,
    roundtrip_single_varint,
    roundtrip_complex_message,
    encode_many_varints,
    encode_long_string
);
criterion_main!(benches);
