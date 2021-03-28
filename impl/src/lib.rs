extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};
#[proc_macro_derive(Proto, attributes(proto_def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = format_ident!("{}", input.ident);
    let q = quote! {
        use std::io::Cursor;
        use anyhow::Result;
        use protowirers::{parser, reader};

        impl #input_indent{
            pub fn parse(bytes: &[u8])->Result<Self>{
                let mut c = Cursor::new(bytes);
                let result = reader::read_wire_binary(&mut c)?;

                let mut s: u32 = 0;
                let mut x: i64 = 0;
                for sw in result {
                    match (sw.field_number(), sw.wire_type()) {
                        (1, reader::WireType::Varint(v)) => {
                            s = parser::parse_u32(*v)?;
                        }
                        (2, reader::WireType::Varint(v)) => {
                            x = parser::parse_i64(*v)?;
                        }
                        _ => (),
                    }
                }
                Ok(Self { s, x })
            }
            pub fn bytes(&self)-> Vec<u8>{
                Vec::new()
            }
        }
    };
    q.into()
}
