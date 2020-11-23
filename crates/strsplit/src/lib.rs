/// This one illustrates timelines and traits
/// as well as the use of options and slices:
///
/// Our simple Iterator implementation is generic over a type D
/// with a trait bound of "Delimiter", which means types that want
/// to work with our iterator need to implement "Delimiter", in other words:
/// the *only* thing we require of D is its ability to find a "something"
/// within itself, expressed by the signature of `find_next`:
///
/// ```
/// pub trait Delimiter {
///   fn find_next(&self, s: &str) -> Option<(usize, usize)>;
/// }
/// ```
///
/// It also illustrates some of the APIs over `Option` and `Result`.
///
/// For `Result`, if you have a function that returns it, and you make calls
/// to other function that return `Result` (by say, an assignment), you can
/// use the `?` suffix to short-circuit the return, if the inner function
/// failed. No more `if` for error handling, yay!
///
/// `Option` can be deconstructed straight by assignments:
///
/// ```
/// fn bar() -> Option<Foo> {}
///
/// fn foo() {
///   if let Some(foo) = bar() {
///     // will be reached when the Option was Some
///   }
/// }
/// ```

pub struct StrSplit<'a, D> {
    remainder: Option<&'a str>,
    delimiter: D,
}

impl<'a, D> StrSplit<'a, D> {
    pub fn new(haystack: &'a str, delimiter: D) -> Self {
        Self {
            remainder: Some(haystack),
            delimiter,
        }
    }
}

impl<'a, D> Iterator for StrSplit<'a, D>
where
    D: Delimiter,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.as_mut()?;
        if let Some((delim_start, delim_end)) = self.delimiter.find_next(remainder) {
            let item = &remainder[..delim_start];
            *remainder = &remainder[delim_end..];
            Some(item)
        } else {
            self.remainder.take()
        }
    }
}

pub trait Delimiter {
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;
}

impl Delimiter for &str {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.find(self).map(|start| (start, start + self.len()))
    }
}

impl Delimiter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.char_indices()
            .find(|(_, c)| c == self)
            .map(|(pos, _)| (pos, pos + self.len_utf8()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works_with_str_delimiter() {
        let res: Vec<_> = StrSplit::new("a b c", " ").collect();
        assert_eq!(res, vec!["a", "b", "c"]);
    }

    #[test]
    fn it_works_with_char_delimiter() {
        let res: Vec<_> = StrSplit::new("hello world", 'o').collect();
        assert_eq!(res, vec!["hell", " w", "rld"]);
    }
}
