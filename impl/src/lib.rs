extern crate proc_macro;

mod ast;
mod expand;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Proto, attributes(def))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
