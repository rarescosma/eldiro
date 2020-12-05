use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

struct Inner<T> {
    value: T,
    refcount: Cell<usize>,
}

struct Rc<T> {
    inner: NonNull<Inner<T>>,
    _marker: PhantomData<Inner<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::new(Inner {
            value,
            refcount: Cell::new(1),
        });
        Self {
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        inner.refcount.set(c + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    // SAFETY: self.inner is a Box that is only deallocated when the last Rc
    // goes away. We have one, therefore the Box has not been deallocated,
    // so deref is fine.
    fn deref(&self) -> &Self::Target {
        &unsafe { self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        if c == 1 {
            // SAFETY: we are the only one, and being dropped.
            // Restore the Box and drop it immediately.
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            // there are other RCs, so don't drop the Box!
            inner.refcount.set(c - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_works() {
        let val = "foo".to_owned();
        let _rc1 = Rc::new(val);
        let _rc2 = Rc::clone(&_rc1);

        assert_eq!(*_rc1, "foo");
        assert_eq!(*_rc2, "foo");
        unsafe {
            assert_eq!((*(_rc1.inner.as_ref())).refcount.get(), 2);
        }
        unsafe {
            assert_eq!(
                (*(_rc1.inner.as_ref())).refcount,
                (*(_rc2.inner.as_ref())).refcount
            );
        }
        drop(_rc1);
        unsafe {
            assert_eq!((*(_rc2.inner.as_ref())).refcount.get(), 1);
        }
    }
}
