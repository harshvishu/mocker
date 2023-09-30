use std::num::NonZeroUsize;

use log::{debug, info};
use lru::LruCache;

use crate::request::RouteConfiguration;

pub struct Cache {
    cache: LruCache<String, RouteConfiguration>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let cache: LruCache<String, RouteConfiguration> =
            LruCache::new(NonZeroUsize::new(capacity).unwrap());
        Cache { cache }
    }

    pub fn get(&mut self, key: String) -> Option<RouteConfiguration> {
        info!("fetching value from cache for {}", key);
        debug!("Cache size {}", self.cache.len());
        self.cache.get(&key).cloned()
    }

    pub fn insert(&mut self, key: String, value: RouteConfiguration) {
        info!("inserting value for {} into cache", key);
        self.cache.put(key, value);
    }

    pub fn invalidate(&mut self) {
        info!("removing all entries from cache");
        self.cache.clear();
    }
}
