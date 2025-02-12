use std::sync::Arc;

use moka::future::Cache;

use super::eusign::DocumentUnit;

pub fn build_cache() -> Cache<String, Arc<DocumentUnit>> {
    Cache::builder().build()
}
