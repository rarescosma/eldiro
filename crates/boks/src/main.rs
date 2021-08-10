#![feature(dropck_eyepatch)]

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct Boks<T> {
    p: NonNull<T>,
    _p: PhantomData<T>,
}

unsafe impl<#[may_dangle] T> Drop for Boks<T> {
    fn drop(&mut self) {
        // This will construct a `Box` from the raw pointer and immediately drop it.
        unsafe { Box::from_raw(self.p.as_ptr()) };
    }
}

impl<T> Boks<T> {
    pub fn ny(t: T) -> Self {
        Boks {
            p: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(t))) },
            _p: PhantomData,
        }
    }
}

impl<T> Deref for Boks<T> {
    type Target = T;

    // Reference is valid since it was constructed from a valid T and turned into a pointer
    // through Box (which creates aligned non-null pointers), and hasn't been freed, since
    // self itself is alive.
    fn deref(&self) -> &Self::Target {
        unsafe { self.p.as_ref() }
    }
}

impl<T> DerefMut for Boks<T> {
    // NOTE: don't need to specity `Target` since `DerefMut` is a subtrait of `Deref`
    // and the compiler understands that Deref -> Target is the associated type.

    // Reference is valid since it was constructed from a valid T and turned into a pointer
    // through Box (which creates aligned non-null pointers), and hasn't been freed, since
    // self itself is alive.
    // Also, since we have `&mut self`, no other mutable reference to `p` has been given out.
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.p.as_mut() }
    }
}

struct Oisann<T: Debug>(T);

// This would break example 3 because the drop implementation will
// prevent the compiler from shortening the lifetime of the mutable borrow
// before the `println!` statement.

// impl<T> Drop for Oisann<T>
// where
//     T: Debug,
// {
//     fn drop(&mut self) {
//         println!("{:?}", self.0)
//     }
// }

// To make a type that's generic _and_ covariant over <T>
// but that doesn't trigger the drop-check for <T> use this construct:
#[derive(Default)]
struct Deserializer<T> {
    _t: PhantomData<fn() -> T>,
}

fn main() {
    let x = 42;
    let b1 = Boks::ny(x);
    println!("{:?}", *b1);

    let mut y = 42;
    let _b2 = Boks::ny(&mut y);
    println!("{:?}", y);

    // Example 3
    let mut z = 42;
    let _b3 = Boks::ny(Oisann(&mut z));
    println!("{:?}", z);

    // This would not compile if we were invariant in the type T
    let s = String::from("hei");
    let mut _b4 = Boks::ny(&*s);
    let b5: Boks<&'static str> = Boks::ny("heisann");
    _b4 = b5;

    let _d = Deserializer::<u8>::default();
}
