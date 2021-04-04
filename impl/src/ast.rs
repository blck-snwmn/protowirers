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
}
pub struct Attribute<'a> {
    pub original: &'a syn::Attribute,
    pub filed_num: u64,
    pub def_type: String,
}

impl<'a> Attribute<'a> {
    fn from_syn(attrs: &'a [syn::Attribute], with_field: &'a syn::Field) -> syn::Result<Self> {
        let mut original: Option<&'a syn::Attribute> = None;
        let mut filed_num: Option<u64> = None;
        let mut def_type: Option<String> = None;

        for attr in attrs {
            let meta_list = attr.parse_meta().ok().and_then(|m| match m {
                syn::Meta::List(ml) if ml.path.is_ident("def") => Some(ml),
                _ => None,
            });
            if meta_list.is_none() {
                continue;
            }
            if original.is_some() {
                // TODO return error
            }
            original = Some(attr);

            let meta_list = meta_list.unwrap();
            for nested_meta in &meta_list.nested {
                let named_value = match nested_meta {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value)) => {
                        Some(meta_name_value)
                    }
                    _ => None,
                };
                // if named_value is None -> return error
                let named_value = named_value.unwrap();

                if named_value.path.is_ident("field_num") {
                    if filed_num.is_some() {
                        // TODO duplicate error
                    }
                    match named_value.lit {
                        syn::Lit::Int(ref v) => {
                            // TODO error を返すようにすると思うので、u64の値を返すようにする
                            let v = v.base10_parse::<u64>();
                            // TODO v is error => return invalid value error
                            let v = v.unwrap();
                            filed_num = Some(v);
                        }
                        _ => {
                            // TODO return invalid type error
                        }
                    }
                } else if named_value.path.is_ident("def_type") {
                    if def_type.is_some() {
                        // TODO duplicate error
                    }
                    match named_value.lit {
                        syn::Lit::Str(ref v) => {
                            let vv = v.value();
                            match vv.as_ref() {
                                "int32" | "sint64" => {
                                    // TODO error を返すようにすると思うので、なにか値を返す
                                    def_type = Some(vv)
                                }
                                _ => {
                                    // unsuported type
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    // unsuported attribute metadata
                }
            }
        }

        if original.is_none() {
            // required
            return Err(syn::Error::new_spanned(
                with_field,
                "suport data is only Sturct",
            ));
        }
        let original = original.unwrap();
        if filed_num.is_none() {
            // required
            return Err(syn::Error::new_spanned(
                original,
                "suport data is only Sturct",
            ));
        }
        if def_type.is_none() {
            // required
            return Err(syn::Error::new_spanned(
                original,
                "suport data is only Sturct",
            ));
        }

        Ok(Self {
            original: original,
            filed_num: filed_num.unwrap(),
            def_type: def_type.unwrap(),
        })
    }
}
