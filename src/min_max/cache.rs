use ahash::{HashMap};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CacheFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct CacheEntry {
    pub(super) value: i32,
    pub(super) level: u8,
    pub(super) flag: CacheFlag,
}

pub trait Cache<S> {
    fn set(&mut self, state: &S, entry: CacheEntry);
    fn get(&mut self, state: &S) -> Option<CacheEntry>;
}

#[derive(Debug, Clone)]
pub struct HashMapCache<S>(HashMap<S, CacheEntry>);

impl <S> HashMapCache<S> {
    pub fn new(map: HashMap<S, CacheEntry>) -> Self {
        Self(map)
    }
}

impl <S> Default for HashMapCache<S> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<S> Cache<S> for HashMapCache<S> where S: Eq + std::hash::Hash + Clone {
    fn set(&mut self, state: &S, entry: CacheEntry) {
        self.0.insert(state.clone(), entry);
    }

    fn get(&mut self, state: &S) -> Option<CacheEntry> {
        self.0.get(state).cloned()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct NullCache;

impl<S> Cache<S> for NullCache {
    fn set(&mut self, _state: &S, _entry: CacheEntry) {}

    fn get(&mut self, _state: &S) -> Option<CacheEntry> {
        None
    }
}