pub trait Stats {
    fn record_prune(&mut self);
    fn record_cache_hit(&mut self);
    fn record_cache_miss(&mut self);
    fn record_state_scored(&mut self);
}

#[derive(Debug, Default)]
pub struct NullStats;

impl Stats for NullStats {
    fn record_prune(&mut self) {}
    fn record_cache_hit(&mut self) {}
    fn record_cache_miss(&mut self) {}
    fn record_state_scored(&mut self) {}
}

#[derive(Debug, Default, Clone)]
pub struct SimpleStats {
    pub prune_count: u64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub state_scored_count: u64,
}

impl Stats for SimpleStats {
    fn record_prune(&mut self) {
        self.prune_count += 1;
    }

    fn record_cache_hit(&mut self) {
        self.cache_hit_count += 1;
    }

    fn record_cache_miss(&mut self) {
        self.cache_miss_count += 1;
    }

    fn record_state_scored(&mut self) {
        self.state_scored_count += 1;
    }
}