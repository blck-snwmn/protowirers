use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(field_num = 1, def_type = "sint64")]
    s: u32,
}

fn main() {}
