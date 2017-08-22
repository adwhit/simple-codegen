# Simple Codegen
Simple code generation for Rust

This library provides utilities to help users generate valid Rust code.

The focus is on the 80% use case. It tries to make it the simple, common cases easy to write,
but should not stop the user from building more complicated constructs.


Example:

```rust
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
    field2: Type2,
    #[serde(rename = "Snake Case Me")]
    snake_case_me: Type3,
}
"#;
assert_eq!(pretty, expect);
```
