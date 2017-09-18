use std::borrow::Cow;
use std::collections::BTreeSet;

use keywords;
use errors::*;

lazy_static! {
    static ref RUST_KEYWORDS: BTreeSet<&'static str> = {
        keywords::RUST_KEYWORDS.iter().map(|v| *v).collect()
    };
}

pub(crate) fn validate_identifier(ident: &str) -> Result<()> {
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

pub(crate) fn make_valid_identifier(ident: &str) -> Result<Cow<str>> {
    // strip out invalid characters and ensure result is valid
    // bit ugly to reallocate but at least it is simple
    // TODO use unicode XID_start/XID_continue
    if let Ok(()) = validate_identifier(ident) {
        // happy path
        return Ok(Cow::Borrowed(ident))
    }
    let mut out = String::new();
    let mut is_leading_char = true;
    for c in ident.chars() {
        if is_leading_char {
            match c {
                'A'...'Z' | 'a'...'z' | '_' => {
                    is_leading_char = false;
                    out.push(c);
                }
                _ => (),
            }
        } else {
            match c {
                'A'...'Z' | 'a'...'z' | '_' | '0'...'9' => out.push(c),
                _ => (),
            }
        }
    }
    if RUST_KEYWORDS.contains(&*out) {
        out.push('_')
    };
    if out.len() == 0 || out == "_" {
        bail!("could not generate valid identifier from {}", ident)
    }
    Ok(Cow::Owned(out))
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
    // FIXME this error will trigger if the input is *correctly* unchanged
    if buf == code {
        bail!("Syntax error detected")
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_make_valid_identifier() {
        let id1 = "1234_abcd".into();
        assert_eq!(make_valid_identifier(id1).unwrap(), "_abcd");
        let id2 = "$1234Abcd".into();
        assert_eq!(make_valid_identifier(id2).unwrap(), "Abcd");
        let id3 = "$@1234\\|./".into();
        assert!(make_valid_identifier(id3).is_err());
        let id4 = "1234_".into();
        assert!(make_valid_identifier(id4).is_err());
        let id5 = "".into();
        assert!(make_valid_identifier(id5).is_err());
        let id6 = "_".into();
        assert!(make_valid_identifier(id6).is_err());
        let id7 = "type".into();
        assert_eq!(make_valid_identifier(id7).unwrap(), "type_");
        let id8 = "this 123".into();
        assert_eq!(make_valid_identifier(id8).unwrap(), "this123");
    }

}
