use std::num::NonZeroUsize;

use log::{debug, info};
use lru::LruCache;

use crate::request::IncomingRequest;

pub struct Cache {
    cache: LruCache<String, IncomingRequest>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let cache: LruCache<String, IncomingRequest> =
            LruCache::new(NonZeroUsize::new(capacity).unwrap());
        Cache { cache }
    }

    pub fn get(&mut self, key: String) -> Option<IncomingRequest> {
        info!("fetching value from cache for {}", key);
        debug!("Cache size {}", self.cache.len());
        self.cache.get(&key).cloned()
    }

    pub fn insert(&mut self, key: String, value: IncomingRequest) {
        info!("inserting value for {} into cache", key);
        self.cache.put(key, value);
    }

    pub fn invalidate(&mut self) {
        info!("removing all entries from cache");
        self.cache.clear();
    }
}
