#[derive(Copy, Clone)]
pub enum Multi<T> {
    None,
    One(T),
    Two(T, T),
    Three(T, T, T),
}

impl<T> Multi<T> {
    pub fn take(&mut self) -> Option<T> {
        match core::mem::take(self) {
            Multi::None => None,
            Multi::One(t) => {
                *self = Multi::None;
                Some(t)
            }
            Multi::Two(t, r) => {
                *self = Multi::One(r);
                Some(t)
            }
            Multi::Three(t, r, s) => {
                *self = Multi::Two(r, s);
                Some(t)
            }
        }
    }

    pub fn push(&mut self, t: T) {
        match core::mem::take(self) {
            Multi::None => *self = Multi::One(t),
            Multi::One(r) => *self = Multi::Two(t, r),
            Multi::Two(r, s) => *self = Multi::Three(t, r, s),
            _ => *self = Multi::None,
        }
    }

    pub fn add(self, other: Multi<T>) -> Multi<T> {
        match (self, other) {
            (Multi::None, other) => other,
            (Multi::One(t), Multi::One(r)) => Multi::Two(t, r),
            (Multi::One(t), Multi::Two(r, s)) => Multi::Three(t, r, s),
            (Multi::Two(t, r), Multi::One(s)) => Multi::Three(t, r, s),
            _ => Multi::None,
        }
    }
}

impl<T> Default for Multi<T> {
    fn default() -> Self {
        Multi::None
    }
}

impl<T> IntoIterator for Multi<T> {
    type Item = T;
    type IntoIter = MultiIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        MultiIter(self)
    }
}

pub struct MultiIter<T>(Multi<T>);

impl<T> Iterator for MultiIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.take()
    }
}
