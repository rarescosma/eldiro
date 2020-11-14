const WHITESPACE: &[char] = &[' ', '\n'];

pub(crate) fn extract_digits(s: &str) -> Result<(&str, &str), String> {
    take_while1(
        |c| c.is_ascii_digit(),
        s,
        "expected digits".to_string(),
    )
}

pub(crate) fn extract_whitespace(s: &str) -> (&str, &str) {
    take_while(|c| WHITESPACE.contains(&c), s)
}

pub(crate) fn extract_whitespace1(s: &str) -> Result<(&str, &str), String> {
    take_while1(|c| WHITESPACE.contains(&c), s, "expected a space".to_string())
}

pub(crate) fn extract_ident(s: &str) -> Result<(&str, &str), String> {
    let starts_with_alphabetic = s
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false);

    if starts_with_alphabetic {
        Ok(take_while(|c| c.is_ascii_alphanumeric(), s))
    } else {
        Err("expected identifier".to_string())
    }
}

pub(crate) fn take_while(accept: impl Fn(char) -> bool, s: &str) -> (&str, &str) {
    let take_end = s
        .char_indices()
        .find_map(
            |(idx, c)| if accept(c) { None } else { Some(idx) }
        )
        .unwrap_or_else(|| s.len());
    (&s[take_end..], &s[..take_end])
}

pub(crate) fn take_while1(
    accept: impl Fn(char) -> bool,
    s: &str,
    error_msg: String,
) -> Result<(&str, &str), String> {
    let (remainder, extracted) = take_while(accept, s);

    if extracted.is_empty() {
        Err(error_msg)
    } else {
        Ok((remainder, extracted))
    }
}

/*
Those lifetimes on tag might look scary,
but all they’re doing is telling Rust that
the lifetimes of s and the output are related,
while the lifetimes of prefix and the output aren’t.

If we didn’t have the lifetimes,
Rust wouldn’t know if the returned value has to live
as long as prefix, or s, or both.
*/
pub(crate) fn tag<'a, 'b>(prefix: &'a str, s: &'b str) -> Result<&'b str, String> {
    if s.starts_with(prefix) {
        Ok(&s[prefix.len()..])
    } else {
        Err(format!("expected {}", prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_one_digit() {
        assert_eq!(
            extract_digits("1+2"),
            Ok(("+2", "1"))
        );
    }

    #[test]
    fn extract_multiple_digits() {
        assert_eq!(
            extract_digits("10-20"),
            Ok(("-20", "10"))
        );
    }

    #[test]
    fn do_not_extract_digits_from_invalid_input() {
        assert_eq!(extract_digits("abcd"), Err("expected digits".to_string()));
    }

    #[test]
    fn extract_digits_with_no_remainder() {
        assert_eq!(extract_digits("100"), Ok(("", "100")));
    }

    #[test]
    fn extract_spaces() {
        assert_eq!(extract_whitespace("    1"), ("1", "    "));
    }

    #[test]
    fn extract_newlines_or_spaces() {
        assert_eq!(extract_whitespace(" \n   \n\nabc"), ("abc", " \n   \n\n"));
    }

    #[test]
    fn do_not_extract_spaces1_when_input_does_not_start_with_them() {
        assert_eq!(
            extract_whitespace1("blah"),
            Err("expected a space".to_string()),
        );
    }

    #[test]
    fn extract_alphabetic_ident() {
        assert_eq!(extract_ident("abcdEFG stop"), Ok((" stop", "abcdEFG")));
    }

    #[test]
    fn extract_alphanumeric_ident() {
        assert_eq!(extract_ident("bazbleh13()"), Ok(("()", "bazbleh13")));
    }

    #[test]
    fn will_not_extract_ident_beginning_with_number() {
        assert_eq!(
            extract_ident("123abc"),
            Err("expected identifier".to_string()),
        );
    }

    #[test]
    fn tag_word() {
        assert_eq!(tag("let", "let a"), Ok(" a"))
    }
}
