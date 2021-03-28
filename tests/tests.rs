use protowirers::*;
#[derive(Proto)]
struct Sample {
    // #[proto_def(field_num=1, def_type=int32)]
    s: u32,
    // #[proto_def(field_num=2, def_type=int64)]
    x: i64,
}
#[test]
fn test_can_call() {
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010100];
    let x = Sample::parse(bytes).unwrap();
    x.bytes();
}

#[test]
fn test_mapping() {
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010011];
    let x = Sample::parse(bytes).unwrap();
    assert_eq!(x.s, 2);
    assert_eq!(x.x, -10);
}
