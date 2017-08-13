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
use std::collections::HashSet;

mod keywords;

use errors::*;

#[allow(unused_doc_comment)]
mod errors {
    error_chain!{
       foreign_links {
           Io(::std::io::Error);
        }
    }
}

lazy_static! {
    static ref RUST_KEYWORDS: HashSet<&'static str> = {
        keywords::RUST_KEYWORDS.iter().map(|v| *v).collect()
    };
}

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
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

// #[derive(Clone, Default)]
// pub struct Alias {
//     name: String,
//     typ: String
// }

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub derive: Vec<Derive>,
    pub cfg: Vec<Cfg>,
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
    pub attrs: Vec<FieldAttr>, // TODO separate field attrs?
}

impl Field {
    pub fn new(name: String, typ: String, attrs: Vec<FieldAttr>) -> Result<Field> {
        validate_identifier(&name)?;
        validate_identifier(&typ)?;
        Ok(Field { name, typ, attrs })
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

fn validate_identifier(ident: &str) -> Result<()> {
    // This assumes ASCII character set
    if ident == "_" {
        bail!("'_' is not a valid item name")
    }
    if ident.len() == 0 {
        bail!("Identifier is empty string")
    }
    if RUST_KEYWORDS.contains(ident) {
        bail!("Identifier '{}' is a Rust keyword", ident)
    }
    let mut is_leading_char = true;
    for (ix, c) in ident.chars().enumerate() {
        if is_leading_char {
            match c {
                'A'...'Z' | 'a'...'z' | '_' => {
                    is_leading_char = false;
                }
                _ => bail!("Identifier has invalid character at index {}: '{}'", ix, c),
            }
        } else {
            match c {
                'A'...'Z' | 'a'...'z' | '_' | '0'...'9' => {}
                _ => bail!("Identifier has invalid character at index {}: '{}'", ix, c),
            }
        }
    }
    Ok(())
}

pub fn rust_format(code: &str) -> Result<String> {
    use rustfmt::{Input, format_input};
    use std::fs::File;
    use tempdir::TempDir;
    use std::io::prelude::*;

    let tmpdir = TempDir::new("codegen-rustfmt")?;
    let tmppath = tmpdir.path().join("to_format.rs");

    // FIXME workaround is necessary until rustfmt works programmatically
    {
        let mut tmp = File::create(&tmppath)?;
        tmp.write_all(code.as_bytes())?;
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
    if buf == code {
        bail!("Syntax error detected")
    }
    Ok(buf)
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
            Attributes {
                derive: vec![Clone, Debug],
                cfg: vec![Test, TargetOs("linux".into())],
                ..Default::default()
            },
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
        let expect = r#"#[derive(Clone, Debug)]
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
            Attributes {
                derive: vec![Clone, Eq, Derive::Custom("MyDerive".into())],
                custom: vec!["my_custom_attribute".into()],
                ..Default::default()
            },
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
    fn test_validate_ident() {
        assert!(validate_identifier("thisIsValid").is_ok());
        assert!(validate_identifier("_this_also_valid").is_ok());
        assert!(validate_identifier("_type").is_ok());
        assert!(validate_identifier("T343434234").is_ok());

        assert!(validate_identifier("_").is_err());
        assert!(validate_identifier("@").is_err());
        assert!(validate_identifier("contains space").is_err());
        assert!(validate_identifier("contains££££symbol").is_err());
        assert!(validate_identifier("type").is_err());
        assert!(validate_identifier("3_invalid").is_err());
    }
}
