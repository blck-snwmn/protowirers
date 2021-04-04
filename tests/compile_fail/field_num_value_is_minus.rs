use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(field_num = -100, def_type = "int32")]
    s: u32,
}

fn main() {}
