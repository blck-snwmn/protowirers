extern crate proc_macro;

mod ast;

use ast::Input;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Proto, attributes(def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).into()
}

fn expand(input: DeriveInput) -> proc_macro2::TokenStream {
    let input_indent = format_ident!("{}", input.ident);
    let input = Input::from_syn(&input);
    if let Err(e) = input {
        return e.to_compile_error().into();
    }
    let Input::Struct(data) = input.unwrap();

    let init_fields = data.build_declare_for_init();

    let build_fields = data.build_struct_fields();

    let build_parse_fields = data.build_match_case();
    if let Err(e) = build_parse_fields {
        return e.to_compile_error().into();
    }

    let build_parse_fields = build_parse_fields.unwrap();

    quote! {
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
                        #build_parse_fields
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
    }
}
