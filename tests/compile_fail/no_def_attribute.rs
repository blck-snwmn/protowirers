use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[defx(field_num = 1, def_type = "int32")]
    s: u32,
}

fn main() {}
