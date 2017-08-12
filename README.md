# Simple Codegen
Simple code generation for Rust

This library provides utilities to help users generate valid Rust code.

The focus is on the 80% use case. It tries to make it the simple, common cases easy to write,
but should not stop the user from building more complicated constructs.


Example:

```rust
let my_struct = Struct {
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

let pretty = rust_format(&my_struct.to_string()).unwrap();
let expect =
r#"#[derive(Clone, Debug)]
#[cfg(test, target_os = "linux")]
pub struct MyStruct {
    field1: Type1,
    #[serde(rename = "Field-2")]
    #[serde(default)]
    field2: Type2,
}
"#;
assert_eq!(pretty, expect);
```
