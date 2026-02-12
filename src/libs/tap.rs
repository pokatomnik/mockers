pub(crate) trait Tap: Sized {
    fn tap(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

impl<T> Tap for T {}
