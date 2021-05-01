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
        #[def(field_num = 1, def_type = "uint32")]
        u_uint32: u32,
        #[def(field_num = 2, def_type = "uint64")]
        u_uint64: u64,
        #[def(field_num = 3, def_type = "sint32")]
        s_sint32: i32,
        #[def(field_num = 4, def_type = "sint64")]
        s_sint64: i64,
        #[def(field_num = 5, def_type = "int32")]
        i_int32: i32,
        #[def(field_num = 6, def_type = "int64")]
        i_int64: i64,
        #[def(field_num = 7, def_type = "bool")]
        b_bool: bool,
    }
    let bytes: &[u8] = &[
        0b00001000, 0b00000001, // field_num = 1
        0b00010000, 0b01100100, // field_num = 2
        0b00011000, 0b11110011, 0b00000001, // field_num = 3
        0b00100000, 0b10101001, 0b11110111, 0b00000110, // field_num = 4
        0b00101000, 0b11101001, 0b11001101, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
        0b11111111, 0b11111111, 0b11111111, 0b00000001, // field_num = 5
        0b00110000, 0b10000010, 0b10100110, 0b00001110, // field_num = 6
        0b00111000, 0b00000001, // field_num = 7
    ];
    {
        let x = Sample::parse(bytes).unwrap();
        assert_eq!(x.u_uint32, 1);
        assert_eq!(x.u_uint64, 100);
        assert_eq!(x.s_sint32, -122);
        assert_eq!(x.s_sint64, -56789);
        assert_eq!(x.i_int32, -6423);
        assert_eq!(x.i_int64, 234242);
        assert!(x.b_bool);
        let x = x.bytes().unwrap();
        assert_eq!(
            x,
            vec![
                0b00001000, 0b00000001, // field_num = 1
                0b00010000, 0b01100100, // field_num = 2
                0b00011000, 0b11110011, 0b00000001, // field_num = 3
                0b00100000, 0b10101001, 0b11110111, 0b00000110, // field_num = 4
                0b00101000, 0b11101001, 0b11001101, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                0b11111111, 0b11111111, 0b11111111, 0b00000001, // field_num = 5
                0b00110000, 0b10000010, 0b10100110, 0b00001110, // field_num = 6
                0b00111000, 0b00000001, // field_num = 7
            ]
        );
    }
    {
        let bytes: &[u8] = &[
            0b00001000, 0b00000001, // field_num = 1
            0b00010000, 0b01100100, // field_num = 2
            0b00011000, 0b11110011, 0b00000001, // field_num = 3
            0b00100000, 0b10101001, 0b11110111, 0b00000110, // field_num = 4
            0b00101000, 0b10010111, 0b00110010, // field_num = 5
            0b00110000, 0b10000010, 0b10100110, 0b00001110, // field_num = 6
            0b00111000, 0b00000001, // field_num = 7
        ];
        let x = Sample::parse(bytes).unwrap();
        assert_eq!(x.u_uint32, 1);
        assert_eq!(x.u_uint64, 100);
        assert_eq!(x.s_sint32, -122);
        assert_eq!(x.s_sint64, -56789);
        assert_eq!(x.i_int32, 6423); // ここを正の数でも確認する
        assert_eq!(x.i_int64, 234242);
        assert!(x.b_bool);
        let x = x.bytes().unwrap();
        assert_eq!(
            x,
            vec![
                0b00001000, 0b00000001, // field_num = 1
                0b00010000, 0b01100100, // field_num = 2
                0b00011000, 0b11110011, 0b00000001, // field_num = 3
                0b00100000, 0b10101001, 0b11110111, 0b00000110, // field_num = 4
                0b00101000, 0b10010111, 0b00110010, // field_num = 5
                0b00110000, 0b10000010, 0b10100110, 0b00001110, // field_num = 6
                0b00111000, 0b00000001, // field_num = 7
            ]
        );
    }
    {
        let bytes: &[u8] = &[
            0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110011, 0b00000001,
            0b00100000, 0b10101001, 0b11110111, 0b00000110, 0b00101000, 0b11101001, 0b11001101,
            0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
            0b00000001, 0b00110000, 0b11111110, 0b11011001, 0b11110001, 0b11111111, 0b11111111,
            0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001, 0b00111000, 0b00000001,
        ];
        let x = Sample::parse(bytes).unwrap();
        assert_eq!(x.u_uint32, 1);
        assert_eq!(x.u_uint64, 100);
        assert_eq!(x.s_sint32, -122);
        assert_eq!(x.s_sint64, -56789);
        assert_eq!(x.i_int32, -6423);
        assert_eq!(x.i_int64, -234242);
        assert!(x.b_bool);
        let x = x.bytes().unwrap();
        assert_eq!(
            x,
            vec![
                0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110011, 0b00000001,
                0b00100000, 0b10101001, 0b11110111, 0b00000110, 0b00101000, 0b11101001, 0b11001101,
                0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                0b00000001, 0b00110000, 0b11111110, 0b11011001, 0b11110001, 0b11111111, 0b11111111,
                0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b00000001, 0b00111000, 0b00000001,
            ]
        );
    }
    {
        let bytes: &[u8] = &[
            0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110100, 0b00000001,
            0b00100000, 0b10101010, 0b11110111, 0b00000110, 0b00101000, 0b10010111, 0b00110010,
            0b00110000, 0b10000010, 0b10100110, 0b00001110, 0b00111000, 0b00000001,
        ];
        let x = Sample::parse(bytes).unwrap();
        assert_eq!(x.u_uint32, 1);
        assert_eq!(x.u_uint64, 100);
        assert_eq!(x.s_sint32, 122);
        assert_eq!(x.s_sint64, 56789);
        assert_eq!(x.i_int32, 6423);
        assert_eq!(x.i_int64, 234242);
        assert!(x.b_bool);
        let x = x.bytes().unwrap();
        assert_eq!(
            x,
            vec![
                0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110100, 0b00000001,
                0b00100000, 0b10101010, 0b11110111, 0b00000110, 0b00101000, 0b10010111, 0b00110010,
                0b00110000, 0b10000010, 0b10100110, 0b00001110, 0b00111000, 0b00000001,
            ]
        );
    }
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
