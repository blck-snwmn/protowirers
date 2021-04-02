use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(field_num = 1, def_type = "xint32")]
    s: u32,
}

fn main() {}
