use std::collections::HashMap;

use log::{debug, info};

use crate::request::IncomingRequest;

#[derive(Clone)]
pub struct Cache {
    cache: HashMap<String, IncomingRequest>,
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        let cache = HashMap::with_capacity(capacity);
        //let mut cache: LruCache<String, IncomingRequest> =
        //    LruCache::new(NonZeroUsize::new(capacity).unwrap());
        Cache { cache }
    }

    pub fn get(&self, key: String) -> Option<IncomingRequest> {
        info!("fetching value from cache for {}", key);
        debug!("Cache size {}", self.cache.len());
        self.cache.get(&key).cloned()
    }

    pub fn insert(&mut self, key: String, value: IncomingRequest) {
        info!("inserting value {:?} for {} into cache", value, key);
        self.cache.insert(key, value);
    }

    pub fn invalidate(&mut self) {
        info!("removing all entries from cache");
        self.cache.clear();
    }
}
