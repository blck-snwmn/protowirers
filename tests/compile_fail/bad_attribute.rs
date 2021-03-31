#[derive(Proto)]
struct Sample {
    #[proto_def(field_num=1, p_type=int32)]
    s: u32,
    #[proto_def(field_num=2, p_type=int32)]
    x: i64,
}
