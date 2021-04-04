extern crate proc_macro;

mod ast;

use ast::Input;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Proto, attributes(def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let input_indent = format_ident!("{}", input.ident);
    let Input::Struct(data) = Input::from_syn(&input)?;

    let init_fields = data.build_declare_for_init();
    let build_fields = data.build_struct_fields();
    let build_parse_fields = data.build_match_case();

    Ok(quote! {
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
    })
}
