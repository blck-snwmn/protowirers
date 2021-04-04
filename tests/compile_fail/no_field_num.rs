use protowirers::*;

#[derive(Proto)]
struct Sample {
    #[def(def_type = "int32")]
    s: u32,
}

fn main() {}
