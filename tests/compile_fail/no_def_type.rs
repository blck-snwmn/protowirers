use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(field_num = 1, field_num = 1)]
    s: u32,
}

fn main() {}
