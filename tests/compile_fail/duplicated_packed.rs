use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(field_num = 1, def_type = "int32", packed, packed)]
    s: u32,
}

fn main() {}
