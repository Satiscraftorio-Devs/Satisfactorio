pub struct Updatable<T> {
    last: T,
    current: T,
}

impl<T: Clone> Updatable<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        Self {
            current: value.clone(),
            last: value,
        }
    }

    #[inline(always)]
    pub fn update(&mut self, value: T) {
        self.last = std::mem::replace(&mut self.current, value);
    }

    #[inline(always)]
    pub fn last(&self) -> &T {
        &self.last
    }

    #[inline(always)]
    pub fn current(&self) -> &T {
        &self.current
    }
}

impl<T: PartialEq> Updatable<T> {
    #[inline(always)]
    pub fn has_changed(&self) -> bool {
        self.current != self.last
    }
}
