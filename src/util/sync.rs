use std::fmt;

pub use inner::*;

#[cfg(not(feature = "sync"))]
mod inner {
    use std::{cell::RefCell, rc::Rc};

    #[repr(transparent)]
    pub struct RefCount<T>(pub(super) Rc<RefCell<T>>);

    #[repr(transparent)]
    pub struct Weak<T>(pub(super) std::rc::Weak<RefCell<T>>);

    pub type Ref<'a, T> = std::cell::Ref<'a, T>;

    pub type RefMut<'a, T> = std::cell::RefMut<'a, T>;

    impl<T> RefCount<T> {
        pub fn new(inner: T) -> Self {
            Self(Rc::new(RefCell::new(inner)))
        }

        pub fn clone(this: &Self) -> Self {
            Self(Rc::clone(&this.0))
        }

        pub fn downgrade(&self) -> Weak<T> {
            Weak(Rc::downgrade(&self.0))
        }

        pub fn get(&self) -> Ref<'_, T> {
            self.0.borrow()
        }

        pub fn get_mut(&self) -> RefMut<'_, T> {
            self.0.borrow_mut()
        }
    }
}

#[cfg(feature = "sync")]
mod inner {
    use std::{
        marker::PhantomData,
        ops,
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    #[repr(transparent)]
    pub struct RefCount<T>(pub(super) Arc<RwLock<T>>);

    #[repr(transparent)]
    pub struct Weak<T>(pub(super) std::sync::Weak<RwLock<T>>);

    pub struct Ref<'a, T: ?Sized>(RwLockReadGuard<'a, T>);

    pub type RefMut<'a, T> = RwLockWriteGuard<'a, T>;

    impl<T> RefCount<T> {
        pub fn new(inner: T) -> Self {
            Self(Arc::new(RwLock::new(inner)))
        }

        pub fn clone(this: &Self) -> Self {
            Self(Arc::clone(&this.0))
        }

        pub fn downgrade(&self) -> Weak<T> {
            Weak(Arc::downgrade(&self.0))
        }

        pub fn get(&self) -> Ref<'_, T> {
            Ref(self.0.read().unwrap())
        }

        pub fn get_mut(&self) -> RefMut<'_, T> {
            self.0.write().unwrap()
        }
    }

    impl<T> Ref<'_, T> {
        pub const fn map<U: ?Sized, F>(orig: Ref<'_, T>, f: F) -> RefWrap<'_, T, U, F>
        where
            F: Copy + FnOnce(&T) -> &U,
        {
            RefWrap {
                orig,
                access: f,
                _phantom: PhantomData,
            }
        }
    }

    pub struct RefWrap<'a, T, U: ?Sized, F> {
        orig: Ref<'a, T>,
        access: F,
        _phantom: PhantomData<U>,
    }

    impl<T, U: ?Sized, F> ops::Deref for RefWrap<'_, T, U, F>
    where
        F: Copy + FnOnce(&T) -> &U,
    {
        type Target = U;

        fn deref(&self) -> &Self::Target {
            (self.access)(&self.orig)
        }
    }

    impl<T: ?Sized> ops::Deref for Ref<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            ops::Deref::deref(&self.0)
        }
    }
}

impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<RefCount<T>> {
        self.0.upgrade().map(RefCount)
    }
}

impl<T: fmt::Debug> fmt::Debug for RefCount<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

/// ```compile_fail
/// use rosu_pp::{taiko::TaikoGradualDifficulty, Beatmap, Difficulty};
///
/// let map = Beatmap::from_bytes(&[]).unwrap();
/// let difficulty = Difficulty::new();
/// let mut gradual = TaikoGradualDifficulty::new(&difficulty, &map).unwrap();
///
/// // Rc<RefCell<_>> cannot be shared across threads so compilation should fail
/// std::thread::spawn(move || { let _ = gradual.next(); });
/// ```
#[cfg(not(feature = "sync"))]
const fn _share_gradual_taiko() {}

#[cfg(all(test, feature = "sync"))]
mod tests {
    #[test]
    fn share_gradual_taiko() {
        use crate::{Beatmap, Difficulty, taiko::TaikoGradualDifficulty};

        let map = Beatmap::from_bytes(&[]).unwrap();
        let mut gradual = TaikoGradualDifficulty::new(Difficulty::new(), &map).unwrap();

        // Arc<RwLock<_>> *can* be shared across threads so this should compile
        std::thread::spawn(move || {
            let _ = gradual.next();
        });
    }
}
