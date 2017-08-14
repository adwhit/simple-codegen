#[macro_use]
extern crate error_chain;
extern crate rustfmt;
extern crate tempdir;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_new;

use std::fmt;
use std::collections::BTreeSet;

mod keywords;
pub mod utils;
pub mod typebuilder;

use errors::*;
use typebuilder::Type;

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

/// Wrapper around String which guarantees that
/// the value can be used as a Rust identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id(String);

impl Id {
    pub fn new<I: Into<String>>(ident: I) -> Result<Id> {
        let ident: String = ident.into();
        utils::validate_identifier(&ident)?;
        Ok(Id(ident))
    }
}

impl std::ops::Deref for Id {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Struct {
    name: Id,
    vis: Visibility,
    attrs: Attributes,
    fields: Vec<Field>,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fields = render_delimited(&self.fields, ", ");
        write!(f, "{} {} struct {} {{ {} }}", self.attrs, self.vis, self.name, fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Enum {
    name: Id,
    vis: Visibility,
    attrs: Attributes,
    variants: Vec<Variant>,
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variants = render_delimited(&self.variants, ", ");
        write!(f, "{} {} enum {} {{ {} }}", self.attrs, self.vis, self.name, variants)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct NewType {
    name: Id,
    vis: Visibility,
    attrs: Attributes,
    typ: Type,
}

impl fmt::Display for NewType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} struct {}({});", self.attrs, self.vis, self.name, self.typ)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Alias {
    name: Id,
    vis: Visibility,
    typ: Type,
}

impl fmt::Display for Alias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} type {} = {};",self.vis, self.name, self.typ)
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

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.derive.len() > 0 {
            let derives = render_delimited(&self.derive.iter().collect::<Vec<_>>(), ", ");
            write!(f, "#[derive({})]", derives)?;
        }
        if self.cfg.len() > 0 {
            let cfgs = render_delimited(&self.cfg.iter().collect::<Vec<_>>(), ", ");
            write!(f, "#[cfg({})]", cfgs)?;
        }
        if self.custom.len() > 0 {
            let customs = render_delimited(&self.custom.iter().collect::<Vec<_>>(), ", ");
            write!(f, "#[{}]", customs)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Field {
    pub name: Id,
    pub typ: Type,
    pub attrs: Vec<FieldAttr>, // TODO separate field attrs?
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let attrs = render_delimited(&self.attrs, " ");
        write!(f, "{} {}:{}", attrs, self.name, self.typ)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Variant {
    name: Id,
    typ: Option<Type>,
    attrs: Vec<FieldAttr>, // TODO separate field attrs?
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let attrs = render_delimited(&self.attrs, ", ");
        match self.typ {
            Some(ref t) => {
                write!(f, "{} {}({})", attrs, self.name, t)
            }
            None => {
                write!(f, "{} {}", attrs, self.name)
            }
        }
    }
}

fn render_delimited<T: fmt::Display>(items: &[T], delimiter: &str) -> String {
    items.iter().map(|item| format!("{}", item)).collect::<Vec<String>>().join(delimiter).to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
    Crate,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Visibility::Private => Ok(()),
            Visibility::Public => write!(f, "pub "),
            Visibility::Crate => write!(f, "pub(crate) "),
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

impl fmt::Display for FieldAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FieldAttr::*;
        match *self {
            SerdeDefault => write!(f, "#[serde(default)]"),
            SerdeRename(ref name) => write!(f, "#[serde(rename = \"{}\")]", name),
            Custom(ref name) => write!(f, "{}", name)
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
    use utils::rust_format;

    #[test]
    fn test_struct() {
        let my_struct = Struct::new(
            Id::new("MyStruct").unwrap(),
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
                Field::new(
                    Id::new("field1").unwrap(),
                    Type::named("Type1").unwrap(),
                    Default::default()
                ),
                Field::new(
                    Id::new("field2").unwrap(),
                    Type::named("Type2").unwrap(),
                    vec![SerdeRename("Field-2".into()), SerdeDefault]
                )
            ],
        );

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
            Id::new("MyEnum").unwrap(),
            Visibility::Crate,
            Attributes::default()
                .derive(&[Clone, Eq, Derive::Custom("MyDerive".into())])
                .custom(&["my_custom_attribute".into()]),
            vec![
                Variant::new(
                    Id::new("Variant1").unwrap(),
                    Default::default(),
                    vec![FieldAttr::SerdeRename("used-to-be-this".into())]
                ),
                Variant::new(
                    Id::new("Variant2").unwrap(),
                    Some(Type::named("VType").unwrap()),
                    Default::default()
                )
            ],
        );
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
            Id::new("MyNewType").unwrap(),
            Visibility::Private,
            Default::default(),
            Type::named("MyOldType").unwrap(),
        );
        let pretty = rust_format(&n.to_string()).unwrap();
        let expect = "struct MyNewType(MyOldType);\n";
        assert_eq!(pretty, expect);
    }

    #[test]
    fn test_alias() {
        let a = Alias::new(
            Id::new("MyAlias").unwrap(),
            Visibility::Crate,
            Type::named("MyAliasedType").unwrap());
        let pretty = rust_format(&a.to_string()).unwrap();
        let expect = "pub(crate) type MyAlias = MyAliasedType;\n";
        assert_eq!(pretty, expect);
    }

}
