use quote::quote;
pub enum Input<'a> {
    Struct(Struct<'a>),
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a syn::DeriveInput) -> syn::Result<Self> {
        match &node.data {
            syn::Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            _ => Err(syn::Error::new_spanned(node, "suport data is only Sturct")),
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
        let attr = Attribute::from_syn(&f.attrs, f)?;
        Ok(Self {
            original: f,
            attr: attr,
        })
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
        let def_type = a.def_type.to_parse_function();
        quote! {
            (#fieild_num, wire::WireType::Varint(v)) => {
                #filed_indent = Some(#def_type(v)?);
            }
        }
    }

    fn build_gen_wirestructs(&self) -> proc_macro2::TokenStream {
        let filed_indent = &self.original.ident;
        let a = &self.attr;
        let fieild_num = a.filed_num as u128;
        let gen_fn = a.def_type.to_gen_function();
        quote! {
            #gen_fn(#fieild_num, self.#filed_indent)
        }
    }
}
pub struct Attribute<'a> {
    pub original: &'a syn::Attribute,
    pub filed_num: u64,
    pub def_type: DefType,
}

impl<'a> Attribute<'a> {
    fn from_syn(attrs: &'a [syn::Attribute], with_field: &syn::Field) -> syn::Result<Self> {
        let mut a: Vec<(&'a syn::Attribute, syn::MetaList)> = attrs
            .iter()
            .filter_map(|attr| {
                let ml = attr.parse_meta().ok().and_then(|m| match m {
                    syn::Meta::List(ml) if ml.path.is_ident("def") => Some(ml),
                    _ => None,
                })?;
                return Some((attr, ml));
            })
            .collect();
        if a.len() == 0 {
            return Err(syn::Error::new_spanned(
                &with_field.ident,
                "#[def(...)] attribute is required",
            ));
        } else if a.len() > 1 {
            return Err(syn::Error::new_spanned(
                &with_field,
                "only one #[def(...)] attribute is allowed",
            ));
        }

        let (original, meta_list): (&'a syn::Attribute, syn::MetaList) = a.remove(0);

        let mut filed_num: Option<u64> = None;
        let mut def_type: Option<DefType> = None;
        for nested_meta in &meta_list.nested {
            let named_value = match nested_meta {
                syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value)) => Ok(meta_name_value),
                _ => Err(syn::Error::new_spanned(
                    nested_meta,
                    "only `foo = bar` format is allowed. ",
                )),
            }?;

            if named_value.path.is_ident("field_num") {
                if filed_num.is_some() {
                    return Err(syn::Error::new_spanned(
                        &named_value,
                        "field_num is duplicated in #[def(...)]. ",
                    ));
                }
                let v = match named_value.lit {
                    syn::Lit::Int(ref v) => Ok(v),
                    _ => Err(syn::Error::new_spanned(
                        &named_value.lit,
                        "invalid value. value is integer only.",
                    )),
                }?;
                let v = v
                    .base10_parse::<u64>()
                    .map_err(|e| syn::Error::new(v.span(), format!("faild to parse u64: {}", e)))?;
                filed_num = Some(v);
            } else if named_value.path.is_ident("def_type") {
                if def_type.is_some() {
                    return Err(syn::Error::new_spanned(
                        &named_value,
                        "def_type is duplicated in #[def(...)].",
                    ));
                }
                let v = match named_value.lit {
                    syn::Lit::Str(ref v) => DefType::new(v.value()).ok_or(syn::Error::new_spanned(
                        &named_value.lit,
                        format!("no suport def_type. got=`{}`.", v.value()),
                    )),
                    _ => Err(syn::Error::new_spanned(
                        &named_value.lit,
                        "invalid num of sub field in #[def(...)]. ",
                    )),
                }?;
                def_type = Some(v);
            } else {
                // unsuported attribute metadata
                return Err(syn::Error::new_spanned(
                    named_value,
                    "unsuported meta data in #[def(...)]. ",
                ));
            }
        }

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
            original: original,
            filed_num: filed_num.unwrap(),
            def_type: def_type.unwrap(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DefType {
    Int32,
    Sint64,
}

impl DefType {
    fn new(s: String) -> Option<Self> {
        match s.as_ref() {
            "int32" => Some(DefType::Int32),
            "sint64" => Some(DefType::Sint64),
            _ => None,
        }
    }
    fn to_parse_function(&self) -> proc_macro2::TokenStream {
        match &self {
            DefType::Int32 => quote! {parser::parse_u32},
            DefType::Sint64 => quote! {parser::parse_i64},
        }
    }
    fn to_gen_function(&self) -> proc_macro2::TokenStream {
        match &self {
            DefType::Int32 => quote! {wire::WireStruct::from_u32},
            DefType::Sint64 => quote! {wire::WireStruct::from_i64},
        }
    }
}
