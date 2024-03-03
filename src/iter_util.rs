pub trait IterUtil: Iterator {
    fn collect_vec_with_capacity(self, capacity: usize) -> Vec<Self::Item> where Self: Sized {
        let mut vec = Vec::with_capacity(capacity);
        vec.extend(self);
        vec
    }
}

impl<T> IterUtil for T where T: Iterator + ?Sized {}