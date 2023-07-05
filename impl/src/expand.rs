use crate::ast::{Enum, Input, Struct};
use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn derive(node: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let input_indent = format_ident!("{}", node.ident);
    match Input::from_syn(node)? {
        Input::Struct(data) => Ok(gen_struct(data, input_indent)),
        Input::Enum(data) => Ok(gen_enum(data, input_indent)),
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

fn gen_enum(data: Enum, input_indent: syn::Ident) -> proc_macro2::TokenStream {
    // impl
    // ```rust
    // impl protowirers::parser::VariantEnum for Test {}
    // impl From<i32> for Test {
    //     fn from(i: i32) -> Self {
    //         match i {
    //             0 => Test::Value1,
    //             1 => Test::Value2,
    //             2 => Test::Value3,
    //             i => Test::ValueOther(i),
    //         }
    //     }
    // }
    // impl From<Test> for i32 {
    //     fn from(v: Test) -> Self {
    //         match v {
    //             Test::Value1 => 0,
    //             Test::Value2 => 1,
    //             Test::Value3 => 2,
    //             Test::ValueOther(vv) => vv,
    //         }
    //     }
    // }
    // impl Default for Test {
    //     fn default() -> Self {
    //         Test::Value1
    //     }
    // }
    // ```

    let last_index = data.variants.len() - 1;
    // TODO とりあえず unwrap
    // default は先頭要素
    let first_ident = data.variants.first().unwrap();

    let idents: Vec<(usize, &syn::Ident)> = data
        .variants
        .iter()
        .enumerate()
        .map(|(index, v)| (index, &v.ident))
        .collect();

    // 最後の要素以外は引数を持たない
    // 最後の要素は１つだけ引数を持つ
    // TODO これをチェックすること！
    let froms = idents.iter().map(|(index, i)| {
        if *index == last_index {
            quote! { i => #input_indent::#i(i)}
        } else {
            let index = *index as i32;
            quote! { #index => #input_indent::#i}
        }
    });
    let from = quote! {
        #(#froms,)*
    };
    let tos = idents.iter().map(|(index, i)| {
        if *index == last_index {
            quote! { #input_indent::#i(i) => i }
        } else {
            let index = *index as i32;
            quote! { #input_indent::#i => #index }
        }
    });
    let to = quote! {
        #(#tos,)*
    };
    quote! {
        impl protowirers::parser::VariantEnum for #input_indent {}
        impl From<i32> for #input_indent {
            fn from(i: i32) -> Self {
                match i {
                    #from
                }
            }
        }
        impl From<#input_indent> for i32 {
            fn from(v: #input_indent) -> Self {
                match v {
                    #to
                }
            }
        }
        impl Default for #input_indent {
            fn default() -> Self {
                #input_indent::#first_ident
            }
        }
    }
}
