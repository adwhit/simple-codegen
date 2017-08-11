#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate quote;
extern crate rustfmt;

use quote::{Tokens, ToTokens, Ident};

use errors::*;

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

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    derive: Vec<DeriveAttr>,
    cfg: Vec<CfgAttr>,
    custom: Vec<String>,
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
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub typ: String,
    pub attrs: Vec<FieldAttr>  // TODO separate field attrs?
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let name = Ident::from(&*self.name);
        let attrs = &self.attrs;
        let typ = Ident::from(&*self.typ);
        let tok =
            quote! {
                #attrs
                #name: #typ
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

pub fn rust_format(t: &Tokens) -> Result<String> {
    use rustfmt::*;
    use std::fs::File;
    use std::io::prelude::*;

    // FIXME workaround is necessary until rustfmt works programmatically
    let tmppath = "/tmp/rustfmt.rs"; // TODO use tempdir
    //let tmppath = "/home/alex/scratch/stubgen/src/gen.rs"; // TODO use tempdir
    {
        let mut tmp = File::create(tmppath)?;
        tmp.write_all(t.as_str().as_bytes())?;
    }

    let input = Input::File(tmppath.into());
    let mut fakebuf = Vec::new(); // pretty weird that this is necessary.. but it is

    match format_input(input, &Default::default(), Some(&mut fakebuf)) {
        Ok((_summmary, _filemap, _report)) => {}
        Err((e, _summary)) => Err(e)?,
    }

    let mut tmp = File::open(tmppath)?;
    let mut buf = String::new();
    tmp.read_to_string(&mut buf)?;
    Ok(buf)
}

#[derive(Clone, Debug)]
enum FieldAttr {
    SerdeRename(String),
    SerdeDefault,
    Custom(String),
}

#[derive(Clone, Debug)]
enum VariantAttr {
    SerdeRename(String),
    SerdeSkipSerialize,
    SerdeSkipDeserialize,
    Custom(String),
}

#[derive(Clone, Debug)]
enum DeriveAttr {
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Custom(String)
}

enum CfgAttr {
    Test,
    TargetOs(String)
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
                Field {
                    name: "field1".into(),
                    typ: "Type1".into(),
                    attrs: Default::default()
                },
                Field {
                    name: "field2".into(),
                    typ: "Type2".into(),
                    attrs: vec![FieldAttr::SerdeRename("Field-2".into())]
                }
                ]
        };

        let tokens = quote!{#s};
        let pretty = rust_format(&tokens).unwrap();
        println!("\n{}", pretty);
        assert!(false);

    }
}
