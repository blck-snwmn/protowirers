use crate::ast::{Input, Struct};
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn derive(node: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let input_indent = format_ident!("{}", node.ident);
    match Input::from_syn(&node)? {
        Input::Struct(data) => Ok(gen_struct(data, input_indent)),
        // _ => Err(syn::Error::new_spanned(
        //     input_indent,
        //     "struct, enum only suport",
        // )),
    }
}

fn gen_struct(data: Struct, input_indent: syn::Ident) -> proc_macro2::TokenStream {
    // TODO エラーメッセージ改善
    // atribute自体がエラーの場合、() が表示されてしまう, など
    let init_fields = data.build_declare_for_init();
    let build_fields = data.build_struct_fields();
    let build_parse_fields = data.build_match_case();
    let build_gen_wirestructs = data.build_gen_wirestructs();

    quote! {
        impl protowirers::wire::Proto for #input_indent{
            fn parse(bytes: &[u8])->anyhow::Result<Self>{
                use protowirers::parser::*;

                let mut c = std::io::Cursor::new(bytes);
                let result = decode::decode_wire_binary(&mut c)?;

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
            fn bytes(&self)-> anyhow::Result<Vec<u8>>{
                use protowirers::parser::*;

                let inputs = vec![
                    #build_gen_wirestructs
                ];
                let mut c = std::io::Cursor::new(Vec::new());
                encode::encode_wire_binary(&mut c, inputs)?;
                Ok(c.into_inner())
            }
        }
    }
}
