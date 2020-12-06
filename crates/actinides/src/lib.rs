use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

/*
want to make a thread-safe multi-writer thing that uses atomics to
detect if there's already an "active" writer.
 */
const YIELD_AFTER: i32 = 25;

pub struct WriteGuard<'wg, T> {
    t: &'wg mut T,
    handle: &'wg WriteHandle<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.t
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.t
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.handle._writing.fetch_and(false, Ordering::Release);
    }
}

pub struct WriteHandle<T> {
    t: NonNull<T>,
    _writing: Arc<AtomicBool>,
    _refcount: Arc<AtomicI32>,
    _marker: PhantomData<T>, // 👻
}

impl<T> WriteHandle<T> {
    pub fn new(t: T) -> Self {
        Self {
            t: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(t))) },
            _writing: Arc::new(AtomicBool::from(false)),
            _refcount: Arc::new(AtomicI32::from(1)),
            _marker: PhantomData,
        }
    }

    fn lock(&self) {
        let mut i = 0;
        loop {
            if self._writing.compare_and_swap(false, true, Ordering::Acquire) == false {
                return;
            };
            if i == YIELD_AFTER {
                thread::yield_now();
            } else {
                i += 1;
            }
        }
    }

    pub fn write(&self) -> WriteGuard<T> {
        self.lock();
        WriteGuard {
            t: unsafe { self.t.as_ptr().as_mut() }.unwrap(),
            handle: self,
        }
    }
}

impl<T> Clone for WriteHandle<T> {
    fn clone(&self) -> Self {
        self._refcount.fetch_add(1, Ordering::Acquire);
        Self {
            t: self.t,
            _writing: self._writing.clone(),
            _refcount: self._refcount.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for WriteHandle<T> {
    type Target = T;

    // SAFETY: self.t is a Box that is only deallocated when the last
    // WriteHandle goes away.
    fn deref(&self) -> &Self::Target {
        unsafe { self.t.as_ref() }
    }
}

impl<T> Drop for WriteHandle<T> {
    fn drop(&mut self) {
        if self._refcount.fetch_sub(1, Ordering::Release) == 1 {
            // Wrap the raw pointer in a Box and immediately drop it!
            let _ = unsafe { Box::from_raw(self.t.as_ptr()) };
        }
    }
}

unsafe impl<T> Send for WriteHandle<T> {}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    impl<T> WriteHandle<T> {
        fn refcount(&self) -> i32 {
            self._refcount.load(Ordering::SeqCst)
        }

        fn writing(&self) -> bool {
            self._writing.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_writing() {
        let sut = WriteHandle::new(17);
        assert_eq!(sut.writing(), false);

        {
            let mut _foo = sut.write();
            *_foo += 3;
            assert_eq!(sut.writing(), true);
        }

        assert_eq!(sut.writing(), false);
        unsafe {
            assert_eq!(sut.t.as_ref(), &20);
        }
    }

    #[test]
    fn test_two_writers() {
        let sut = WriteHandle::new(17);
        {
            let s1 = sut.clone();
            let s2 = sut.clone();
            assert_eq!(sut.refcount(), 3);

            let jh1 = thread::spawn(move || {
                for _ in 0..1500 {
                    *s1.write() += 1;
                }
            });
            let jh2 = thread::spawn(move || {
                for _ in 0..500 {
                    *s2.write() += 2;
                }
            });

            jh1.join().expect("jh1 panic");
            jh2.join().expect("jh2 panic");
        }

        assert_eq!(*sut, 2517);
        assert_eq!(sut.refcount(), 1);
    }

    #[test]
    fn test_many_writers() {
        let sut = WriteHandle::new(17);
        for _ in 0..10 {
            let wh = sut.clone();
            thread::spawn(move || {
                for _ in 0..10 {
                    let inner_wh = wh.clone();
                    thread::spawn(move || {
                        *inner_wh.write() += 1;
                    })
                    .join()
                    .expect("nope");
                }
            })
            .join()
            .expect("nope");
        }

        assert_eq!(*sut, 117);
        assert_eq!(sut.writing(), false);
        assert_eq!(sut.refcount(), 1);
    }

    #[test]
    fn test_panic_handling() {
        let sut = WriteHandle::new(17);
        {
            let s1 = sut.clone();
            let s2 = sut.clone();
            assert_eq!(sut.refcount(), 3);

            let jh1 = thread::spawn(move || {
                for _ in 0..500 {
                    *s1.write() += 1;
                }
            });
            let r = std::panic::catch_unwind(move || {
                for _ in 0..500 {
                    *s2.write() += 1;
                }
                let _dangling = s2.write();
                panic!();
            });
            assert!(r.is_err());
            jh1.join().expect("jh1 panic");
        }

        // this shouldn't deadlock!
        *sut.write() += 1;

        assert_eq!(*sut, 1018);
        assert_eq!(sut.refcount(), 1);
    }
}
