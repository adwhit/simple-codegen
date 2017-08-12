#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate rustfmt;
extern crate tempdir;

use quote::{Tokens, ToTokens, Ident};

use std::fmt;

use errors::*;

#[allow(unused_doc_comment)]
mod errors {
    error_chain!{
       foreign_links {
           Io(::std::io::Error);
        }
    }
}


#[derive(Clone, Default)]
pub struct Struct {
    pub name: String,
    pub vis: Visibility,
    pub attrs: Attributes,
    pub fields: Vec<Field>
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

#[derive(Clone, Default)]
pub struct Enum {
    pub name: String,
    pub vis: Visibility,
    pub attrs: Attributes,
    pub variants: Vec<Variant>
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

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub derive: Vec<DeriveAttr>,
    pub cfg: Vec<CfgAttr>,
    pub custom: Vec<String>,
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

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub typ: String,
    pub attrs: Vec<FieldAttr>  // TODO separate field attrs?
}

impl Field {
    pub fn new(name: String, typ: String, attrs: Vec<FieldAttr>) -> Field {
        Field {name, typ, attrs}
    }
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

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub typ: Option<String>,
    pub attrs: Vec<FieldAttr>  // TODO separate field attrs?
}

impl Variant {
    pub fn new(name: String, typ: Option<String>, attrs: Vec<FieldAttr>) -> Variant {
        Variant{ name, typ, attrs}
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
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
            Custom(ref name) => tokens.append(name)
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

#[derive(Clone, Debug)]
pub enum DeriveAttr {
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Custom(String)
}

impl fmt::Display for DeriveAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeriveAttr::Custom(ref custom) => write!(f, "{}", custom),
            ref other => write!(f, "{:?}", other)
        }
    }
}

#[derive(Clone, Debug)]
pub enum CfgAttr {
    Test,
    TargetOs(String),
    Custom(String)
}

impl fmt::Display for CfgAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CfgAttr::*;
        match *self {
            Test => write!(f, "test"),
            TargetOs(ref target) => write!(f, "target_os = \"{}\"", target),
            Custom(ref custom) => write!(f, "{}", custom),
        }
    }
}

pub fn rust_format(t: &Tokens) -> Result<String> {
    use rustfmt::{Input, format_input};
    use std::fs::File;
    use tempdir::TempDir;
    use std::io::prelude::*;

    let tmpdir = TempDir::new("codegen-rustfmt")?;
    let tmppath = tmpdir.path().join("to_format.rs");

    // FIXME workaround is necessary until rustfmt works programmatically
    {
        let mut tmp = File::create(&tmppath)?;
        tmp.write_all(t.as_str().as_bytes())?;
    }

    let input = Input::File((&tmppath).into());
    let mut fakebuf = Vec::new(); // pretty weird that this is necessary.. but it is

    match format_input(input, &Default::default(), Some(&mut fakebuf)) {
        Ok((_summmary, _filemap, _report)) => {}
        Err((e, _summary)) => Err(e)?,
    }

    let mut tmp = File::open(&tmppath)?;
    let mut buf = String::new();
    tmp.read_to_string(&mut buf)?;
    if buf == t.as_str() {
        bail!("Syntax error detected")
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct() {
        let s = Struct {
            name: "MyStruct".into(),
            vis: Visibility::Public,
            attrs: Attributes {
                derive: vec![DeriveAttr::Clone, DeriveAttr::Debug],
                cfg: vec![CfgAttr::Test, CfgAttr::TargetOs("linux".into())],
                ..Default::default()
            },
            fields: vec![
                Field::new("field1".into(), "Type1".into(), Default::default()),
                Field::new("field2".into(), "Type2".into(),
                           vec![FieldAttr::SerdeRename("Field-2".into()),
                                FieldAttr::SerdeDefault])]
        };

        let tokens = quote!{#s};
        let pretty = rust_format(&tokens).unwrap();
        let expect = r#"
#[derive(Clone, Debug)]
#[cfg(test, target_os = "linux")]
pub struct MyStruct {
    field1: Type1,
    #[serde(rename = "Field-2")]
    #[serde(default)]
    field2: Type2,
}"#;
        assert_eq!(pretty.trim(), expect.trim());
    }

    #[test]
    fn test_enum() {
        let e = Enum {
            name: "MyEnum".into(),
            vis: Visibility::Crate,
            attrs: Attributes {
                derive: vec![DeriveAttr::Clone,
                             DeriveAttr::Eq,
                             DeriveAttr::Custom("MyDerive".into())],
                custom: vec!["my_custom_attribute".into()],
                ..Default::default()
            },
            variants: vec![
                Variant::new("Variant1".into(), Default::default(), vec![
                    FieldAttr::SerdeRename("used-to-be-this".into())
                    ]),
                Variant::new("Variant2".into(), Some("VType".into()), Default::default())
                ]
        };
        let tokens = quote!{#e};
        println!("{}", tokens);
        let pretty = rust_format(&tokens).expect("Format failed");
        let expect = r#"
#[derive(Clone, Eq, MyDerive)]
#[my_custom_attribute]
pub(crate) enum MyEnum {
    #[serde(rename = "used-to-be-this")]
    Variant1,
    Variant2(VType),
}"#;
        assert_eq!(pretty.trim(), expect.trim());
    }
}
