use std::mem;

pub enum FunList {
    Empty,
    Elem(i32, Box<List>),
}

/// Avoids allocating the Empty case, reducing the total number of
/// heap allocations by 1.
///
/// But... loses out on null pointer optimization
pub enum AnotherList {
    Empty,
    ElemThenEmpty(i32),
    ElemThenNotEmpty(i32, Box<List>),
}

/*
Every enum has to store a tag to specify which variant of the enum
its bits represent. However, if we have a special kind of enum:

```
enum Foo {
    A,
    B(ContainsANonNullPtr),
}
```

the null pointer optimization kicks in: it eliminates the space needed
for the tag. If the variant is A, the whole enum is set to all 0's.
Otherwise, the variant is B. This works because B can never be all 0's,
since it contains a non-zero pointer. Slick!
*/

/// * Tail of a list never allocates extra junk: check!
/// * enum is in delicious null-pointer-optimized form: check!
/// * All elements are uniformly allocated: check!
pub struct List {
    head: Link,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

impl Default for Link {
    fn default() -> Self {
        Self::Empty
    }
}
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = self.pop_node();
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::take(&mut boxed_node.next);
        }
    }
}

impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem,
            next: mem::take(&mut self.head),
        });
        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.pop_node() {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }

    fn pop_node(&mut self) -> Link {
        mem::take(&mut self.head)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        list.push(4);

        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
        assert_eq!(list.pop(), None);
    }
}
