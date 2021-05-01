use protowirers::*;

#[test]
fn test_can_call() {
    #[derive(Proto)]
    struct Sample {}
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010100];
    let x = Sample::parse(bytes).unwrap();
    // let _ = x.bytes();
}

#[test]
fn test_mapping() {
    #[derive(Proto)]
    struct Sample {
        #[def(field_num = 1, def_type = "uint32")]
        s: u32,
        #[def(field_num = 2, def_type = "sint64")]
        x: i64,
    }
    let bytes: &[u8] = &[0b00001000, 0b00000010, 0b00010000, 0b00010011];
    let x = Sample::parse(bytes).unwrap();
    assert_eq!(x.s, 2);
    assert_eq!(x.x, -10);
    // let x = x.bytes().unwrap();
    // assert_eq!(x, vec![0b00001000, 0b00000010, 0b00010000, 0b00010011]);
}

#[test]
fn test_mapping_change_order_field_num() {
    #[derive(Proto)]
    struct Sample {
        #[def(field_num = 2, def_type = "uint32")]
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

#[test]
fn test_repeated_field() {
    #[derive(Proto)]
    struct Sample {
        #[def(field_num = 4, def_type = "string")]
        str_field: String,
        #[def(field_num = 9, def_type = "sint64", repeated, packed)]
        vec_field: Vec<i64>,
    }
    let bytes: &[u8] = &[
        0b00100010, 0b00000011, 0b01100001, 0b01100010, 0b01100011, 0b01001010, 0b00000110,
        0b10000010, 0b10000100, 0b10101111, 0b01011111, 0b00000100, 0b00000110,
    ];
    let x = Sample::parse(bytes).unwrap();
    assert_eq!(x.str_field, "abc");
    assert_eq!(x.vec_field, vec![100000001, 2, 3,]);
}
