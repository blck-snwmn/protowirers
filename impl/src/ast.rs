use quote::quote;
pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a syn::DeriveInput) -> syn::Result<Self> {
        match &node.data {
            syn::Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            syn::Data::Enum(data) => Ok(Input::Enum(Enum::from_syn(node, data))),
            _ => Err(syn::Error::new_spanned(node, "suport data is only Sturct")),
        }
    }
}

pub struct Enum<'a> {
    pub original: &'a syn::DeriveInput,
    pub variants: Vec<&'a syn::Variant>,
}

impl<'a> Enum<'a> {
    fn from_syn(node: &'a syn::DeriveInput, data: &'a syn::DataEnum) -> Self {
        use std::iter::FromIterator;
        Enum {
            original: node,
            variants: Vec::from_iter(&data.variants),
        }
    }
}

pub struct Struct<'a> {
    pub original: &'a syn::DeriveInput,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Struct<'a> {
    fn from_syn(node: &'a syn::DeriveInput, data: &'a syn::DataStruct) -> syn::Result<Self> {
        Ok(Struct {
            original: node,
            fields: Field::from_syns(&data.fields)?,
        })
    }
    // build_struct_fields は パース結果の値を構造体にマッピング部を組み立てます
    pub fn build_struct_fields(&self) -> proc_macro2::TokenStream {
        let build_fields = self.fields.iter().map(|f| f.build_struct_fields());
        quote! {
            #(#build_fields,)*
        }
    }

    // declare_for_init は パース処理における各フィールドの初期化部を組み立てます
    // 現在はすべてのフィールドを初期化するため、入力データに値がない場合でも正常終了します
    // また、現時点での初期化は 数値型のみ機能しています。
    pub fn build_declare_for_init(&self) -> proc_macro2::TokenStream {
        let init_fields = self.fields.iter().map(|f| f.build_declare_for_init());
        quote! {
            #(#init_fields)*
        }
    }

    // build_match_in_parse は パーサーのmatch部の処理を組み立てます
    pub fn build_match_case(&self) -> proc_macro2::TokenStream {
        let build_parse_fields = self.fields.iter().map(|f| f.build_match_case());
        // .try_fold(
        //     Vec::new(),
        //     |mut acc, r: syn::Result<proc_macro2::TokenStream>| {
        //         r.and_then(|rr| {
        //             acc.push(rr);
        //             Ok(acc)
        //         })
        //     },
        // )?;
        quote! {
            #(#build_parse_fields,)*
        }
    }

    #[allow(dead_code)]
    pub fn build_gen_wirestructs(&self) -> proc_macro2::TokenStream {
        let build_gen_wirestructs = self.fields.iter().map(|f| f.build_gen_wirestructs());
        quote! {
            #(#build_gen_wirestructs,)*
        }
    }
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attr: Attribute<'a>,
}
impl<'a> Field<'a> {
    fn from_syns(data: &'a syn::Fields) -> syn::Result<Vec<Self>> {
        data.iter().map(Field::from_syn).collect()
    }
    fn from_syn(f: &'a syn::Field) -> syn::Result<Self> {
        // TODO 番号がだぶってないかチェックする
        let attr = Attribute::from_syn(&f.attrs, f)?;
        let ty = &f.ty;
        // match ty {
        //     syn::Type::Array(_) => Err(syn::Error::new_spanned(&ty, "Array"))?,
        //     syn::Type::BareFn(_) => Err(syn::Error::new_spanned(&ty, "BareFn"))?,
        //     syn::Type::Group(_) => Err(syn::Error::new_spanned(&ty, "Group"))?,
        //     syn::Type::ImplTrait(_) => Err(syn::Error::new_spanned(&ty, "ImplTrait"))?,
        //     syn::Type::Infer(_) => Err(syn::Error::new_spanned(&ty, "Infer"))?,
        //     syn::Type::Macro(_) => Err(syn::Error::new_spanned(&ty, "Macro"))?,
        //     syn::Type::Never(_) => Err(syn::Error::new_spanned(&ty, "Never"))?,
        //     syn::Type::Paren(_) => Err(syn::Error::new_spanned(&ty, "Paren"))?,
        //     syn::Type::Path(_) => Err(syn::Error::new_spanned(&ty, "Path"))?,
        //     syn::Type::Ptr(_) => Err(syn::Error::new_spanned(&ty, "Ptr"))?,
        //     syn::Type::Reference(_) => Err(syn::Error::new_spanned(&ty, "Reference"))?,
        //     syn::Type::Slice(_) => Err(syn::Error::new_spanned(&ty, "Slice"))?,
        //     syn::Type::TraitObject(_) => Err(syn::Error::new_spanned(&ty, "TraitObject"))?,
        //     syn::Type::Tuple(_) => Err(syn::Error::new_spanned(&ty, "Tuple"))?,
        //     syn::Type::Verbatim(_) => Err(syn::Error::new_spanned(&ty, "Verbatim"))?,
        //     syn::Type::__TestExhaustive(_) => {
        //         Err(syn::Error::new_spanned(&ty, "__TestExhaustive"))?
        //     }
        // }
        if !attr.allows_rust_type(ty) {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "defined def_type `{:?}` does not match this Rust type",
                    attr.def_type,
                ),
            ));
        }
        Ok(Self { original: f, attr })
    }
    fn build_struct_fields(&self) -> proc_macro2::TokenStream {
        let filed_indent = &self.original.ident;
        // すべてOptionalとして扱い、値が設定されていないフィールドはdefault値にする
        quote! {
            #filed_indent: #filed_indent.unwrap_or_default()
        }
    }
    fn build_declare_for_init(&self) -> proc_macro2::TokenStream {
        let f = self.original;
        let filed_indent = &f.ident;
        let filed_ty = &f.ty;
        // Noneで初期化
        quote! {
            let mut #filed_indent: Option<#filed_ty> = None;
        }
    }

    fn build_match_case(&self) -> proc_macro2::TokenStream {
        let filed_indent = &self.original.ident;
        let a = &self.attr;
        let fieild_num = a.filed_num as u128;
        let wire_data_type = a.def_type.to_input_wire_data_type();

        // repeated & packed は LengthDelimited として扱う
        if self.attr.repeated && self.attr.packed {
            return quote! {
                (#fieild_num, protowirers::wire::WireData::LengthDelimited(v)) => {
                    // #filed_indent = Some(#def_type(v)?);
                    let vv = protowirers::wire::TypeLengthDelimited::PackedRepeatedFields(
                        protowirers::wire::AllowedPakcedType::Variant(#wire_data_type)
                    );
                    #filed_indent = Some(v.parse(vv)?);
                }
            };
        }

        let mach_wire_type = a.def_type.to_corresponding_wire_type();
        quote! {
            (#fieild_num, #mach_wire_type(v)) => {
                // #filed_indent = Some(#def_type(v)?);
                #filed_indent = Some(v.parse(#wire_data_type)?);
            }
        }
    }

    fn build_gen_wirestructs(&self) -> proc_macro2::TokenStream {
        let filed_indent = &self.original.ident;
        let a = &self.attr;
        let fieild_num = a.filed_num as u128;
        let wt = a.def_type.to_corresponding_wire_type();
        let wdt = a.def_type.to_input_wire_data_type();
        if self.attr.repeated && self.attr.packed {
            return quote! {
                protowirers::wire::WireStruct::new(
                    #fieild_num,
                    protowirers::wire::WireData::LengthDelimited(protowirers::parser::Parser::from(
                        self.#filed_indent.clone(),
                        protowirers::wire::TypeLengthDelimited::PackedRepeatedFields(protowirers::wire::AllowedPakcedType::Variant(
                            #wdt,
                        )),
                    )?),
                )
            };
        }
        // TODO 暫定として一律cloneするが、要検討。
        quote! {
            protowirers::wire::WireStruct::new(
                #fieild_num,
                #wt(protowirers::parser::Parser::from(self.#filed_indent.clone(), #wdt)?),
            )
        }
    }
}
pub struct Attribute<'a> {
    pub original: &'a syn::Attribute,
    pub filed_num: u64,
    pub def_type: DefType,
    pub repeated: bool,
    pub packed: bool,
}

impl<'a> Attribute<'a> {
    fn from_syn(attrs: &'a [syn::Attribute], with_field: &syn::Field) -> syn::Result<Self> {
        let mut a: Vec<(&'a syn::Attribute, syn::MetaList)> = attrs
            .iter()
            .filter_map(|attr| match attr.meta {
                syn::Meta::List(ref ml) if ml.path.is_ident("def") => Some((attr, ml.clone())),
                _ => None,
            })
            .collect();
        if a.is_empty() {
            return Err(syn::Error::new_spanned(
                &with_field.ident,
                "#[def(...)] attribute is required",
            ));
        } else if a.len() > 1 {
            return Err(syn::Error::new_spanned(
                with_field,
                "only one #[def(...)] attribute is allowed",
            ));
        }

        let (original, meta_list): (&'a syn::Attribute, syn::MetaList) = a.remove(0);

        let mut filed_num: Option<u64> = None;
        let mut def_type: Option<DefType> = None;
        let mut repeated: Option<()> = None;
        let mut packed: Option<()> = None;

        meta_list.parse_nested_meta(|nested_meta| {
            if nested_meta.path.is_ident("field_num") {
                if filed_num.is_some() {
                    return Err(syn::Error::new_spanned(
                        nested_meta.path,
                        "field_num is duplicated in #[def(...)]. ",
                    ));
                }
                let value = nested_meta.value()?;
                let v: syn::LitInt = value.parse()?;
                println!("LitIntz");

                let v = v
                    .base10_parse::<u64>()
                    .map_err(|e| syn::Error::new(v.span(), format!("faild to parse u64: {}", e)))?;
                filed_num = Some(v);
            }
            if nested_meta.path.is_ident("def_type") {
                println!("def_typexxxx");
                if def_type.is_some() {
                    return Err(syn::Error::new_spanned(
                        nested_meta.path,
                        "def_type is duplicated in #[def(...)].",
                    ));
                }
                let value = nested_meta.value()?;
                let v: syn::LitStr = value.parse()?;
                match DefType::new(v.value()) {
                    Some(dt) => def_type = Some(dt),
                    None => {
                        return Err(syn::Error::new_spanned(
                            nested_meta.path,
                            format!("no suport def_type. got=`{}`.", v.value()),
                        ))
                    }
                }
            }
            if nested_meta.path.is_ident("repeated") {
                if repeated.is_some() {
                    return Err(syn::Error::new_spanned(
                        nested_meta.path,
                        "repeated is duplicated in #[def(...)]. ",
                    ));
                }
                repeated = Some(());
            }
            if nested_meta.path.is_ident("packed") {
                if packed.is_some() {
                    return Err(syn::Error::new_spanned(
                        nested_meta.path,
                        "packed is duplicated in #[def(...)]. ",
                    ));
                }
                packed = Some(());
            }
            Ok(())
        })?;

        if filed_num.is_none() {
            // required
            return Err(syn::Error::new_spanned(
                original,
                "filed_num is required in #[def(...)]",
            ));
        }
        if def_type.is_none() {
            // required
            return Err(syn::Error::new_spanned(
                original,
                "def_type is required in #[def(\"...\")]",
            ));
        }

        Ok(Self {
            original,
            filed_num: filed_num.unwrap(),
            def_type: def_type.unwrap(),
            repeated: repeated.is_some(),
            packed: packed.is_some(),
        })
    }

    fn allows_rust_type(&self, ty: &syn::Type) -> bool {
        match *ty {
            syn::Type::Path(ref p) => Some(&p.path),
            _ => None,
        }
        .and_then(|p| {
            // ident が取れるならそれをもとに型を。そうでない場合、Vecに指定されている型を採用
            p.get_ident().or_else(|| {
                if !(self.def_type.is_allows_vec() || self.packed && self.repeated) {
                    return None;
                }
                p.segments
                    .iter()
                    // Vec限定
                    .find(|x| x.ident == "Vec")
                    .and_then(|x| match &x.arguments {
                        syn::PathArguments::AngleBracketed(ab) => Some(ab),
                        _ => None,
                    })
                    .and_then(|abga| {
                        abga.args.iter().find_map(|ga| match ga {
                            syn::GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })
                    })
                    .and_then(|t| match t {
                        syn::Type::Path(tp) => tp.path.get_ident(),
                        _ => None,
                    })
            })
        })
        .map(|i| self.def_type.allows_rust_type(&i.to_string()))
        .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DefType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum,
    Fixed64,
    Sfixed64,
    Double,
    String,
    EmbeddedMessages,
    Bytes,
    Fixed32,
    Sfixed32,
    Float,
}

impl DefType {
    fn new(s: String) -> Option<Self> {
        match s.as_ref() {
            "int32" => Some(DefType::Int32),
            "int64" => Some(DefType::Int64),
            "uint32" => Some(DefType::Uint32),
            "uint64" => Some(DefType::Uint64),
            "sint32" => Some(DefType::Sint32),
            "sint64" => Some(DefType::Sint64),
            "bool" => Some(DefType::Bool),
            "enum" => Some(DefType::Enum),
            "fixed64" => Some(DefType::Fixed64),
            "sfixed64" => Some(DefType::Sfixed64),
            "double" => Some(DefType::Double),
            "fixed32" => Some(DefType::Fixed32),
            "string" => Some(DefType::String),
            "bytes" => Some(DefType::Bytes),
            "embedded" => Some(DefType::EmbeddedMessages),
            "sfixed32" => Some(DefType::Sfixed32),
            "float" => Some(DefType::Float),
            _ => None,
        }
    }
    fn is_allows_vec(&self) -> bool {
        matches!(self, DefType::Bytes)
    }
    fn allows_rust_type(&self, rust_type: &str) -> bool {
        let ty = match &self {
            DefType::Int32 => "i32",
            DefType::Int64 => "i64",
            DefType::Uint32 => "u32",
            DefType::Uint64 => "u64",
            DefType::Sint32 => "i32",
            DefType::Sint64 => "i64",
            DefType::Bool => "bool",
            DefType::Enum => {
                return true;
            }
            DefType::String => "String",
            DefType::Bytes => "u8",
            DefType::EmbeddedMessages => {
                return true;
            }
            DefType::Fixed64 => "u64",
            DefType::Sfixed64 => "i64",
            DefType::Double => "f64",
            DefType::Fixed32 => "u32",
            DefType::Sfixed32 => "i32",
            DefType::Float => "f32",
        };
        rust_type == ty
    }

    fn to_input_wire_data_type(self) -> proc_macro2::TokenStream {
        match &self {
            DefType::Int32 => quote! {protowirers::wire::TypeVairant::Int32},
            DefType::Int64 => quote! {protowirers::wire::TypeVairant::Int64},
            DefType::Uint32 => quote! {protowirers::wire::TypeVairant::Uint32},
            DefType::Uint64 => quote! {protowirers::wire::TypeVairant::Uint64},
            DefType::Sint32 => quote! {protowirers::wire::TypeVairant::Sint32},
            DefType::Sint64 => quote! {protowirers::wire::TypeVairant::Sint64},
            DefType::Bool => quote! {protowirers::wire::TypeVairant::Bool},
            DefType::Enum => quote! {protowirers::wire::TypeVairant::Enum},
            DefType::String => quote! {protowirers::wire::TypeLengthDelimited::WireString},
            DefType::Bytes => quote! {protowirers::wire::TypeLengthDelimited::Bytes},
            DefType::EmbeddedMessages => {
                quote! {protowirers::wire::TypeLengthDelimited::EmbeddedMessages}
            }
            DefType::Fixed64 => quote! {protowirers::wire::TypeBit64::Fixed64},
            DefType::Sfixed64 => quote! {protowirers::wire::TypeBit64::Sfixed64},
            DefType::Double => quote! {protowirers::wire::TypeBit64::Double},
            DefType::Fixed32 => quote! {protowirers::wire::TypeBit32::Fixed32},
            DefType::Sfixed32 => quote! {protowirers::wire::TypeBit32::Sfixed32},
            DefType::Float => quote! {protowirers::wire::TypeBit32::Float},
        }
    }

    fn to_corresponding_wire_type(self) -> proc_macro2::TokenStream {
        match &self {
            DefType::Int32
            | DefType::Int64
            | DefType::Uint32
            | DefType::Uint64
            | DefType::Sint32
            | DefType::Sint64
            | DefType::Bool
            | DefType::Enum => {
                quote! {protowirers::wire::WireData::Varint}
            }
            DefType::String | DefType::Bytes | DefType::EmbeddedMessages => {
                quote! {protowirers::wire::WireData::LengthDelimited}
            }
            DefType::Fixed64 | DefType::Sfixed64 | DefType::Double => {
                quote! {protowirers::wire::WireData::Bit64}
            }
            DefType::Fixed32 | DefType::Sfixed32 | DefType::Float => {
                quote! {protowirers::wire::WireData::Bit32}
            }
        }
    }
}
