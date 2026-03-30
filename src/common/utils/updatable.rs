pub struct Updatable<T> {
    last: T,
    current: T,
}

impl<T: Clone> Updatable<T> {
    pub fn new(value: T) -> Self {
        Self {
            current: value.clone(),
            last: value,
        }
    }

    pub fn update(&mut self, value: T) {
        self.last = std::mem::replace(&mut self.current, value);
    }

    pub fn last(&self) -> &T {
        &self.last
    }

    pub fn current(&self) -> &T {
        &self.current
    }
}
