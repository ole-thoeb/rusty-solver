pub trait IterUtil: Iterator {
    fn collect_vec_with_capacity(self, capacity: usize) -> Vec<Self::Item> where Self: Sized {
        let mut vec = Vec::with_capacity(capacity);
        vec.extend(self);
        vec
    }
}

impl<T> IterUtil for T where T: Iterator + ?Sized {}

pub fn flatten_interleaving<I>(iters: Vec<I>) -> FlattenInterleaving<I> where I: Iterator {
    FlattenInterleaving::new(iters)
}

#[derive(Debug)]
pub struct FlattenInterleaving<I: Iterator> {
    iters: Vec<I>,
    index: usize,
}

impl <I: Iterator> FlattenInterleaving<I> {
    fn new(iters: Vec<I>) -> FlattenInterleaving<I> {
        FlattenInterleaving {
            iters,
            index: 0,
        }
    }
}

impl <I: Iterator> Iterator for FlattenInterleaving<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.iters.len();
        for _ in 0..len {
            if let Some(item) = self.iters[self.index].next() {
                self.index = (self.index + 1) % len;
                return Some(item);
            }
            self.index = (self.index + 1) % len;
        }
        None
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn flatten_interleaving() {
        let input = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]].into_iter().map(|v| v.into_iter()).collect();
        let output: Vec<_> = super::flatten_interleaving(input).collect();
        assert_eq!(output, vec![1, 4, 7, 2, 5, 8, 3, 6, 9]);
    }
}