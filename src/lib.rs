//! Simple code generation for Rust
//!
//! Helper utilities to help users generate valid Rust code.
//! The focus is on the 80% use case. It tries to make it the simple,
//! common cases easy to write, but should not stop the user
//! from building more complicated constructs.
//!
//! This crate is designed to be used for codegen with `serde`
//! and comes with helper functions to add `serde` attributes

#[macro_use]
extern crate error_chain;
extern crate rustfmt;
extern crate tempdir;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate derive_new;
extern crate inflector;

use std::fmt;
use std::collections::BTreeSet;

use inflector::Inflector;

mod keywords;
pub mod utils;
pub mod items;
mod typebuilder;

use errors::*;
pub use typebuilder::{Type, Primitive};
pub use items::{Item, ItemMap};

#[allow(unused_doc_comment)]
pub mod errors {
    error_chain!{
       foreign_links {
           Io(::std::io::Error);
        }
    }
}

/// Wrapper around String which guarantees that
/// the value can be used as a Rust identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    pub fn new<I: Into<String>>(ident: I) -> Result<Id> {
        let ident: String = ident.into();
        utils::validate_identifier(&ident)?;
        Ok(Id(ident))
    }

    /// Create a new Id, possibly mangled to make it into a valid identifier
    pub fn make_valid<I: Into<String>>(ident: I) -> Result<Id> {
        let ident = ident.into();
        if let std::borrow::Cow::Owned(id) = utils::make_valid_identifier(&ident)? {
            Ok(Id(id))
        } else {
            Ok(Id(ident))
        }
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

/// Represents a Rust struct
///
/// # Example
///
/// ```ignore
/// let my_struct = Struct::new(
///     Id::new("MyStruct").unwrap(),
///     Visibility::Public,
///     Attributes::default().derive(&[Clone, Debug]).cfg(
///         &[Test, TargetOs("linux".into())],
///     ),
///     vec![Field::new(
///             Id::new("field1").unwrap(),
///             Type::named("Type1").unwrap(),
///             Default::default()
///         ),
///         Field::new(
///             Id::new("field2").unwrap(),
///             Type::Box(Box::new(Type::named("Type2").unwrap())),
///             vec![SerdeRename("Field-2".into()), SerdeDefault]
///         ),
///         Field::with_rename("Snake Case Me", Type::named("Type3").unwrap())
///             .unwrap(),
///     ],
/// );
/// println!("{}", rust_format(my_struct.to_string()).unwrap());
/// // #[derive(Debug, Clone)]
/// // #[cfg(test, target_os = "linux")]
/// // pub struct MyStruct {
/// //     field1: Type1,
/// //     #[serde(rename = "Field-2")]
/// //     #[serde(default)]
/// //     field2: Box<Type2>,
/// //     #[serde(rename = "Snake Case Me")]
/// //     snake_case_me: Type3,
/// // }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Struct {
    name: Id,
    vis: Visibility,
    attrs: Attributes,
    fields: Vec<Field>,
}

impl Struct {
    pub fn merge(new_name: Id, vis: Visibility, attrs: Attributes, structs: &[Struct]) -> Result<Struct> {
        let mut fields = Vec::new();
        let mut field_chk = BTreeSet::new();
        for s in structs {
            for field in &s.fields {
                if !field_chk.insert(field.name.clone()) {
                    bail!("Duplicated field '{}'", field.name)
                }
                fields.push(field.clone());
            }
        }
        Ok(Struct::new(new_name, vis, attrs, fields))
    }

}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fields = render_delimited(&self.fields, ", ");
        write!(
            f,
            "{} {} struct {} {{ {} }}",
            self.attrs,
            self.vis,
            self.name,
            fields
        )
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
        write!(
            f,
            "{} {} enum {} {{ {} }}",
            self.attrs,
            self.vis,
            self.name,
            variants
        )
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
        write!(
            f,
            "{} {} struct {}({});",
            self.attrs,
            self.vis,
            self.name,
            self.typ
        )
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
        write!(f, "{} type {} = {};", self.vis, self.name, self.typ)
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

impl Field {
    /// Create a Field with the poss
    pub fn with_rename<I: Into<String>>(id: I, typ: Type) -> Result<Field> {
        let id: String = id.into();
        if id.is_snake_case() {
            let name = Id::make_valid(id)?;
            Ok(Field {
                name,
                typ,
                attrs: vec![],
            })
        } else {
            let name = Id::make_valid(id.to_snake_case())?;
            let attrs = vec![FieldAttr::SerdeRename(id)];
            Ok(Field { name, typ, attrs })
        }
    }

    pub(crate) fn get_named_type(&self) -> Option<&Id> {
        self.typ.get_named_root()
    }

    pub(crate) fn is_defaultable(&self, map: &ItemMap) -> bool {
        self.typ.is_defaultable(map)
    }

    pub(crate) fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool {
        self.typ.contains_unboxed_id(id, map)
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
            Some(ref t) => write!(f, "{} {}({})", attrs, self.name, t),
            None => write!(f, "{} {}", attrs, self.name),
        }
    }
}

impl Variant {
    // TODO make this into a fold
    fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool {
        match self.typ {
            Some(ref typ) => typ.contains_unboxed_id(id, map),
            None => false,
        }

    }
    pub(crate) fn get_named_type(&self) -> Option<&Id> {
        match self.typ {
            Some(ref typ) => typ.get_named_root(),
            None => None,
        }

    }
}

fn render_delimited<T: fmt::Display>(items: &[T], delimiter: &str) -> String {
    items
        .iter()
        .map(|item| format!("{}", item))
        .collect::<Vec<String>>()
        .join(delimiter)
        .to_string()
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
            Custom(ref name) => write!(f, "{}", name),
        }
    }
}

// TODO not sure if we want/need separate fieldattr and variantattr enums
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
                    Type::Box(Box::new(Type::named("Type2").unwrap())),
                    vec![SerdeRename("Field-2".into()), SerdeDefault]
                ),
                Field::with_rename("Snake Case Me", Type::named("Type3").unwrap())
                    .unwrap(),
            ],
        );

        let pretty = rust_format(&my_struct.to_string()).unwrap();
        let expect = r#"#[derive(Debug, Clone)]
#[cfg(test, target_os = "linux")]
pub struct MyStruct {
    field1: Type1,
    #[serde(rename = "Field-2")]
    #[serde(default)]
    field2: Box<Type2>,
    #[serde(rename = "Snake Case Me")]
    snake_case_me: Type3,
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
                ),
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
            Type::named("MyAliasedType").unwrap(),
        );
        let pretty = rust_format(&a.to_string()).unwrap();
        let expect = "pub(crate) type MyAlias = MyAliasedType;\n";
        assert_eq!(pretty, expect);
    }
}
