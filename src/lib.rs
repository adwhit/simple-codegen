#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate rustfmt;
extern crate tempdir;
#[macro_use]
extern crate lazy_static;

use quote::{Tokens, ToTokens, Ident};

use std::fmt;
use std::collections::BTreeSet;

mod keywords;
pub mod utils;
pub mod typebuilder;

use errors::*;
use utils::*;

#[allow(unused_doc_comment)]
pub mod errors {
    error_chain!{
       foreign_links {
           Io(::std::io::Error);
        }
    }
}

lazy_static! {
    static ref RUST_KEYWORDS: BTreeSet<&'static str> = {
        keywords::RUST_KEYWORDS.iter().map(|v| *v).collect()
    };
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Struct {
    name: String,
    vis: Visibility,
    attrs: Attributes,
    fields: Vec<Field>,
}

impl Struct {
    pub fn new(
        name: String,
        vis: Visibility,
        attrs: Attributes,
        fields: Vec<Field>,
    ) -> Result<Struct> {
        validate_identifier(&name)?;
        Ok(Struct {
            name,
            vis,
            attrs,
            fields,
        })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let vis = self.vis;
        let attrs = &self.attrs;
        let fields = &self.fields;
        let toks =
            quote! {
            #attrs
            #vis struct #name {
                #(#fields),*
            }
        };
        tokens.append(toks)
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tokens = quote!{#self};
        write!(f, "{}", tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    name: String,
    vis: Visibility,
    attrs: Attributes,
    variants: Vec<Variant>,
}

impl Enum {
    pub fn new(
        name: String,
        vis: Visibility,
        attrs: Attributes,
        variants: Vec<Variant>,
    ) -> Result<Enum> {
        validate_identifier(&name)?;
        Ok(Enum {
            name,
            vis,
            attrs,
            variants,
        })
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let vis = self.vis;
        let attrs = &self.attrs;
        let variants = &self.variants;
        let toks =
            quote! {
                #attrs
                #vis enum #name {
                    #(#variants),*
                }
            };
        tokens.append(toks)
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tokens = quote!{#self};
        write!(f, "{}", tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewType {
    name: String,
    vis: Visibility,
    attrs: Attributes,
    typ: String,
}

impl NewType {
    pub fn new(name: String, vis: Visibility, attrs: Attributes, typ: String) -> Result<NewType> {
        validate_identifier(&name)?;
        validate_identifier(&typ)?;
        Ok(NewType {
            name,
            vis,
            attrs,
            typ,
        })
    }
}

impl ToTokens for NewType {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let typ = Ident::from(&*self.typ);
        let attrs = &self.attrs;
        let vis = self.vis;
        let toks =
            quote! {
            #attrs #vis struct #name(#typ);
        };
        tokens.append(toks);
    }
}

impl fmt::Display for NewType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tokens = quote!{#self};
        write!(f, "{}", tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    name: String,
    vis: Visibility,
    typ: String,
}

impl Alias {
    pub fn new(name: String, vis: Visibility, typ: String) -> Result<Alias> {
        validate_identifier(&name)?;
        validate_identifier(&typ)?;
        Ok(Alias { name, vis, typ })
    }
}

impl ToTokens for Alias {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let vis = self.vis;
        let typ = Ident::from(&*self.typ);
        let toks =
            quote! {
                #vis type #name = #typ;
            };
        tokens.append(toks);
    }
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tokens = quote!{#self};
        write!(f, "{}", tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    derive: BTreeSet<Derive>,
    cfg: BTreeSet<Cfg>,
    custom: BTreeSet<String>,
}

impl Attributes {
    pub fn derive(mut self, derives: &[Derive]) -> Self {
        for d in derives {
            self.derive.insert(d.clone());
        }
        self
    }

    pub fn cfg(mut self, cfgs: &[Cfg]) -> Self {
        for c in cfgs {
            self.cfg.insert(c.clone());
        }
        self
    }

    pub fn custom(mut self, customs: &[String]) -> Self {
        for c in customs {
            self.custom.insert(c.to_string());
        }
        self
    }
}

impl ToTokens for Attributes {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if self.derive.len() > 0 {
            let derives = self.derive.iter().map(|d| Ident::from(d.to_string()));
            tokens.append(quote! {
                #[derive(#(#derives),*)]
            });
        }
        if self.cfg.len() > 0 {
            let configs = self.cfg.iter().map(|d| Ident::from(d.to_string()));
            tokens.append(quote! {
                #[cfg(#(#configs),*)]
            });
        }
        if self.custom.len() > 0 {
            let customs = self.custom.iter().map(|d| Ident::from(d.to_string()));
            tokens.append(quote! {
                #[#(#customs),*]
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub typ: String,
    pub attrs: Vec<FieldAttr>, // TODO separate field attrs?
}

impl Field {
    pub fn new(name: String, typ: String, attrs: Vec<FieldAttr>) -> Result<Field> {
        validate_identifier(&name)?;
        validate_identifier(&typ)?;
        Ok(Field { name, typ, attrs })
    }

    // TODO do we want this kind of higher-level functionality in this crate?
    // pub fn new_with_serde_rename(name: String, typ: String, attrs: Vec<FieldAttr>) -> Result<Field> {
    //     let name = match make_valid_identifier(name)? {
    //         Cow::Borrowed(n) => n.to_string(),
    //         Cow::Owned(n) => {
    //             attrs.push(FieldAttr::SerdeRename(name));
    //             n
    //         }
    //     };
    // }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let attrs = &self.attrs;
        let typ = Ident::from(&*self.typ);
        let tok =
            quote! {
                #(#attrs)*
                #name: #typ
            };
        tokens.append(tok);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    name: String,
    typ: Option<String>,
    attrs: Vec<FieldAttr>, // TODO separate field attrs?
}

impl Variant {
    pub fn new(name: String, typ: Option<String>, attrs: Vec<FieldAttr>) -> Result<Variant> {
        validate_identifier(&name)?;
        if let Some(ref typ) = typ {
            validate_identifier(typ)?;
        }
        Ok(Variant { name, typ, attrs })
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let attrs = &self.attrs;
        let tok = match self.typ {
            Some(ref t) => {
                let typ = Ident::from(t.as_str());
                quote! {
                    #(#attrs)* #name(#typ)
                }
            }
            None => {
                quote! {
                    #(#attrs)* #name
                }
            }
        };
        tokens.append(tok);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
    Crate,
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            Visibility::Private => {}
            Visibility::Public => tokens.append("pub"),
            Visibility::Crate => tokens.append("pub(crate) "),
        }
    }
}

impl Default for Visibility {
    fn default() -> Visibility {
        Visibility::Private
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldAttr {
    SerdeDefault,
    SerdeRename(String),
    Custom(String),
}

impl ToTokens for FieldAttr {
    fn to_tokens(&self, tokens: &mut Tokens) {
        use FieldAttr::*;
        match *self {
            SerdeDefault => tokens.append(format!("#[serde(default)]")),
            SerdeRename(ref name) => tokens.append(format!("#[serde(rename = \"{}\")]", name)),
            Custom(ref name) => tokens.append(name),
        }
    }
}

// #[derive(Clone, Debug)]
// enum VariantAttr {
//     SerdeRename(String),
//     SerdeSkipSerialize,
//     SerdeSkipDeserialize,
//     Custom(String),
// }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Derive {
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Custom(String),
}

impl fmt::Display for Derive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Derive::Custom(ref custom) => write!(f, "{}", custom),
            ref other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cfg {
    Test,
    TargetOs(String),
    Custom(String),
}

impl fmt::Display for Cfg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Cfg::*;
        match *self {
            Test => write!(f, "test"),
            TargetOs(ref target) => write!(f, "target_os = \"{}\"", target),
            Custom(ref custom) => write!(f, "{}", custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Derive::*;
    use Cfg::*;
    use FieldAttr::*;

    #[test]
    fn test_struct() {
        let my_struct = Struct::new(
            "MyStruct".into(),
            Visibility::Public,
            Attributes::default().derive(&[Clone, Debug]).cfg(
                &[
                    Test,
                    TargetOs(
                        "linux".into(),
                    ),
                ],
            ),
            vec![
                Field::new("field1".into(), "Type1".into(), Default::default())
                    .unwrap(),
                Field::new(
                    "field2".into(),
                    "Type2".into(),
                    vec![SerdeRename("Field-2".into()), SerdeDefault]
                ).unwrap(),
            ],
        ).unwrap();

        let pretty = rust_format(&my_struct.to_string()).unwrap();
        let expect = r#"#[derive(Debug, Clone)]
#[cfg(test, target_os = "linux")]
pub struct MyStruct {
    field1: Type1,
    #[serde(rename = "Field-2")]
    #[serde(default)]
    field2: Type2,
}
"#;
        assert_eq!(pretty, expect);
    }

    #[test]
    fn test_enum() {
        let e = Enum::new(
            "MyEnum".into(),
            Visibility::Crate,
            Attributes::default()
                .derive(&[Clone, Eq, Derive::Custom("MyDerive".into())])
                .custom(&["my_custom_attribute".into()]),
            vec![
                Variant::new(
                    "Variant1".into(),
                    Default::default(),
                    vec![FieldAttr::SerdeRename("used-to-be-this".into())]
                ).unwrap(),
                Variant::new("Variant2".into(), Some("VType".into()), Default::default())
                    .unwrap(),
            ],
        ).unwrap();
        let pretty = rust_format(&e.to_string()).unwrap();
        let expect = r#"#[derive(Clone, Eq, MyDerive)]
#[my_custom_attribute]
pub(crate) enum MyEnum {
    #[serde(rename = "used-to-be-this")]
    Variant1,
    Variant2(VType),
}
"#;
        assert_eq!(pretty, expect);
    }

    #[test]
    fn test_newtype() {
        let n = NewType::new(
            "MyNewType".into(),
            Visibility::Private,
            Default::default(),
            "MyOldType".into(),
        ).unwrap();
        let pretty = rust_format(&n.to_string()).unwrap();
        let expect = "struct MyNewType(MyOldType);\n";
        assert_eq!(pretty, expect);
    }

    #[test]
    fn test_alias() {
        let a = Alias::new("MyAlias".into(), Visibility::Crate, "MyAliasedType".into()).unwrap();
        let pretty = rust_format(&a.to_string()).unwrap();
        let expect = "pub(crate) type MyAlias = MyAliasedType;\n";
        assert_eq!(pretty, expect);
    }

}
