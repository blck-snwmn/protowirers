use protowirers::wire::*;
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
    #[derive(Proto, Clone, Copy, PartialEq, Eq, Debug)]
    enum TestEnum {
        Value1,
        Value2,
        Value3,
        ValueOther(i32),
    }
    #[derive(Proto, Default, Clone)]
    struct Inner {
        #[def(field_num = 1, def_type = "int32")]
        i_int32: i32,
        #[def(field_num = 2, def_type = "string")]
        s_string: String,
        #[def(field_num = 3, def_type = "sint64")]
        s_int64: i64,
        #[def(field_num = 4, def_type = "fixed64")]
        f_fixed64: u64,
        #[def(field_num = 5, def_type = "sfixed32")]
        s_sfixed32: i32,
    }
    #[derive(Proto, Default, Clone)]
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
        #[def(field_num = 8, def_type = "enum")]
        t_test_enum: TestEnum,

        #[def(field_num = 9, def_type = "fixed64")]
        f_fixed64: u64,
        #[def(field_num = 10, def_type = "sfixed64")]
        s_sfixed64: i64,
        #[def(field_num = 11, def_type = "double")]
        d_double: f64,

        #[def(field_num = 12, def_type = "string")]
        s_string: String,
        #[def(field_num = 13, def_type = "bytes")]
        b_bytes: Vec<u8>,
        #[def(field_num = 14, def_type = "embedded")]
        i_inner: Inner,

        #[def(field_num = 16, def_type = "fixed32")]
        f_fixed32: u32,
        #[def(field_num = 17, def_type = "sfixed32")]
        s_sfixed32: i32,
        #[def(field_num = 18, def_type = "float")]
        f_float: f32,
        #[def(field_num = 101, def_type = "uint32", packed, repeated)]
        r_u_int32: Vec<u32>,
    }
    let bytes: &[u8] = &[
        0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110011, 0b00000001,
        0b00100000, 0b10101001, 0b11110111, 0b00000110, 0b00101000, 0b11101001, 0b11001101,
        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
        0b00000001, 0b00110000, 0b10000010, 0b10100110, 0b00001110, 0b00111000, 0b00000001,
        0b01000000, 0b00000001, 0b01001001, 0b10111011, 0b00000001, 0b00000000, 0b00000000,
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b01010001, 0b01011011, 0b11111101,
        0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b01011001,
        0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110100,
        0b00111111, 0b01100010, 0b00000100, 0b01100110, 0b01110011, 0b01100100, 0b01100101,
        0b01101010, 0b00000100, 0b00000100, 0b00000101, 0b00000110, 0b00000111, 0b01110010,
        0b00011010, 0b00001000, 0b00001010, 0b00010010, 0b00000101, 0b01101000, 0b01100101,
        0b01101100, 0b01101100, 0b01101111, 0b00011000, 0b10010101, 0b01000111, 0b00100001,
        0b00110111, 0b00000010, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
        0b00000000, 0b00101101, 0b10011100, 0b11111111, 0b11111111, 0b11111111, 0b10000101,
        0b00000001, 0b00101011, 0b00000000, 0b00000000, 0b00000000, 0b10001101, 0b00000001,
        0b00100000, 0b00000100, 0b00000000, 0b00000000, 0b10010101, 0b00000001, 0b00000110,
        0b10000001, 0b01001101, 0b01000000, 0b10101010, 0b00000110, 0b00000011, 0b00001010,
        0b00010100, 0b00011110,
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
        assert_eq!(x.t_test_enum, TestEnum::Value2);
        assert_eq!(x.f_fixed64, 443);
        assert_eq!(x.s_sfixed64, -677);
        let error_margin_f64 = f64::EPSILON;
        assert!((x.d_double - 1.25f64).abs() < error_margin_f64);
        assert_eq!(x.s_string, "fsde".to_string());
        assert_eq!(x.b_bytes, vec![4, 5, 6, 7]);
        assert_eq!(x.i_inner.i_int32, 10);
        assert_eq!(x.i_inner.s_string, "hello".to_string());
        assert_eq!(x.i_inner.s_int64, -4555);
        assert_eq!(x.i_inner.f_fixed64, 567);
        assert_eq!(x.i_inner.s_sfixed32, -100);
        assert_eq!(x.f_fixed32, 43);
        assert_eq!(x.s_sfixed32, 1056);
        let error_margin_f32 = f32::EPSILON;
        assert!((x.f_float - 3.211).abs() < error_margin_f32);
        assert_eq!(x.r_u_int32, vec![10, 20, 30]);
        let x = x.bytes().unwrap();
        assert_eq!(
            x,
            vec![
                0b00001000, 0b00000001, 0b00010000, 0b01100100, 0b00011000, 0b11110011, 0b00000001,
                0b00100000, 0b10101001, 0b11110111, 0b00000110, 0b00101000, 0b11101001, 0b11001101,
                0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
                0b00000001, 0b00110000, 0b10000010, 0b10100110, 0b00001110, 0b00111000, 0b00000001,
                0b01000000, 0b00000001, 0b01001001, 0b10111011, 0b00000001, 0b00000000, 0b00000000,
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b01010001, 0b01011011, 0b11111101,
                0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b01011001,
                0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b11110100,
                0b00111111, 0b01100010, 0b00000100, 0b01100110, 0b01110011, 0b01100100, 0b01100101,
                0b01101010, 0b00000100, 0b00000100, 0b00000101, 0b00000110, 0b00000111, 0b01110010,
                0b00011010, 0b00001000, 0b00001010, 0b00010010, 0b00000101, 0b01101000, 0b01100101,
                0b01101100, 0b01101100, 0b01101111, 0b00011000, 0b10010101, 0b01000111, 0b00100001,
                0b00110111, 0b00000010, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
                0b00000000, 0b00101101, 0b10011100, 0b11111111, 0b11111111, 0b11111111, 0b10000101,
                0b00000001, 0b00101011, 0b00000000, 0b00000000, 0b00000000, 0b10001101, 0b00000001,
                0b00100000, 0b00000100, 0b00000000, 0b00000000, 0b10010101, 0b00000001, 0b00000110,
                0b10000001, 0b01001101, 0b01000000, 0b10101010, 0b00000110, 0b00000011, 0b00001010,
                0b00010100, 0b00011110,
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
        assert_eq!(x.f_fixed64, 0);
        assert_eq!(x.s_sfixed64, 0);
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
        // 170, 6, 4, 3, 10, 20, 30
        // 10101010
        // 00000110
        // 00000100
        // 00000011
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
