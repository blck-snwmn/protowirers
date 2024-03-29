# protowirers
[![test suite](https://github.com/blck-snwmn/protowirers/actions/workflows/ci.yml/badge.svg)](https://github.com/blck-snwmn/protowirers/actions/workflows/ci.yml)

sampel to encode/decode binary wire format for protocol buffer messages

## Example

### Struct

Specify `Proto`

```rust
#[derive(Proto, Default, Clone)]
struct Sample {
    #[def(field_num = 1, def_type = "sint64")]
    s_sint64: i64,
    #[def(field_num = 2, def_type = "fixed64")]
    f_fixed64: u64,
    #[def(field_num = 3, def_type = "string")]
    s_string: String,
    #[def(field_num = 4, def_type = "fixed32")]
    f_fixed32: u32,
    #[def(field_num = 5, def_type = "uint32", packed, repeated)]
    r_u_int32: Vec<u32>,
}
```

Implement the following function

```rust
fn parse(bytes: &[u8])->anyhow::Result<Self>{}
fn bytes(&self)-> anyhow::Result<Vec<u8>>{}
```

### Enum

Specify `Proto`
Last Variants must be tupple to have 1 args(i32).

```rust
#[derive(Proto, Clone, Copy, PartialEq, Eq, Debug)]
enum TestEnum {
    Value1,
    Value2,
    Value3,
    ValueOther(i32),
}
```
