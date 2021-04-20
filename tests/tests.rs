use protowirers::*;

#[test]
fn test_can_call() {
    #[derive(Proto)]
    struct Sample {}
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010100];
    let x = Sample::parse(bytes).unwrap();
    let _ = x.bytes();
}

#[test]
fn test_mapping() {
    #[derive(Proto)]
    struct Sample {
        #[def(field_num = 1, def_type = "int32")]
        s: u32,
        #[def(field_num = 2, def_type = "sint64")]
        x: i64,
    }
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010011];
    let x = Sample::parse(bytes).unwrap();
    assert_eq!(x.s, 2);
    assert_eq!(x.x, -10);
    let x = x.bytes().unwrap();
    assert_eq!(x, vec![0b00001000, 0b00000010, 0b00010000, 0b00010011]);
}

#[test]
fn test_mapping_change_order_field_num() {
    #[derive(Proto)]
    struct Sample {
        #[def(field_num = 2, def_type = "int32")]
        age: u32,
        #[def(field_num = 1, def_type = "sint64")]
        score: i64,
    }
    let bytes: &[u8] = &[0b00010000, 0b00000010, 0b00001000, 0b00010011];
    let x = Sample::parse(bytes).unwrap();
    assert_eq!(x.age, 2);
    assert_eq!(x.score, -10);
    let x = x.bytes().unwrap();
    assert_eq!(x, vec![0b00010000, 0b00000010, 0b00001000, 0b00010011]);
}
