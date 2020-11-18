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

impl<'a, D> Iterator for StrSplit<'a, D> where D: Delimiter {
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
        let res: Vec<_> = StrSplit::new(
            "a b c", " ",
        ).collect();
        assert_eq!(res, vec!["a", "b", "c"]);
    }

    #[test]
    fn it_works_with_char_delimiter() {
        let res: Vec<_> = StrSplit::new(
            "hello world", 'o',
        ).collect();
        assert_eq!(res, vec!["hell", " w", "rld"]);
    }
}
