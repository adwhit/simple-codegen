use std::fmt;
use errors::*;
use Id;
use items::ItemMap;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Box(Box<Type>),
    Vec(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Named(Id),
    Ref(Box<Type>),
}

impl Type {
    pub fn named<I: Into<String>>(name: I) -> Result<Type> {
        Ok(Type::Named(Id::new(name)?))
    }

    fn render(&self) -> String {
        use self::Type::*;
        match *self {
            Primitive(ref primitive) => primitive.native().to_string(),
            Box(ref tb) => format!("Box<{}>", tb.render()),
            Vec(ref tb) => format!("Vec<{}>", tb.render()),
            Option(ref tb) => format!("Option<{}>", tb.render()),
            Result(ref tb1, ref tb2) => format!("Result<{}, {}>", tb1.render(), tb2.render()),
            Named(ref name) => name.to_string(),
            Ref(ref tb) => format!("&{}", tb.render()),
        }
    }

    pub(crate) fn named_root(&self) -> Option<&Id> {
        use self::Type::*;
        match *self {
            Primitive(_) => None,
            Box(ref tb) => tb.named_root(),
            Vec(ref tb) => tb.named_root(),
            Option(ref tb) => tb.named_root(),
            Result(ref tb1, ref tb2) => tb1.named_root(), // FIXME discard tb2?
            Named(ref name) => Some(name),
            Ref(ref tb) => tb.named_root(),
        }
    }

    pub(crate) fn is_defaultable(&self, map: &ItemMap) -> bool {
        use self::Type::*;
        match *self {
            Primitive(_) => true,
            Box(ref tb) => tb.is_defaultable(map),
            Vec(_) => true,
            Option(_) => true,
            Result(_, _) => false,
            Named(ref name) => {
                map.get(name)
                    .map(|item| item.is_defaultable(&map))
                    .unwrap_or(false)
            }
            Ref(_) => false,
        }
    }

    pub(crate) fn contains_unboxed_id(&self, id: &Id, map: &ItemMap) -> bool {
        use self::Type::*;
        match *self {
            Option(ref tb) => tb.contains_unboxed_id(id, map),
            Result(ref tb1, ref tb2) => tb1.contains_unboxed_id(id, map) && tb2.contains_unboxed_id(id, map),
            Named(ref name) => {
                map.get(name)
                    .map(|item| item.contains_unboxed_id(id, map))
                    .unwrap_or(false)
            }
            Primitive(_) => false,
            Ref(_) => false,
            Box(_) => false,
            Vec(_) => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Represents a primitive Rust type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Primitive {
    Null,
    Boolean,
    Integer,
    Float,
    String,
}

impl Primitive {
    fn native(&self) -> &str {
        use self::Primitive::*;
        match *self {
            Null => "()",
            Boolean => "bool",
            Integer => "i64",
            Float => "f64",
            String => "String",
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.native())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_builder() {
        let typ = Type::Box(Box::new(Type::Result(
            Box::new(Type::Named(Id::new("ResultLeft").unwrap())),
            Box::new(Type::Vec(Box::new(Type::Option(Box::new(
                Type::Ref(Box::new(Type::Primitive(Primitive::String))),
            ))))),
        )));
        assert_eq!(
            typ.render(),
            "Box<Result<ResultLeft, Vec<Option<&String>>>>"
        );
    }
}
