use std::collections::VecDeque;

use rustc_hash::FxHashMap;

/// for caching memory locations with an id
pub struct MemoryCache {
    /// The key is the id for the specific cache, the value is the Memory address for the id
    map: FxHashMap<usize, usize>,
    id_queue: VecDeque<usize>,
}

impl MemoryCache {
    #[inline(always)]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: FxHashMap::default(),
            id_queue: VecDeque::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn add_cache(&mut self, id: usize, address: usize) {
        // If queue has reached capacity remove the earliest added cache
        if self.id_queue.len() == self.id_queue.capacity() {
            let discarded_id = self.id_queue.pop_front();
            if let Some(discarded_id) = discarded_id {
                self.map.remove(&discarded_id);
            }
        }

        self.id_queue.push_back(id);
        self.map.insert(id, address);
    }

    #[inline(always)]
    pub fn get_cache(&self, id: &usize) -> Option<&usize> {
        self.map.get(id)
    }
}

impl Default for MemoryCache {
    /// Creates a cache with 16 capacity
    fn default() -> Self {
        Self::new(16)
    }
}
