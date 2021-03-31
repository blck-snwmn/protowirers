extern crate proc_macro;
use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};
#[proc_macro_derive(Proto, attributes(def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = format_ident!("{}", input.ident);

    let data = match input.data {
        syn::Data::Struct(s) => Some(s),
        _ => None,
    };
    let data = data.unwrap();

    let build_fields = data.fields.iter().map(|f| {
        let filed_indent = &f.ident;
        quote! {
            #filed_indent
        }
    });
    let init_fields = data.fields.iter().map(|f| {
        let filed_indent = &f.ident;
        let filed_ty = &f.ty;
        // 一旦固定値は0で。
        quote! {
            let mut #filed_indent: #filed_ty = 0;
        }
    });
    let build_parse_fields = data.fields.iter().map(|f| {
        let filed_indent = &f.ident;
        let x = f.attrs.iter().find_map(|a| {
            a.parse_meta().ok().and_then(|m| match m {
                syn::Meta::List(ml) if ml.path.is_ident("def") => Some(ml),
                _ => None,
            })
        });
        if x.is_none() {
            return syn::Error::new_spanned(&f, "expected `def(\"...\")`")
                .to_compile_error()
                .into();
        }
        // TODO エラーメッセージをリッチにする
        let x = x.unwrap();
        if x.nested.len() != 2 {
            return syn::Error::new_spanned(x.path, "zzz")
                .to_compile_error()
                .into();
        }
        let mnv_map: HashMap<String, &syn::Lit> = x
            .nested
            .iter()
            .filter_map(|nm| match nm {
                syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value))
                    if meta_name_value.path.is_ident("field_num")
                        || meta_name_value.path.is_ident("def_type") =>
                {
                    Some(meta_name_value)
                }
                _ => None,
            })
            .map(|mnv| (mnv.path.get_ident().unwrap().to_string(), &mnv.lit))
            .collect();

        if mnv_map.len() != 2 {
            return syn::Error::new_spanned(x.path, "xxx")
                .to_compile_error()
                .into();
        }
        let fieild_num = mnv_map.get("field_num").and_then(|fnum| match fnum {
            syn::Lit::Int(v) => Some(v),
            _ => None,
        });
        if fieild_num.is_none() {
            return syn::Error::new_spanned(x.path, "field_num is not exist")
                .to_compile_error()
                .into();
        }
        let fieild_num = fieild_num.unwrap();

        let def_type = mnv_map
            .get("def_type")
            .and_then(|fnum| match fnum {
                syn::Lit::Str(v) => Some(v.value()),
                _ => None,
            })
            .and_then(|dt| match dt.as_str() {
                "int32" => Some(quote! {parser::parse_u32}),
                "sint64" => Some(quote! {parser::parse_i64}),
                _ => None,
            });
        if def_type.is_none() {
            return syn::Error::new_spanned(x.path, "def_type is not exist")
                .to_compile_error()
                .into();
        }
        let def_type = def_type.unwrap();

        quote! {
            (#fieild_num, reader::WireType::Varint(v)) => {
                #filed_indent = #def_type(*v)?;
            }
        }
    });

    let init_fields = quote! {
        #(#init_fields)*
    };

    let build_fields = quote! {
        #(#build_fields,)*
    };
    let build_parse_fields = quote! {
        #(#build_parse_fields,)*
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
                        // (1, reader::WireType::Varint(v)) => {
                        //     s = parser::parse_u32(*v)?;
                        // }
                        // (2, reader::WireType::Varint(v)) => {
                        //     x = parser::parse_i64(*v)?;
                        // }
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
    };
    q.into()
}
