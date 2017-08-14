use std::fmt;

pub enum TypeBuilder {
    Primitive(Primitive),
    Box(Box<TypeBuilder>),
    Vec(Box<TypeBuilder>),
    Option(Box<TypeBuilder>),
    Result(Box<TypeBuilder>, Box<TypeBuilder>),
    Named(String),
    Ref(Box<TypeBuilder>),
}

impl TypeBuilder {
    fn render(&self) -> String {
        use self::TypeBuilder::*;
        match *self {
            Primitive(ref primitive) => primitive.native().to_string(),
            Box(ref tb) => format!("Box<{}>", tb.render()),
            Vec(ref tb) => format!("Vec<{}>", tb.render()),
            Option(ref tb) => format!("Option<{}>", tb.render()),
            Result(ref tb1, ref tb2) => format!("Result<{}, {}>", tb1.render(), tb2.render()),
            Named(ref name) => name.clone(),
            Ref(ref tb) => format!("&{}", tb.render())
        }
    }
}

impl fmt::Display for TypeBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Primitive {
    Null,
    Boolean,
    Integer,
    Number,
    String,
}

impl Primitive {
    fn native(&self) -> &str {
        use self::Primitive::*;
        match *self {
            Null => "()",
            Boolean => "bool",
            Integer => "i64",
            Number => "f64",
            String => "String",
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_builder() {
        let typ = TypeBuilder::Box(
            Box::new(TypeBuilder::Result(
                Box::new(TypeBuilder::Named("ResultLeft".into())),
                Box::new(TypeBuilder::Vec(
                    Box::new(TypeBuilder::Option(
                        Box::new(TypeBuilder::Ref(
                            Box::new(TypeBuilder::Primitive(Primitive::String)))))))))));
        assert_eq!(typ.render(), "Box<Result<ResultLeft, Vec<Option<&String>>>>");
    }
}
