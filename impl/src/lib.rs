extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};
#[proc_macro_derive(Proto, attributes(proto_def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = format_ident!("{}", input.ident);

    let data = match input.data {
        syn::Data::Struct(s) => Some(s),
        _ => None,
    };
    let data = data.unwrap();

    let mut init_fields = Vec::new();
    let mut build_fields = Vec::new();
    for f in data.fields {
        let filed_indent = f.ident;
        let filed_ty = f.ty;
        // 一旦固定値は0で。
        init_fields.push(quote! {
            let mut #filed_indent: #filed_ty = 0;
        });
        build_fields.push(quote! {
            #filed_indent
        });
    }
    let init_fields = quote! {
        #(#init_fields)*
    };

    let build_fields = quote! {
        #(#build_fields,)*
    };

    let q = quote! {
        use std::io::Cursor;
        use anyhow::Result;
        use protowirers::{parser, reader};

        impl #input_indent{
            pub fn parse(bytes: &[u8])->Result<Self>{
                let mut c = Cursor::new(bytes);
                let result = reader::read_wire_binary(&mut c)?;

                #init_fields
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
                Ok(Self {
                    #build_fields
                })
            }
            pub fn bytes(&self)-> Vec<u8>{
                Vec::new()
            }
        }
    };
    q.into()
}
