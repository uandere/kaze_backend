use std::sync::Arc;

use moka::future::Cache;

use super::eusign::DocumentData;

pub fn build_cache() -> Cache<String, Arc<DocumentData>> {
    let cache = Cache::builder().build();

    cache
}
