use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::*;

use super::{eusign::DocumentUnit, server_error::ServerError};

pub const CACHE_SAVE_LOCATION_DEFAULT: &str = "checkpoint/cache.json";

#[derive(Deserialize, Serialize)]
pub struct SavedChallengeCache {
    pub cache: HashMap<String, DocumentUnit>,
}

pub fn build_cache() -> Cache<String, Arc<DocumentUnit>> {
    Cache::builder().build()
}

/// A helper function used to resume the state of the server's challenge cache from the JSON file.
pub async fn populate_cache_from_file(
    cache_file_location: &str,
    cache: &Cache<String, Arc<DocumentUnit>>,
) -> Result<(), ServerError> {
    // Attempt to open the file
    let file_to_load_cache = if let Ok(file) = fs::File::open(cache_file_location).await {
        Some(file)
    } else if let Ok(file) = fs::File::open(CACHE_SAVE_LOCATION_DEFAULT).await {
        Some(file)
    } else {
        return Err(ServerError(anyhow!(
            "ERROR: UNABLE TO LOAD CACHE. FILE NOT FOUND: {} OR DEFAULT",
            cache_file_location
        )));
    };

    if let Some(mut file) = file_to_load_cache {
        // Read the file contents
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents).await {
            return Err(ServerError(anyhow!(
                "ERROR: UNABLE TO READ CACHE FILE. ERROR: {}",
                e
            )));
        }

        // Deserialize JSON content
        let saved_cache: SavedChallengeCache = match serde_json::from_str(&contents) {
            Ok(data) => data,
            Err(e) => {
                return Err(ServerError(anyhow!(
                    "ERROR: INVALID CACHE FILE FORMAT. ERROR: {}",
                    e
                )))
            }
        };

        // Populate the cache
        for (key, value) in saved_cache.cache {
            cache.insert(key, Arc::new(value)).await;
        }

        println!("CACHE IMPORTED SUCCESSFULLY",);
        Ok(())
    } else {
        Err(ServerError(anyhow!(
            "ERROR: UNABLE TO LOAD CACHE FILE.".to_string()
        )))
    }
}

pub async fn save_cache_to_a_file(
    cache_save_location: &str,
    cache: Cache<String, Arc<DocumentUnit>>,
) {
    // Ensure the directory exists
    if let Some(parent_dir) = std::path::Path::new(&cache_save_location).parent() {
        if let Err(e) = fs::create_dir_all(parent_dir).await {
            eprintln!(
                "ERROR: UNABLE TO CREATE DIRECTORY {}: {}",
                parent_dir.display(),
                e
            );
            return;
        }
    }

    let file_to_save_cache = if let Ok(file) = fs::File::create(&cache_save_location).await {
        Some(file)
    } else if let Ok(file) = fs::File::create(CACHE_SAVE_LOCATION_DEFAULT).await {
        Some(file)
    } else {
        warn!("ERROR: UNABLE TO SAVE CACHE. ALL STATE WILL BE LOST");
        None
    };

    if let Some(mut file) = file_to_save_cache {
        // Saving the cache
        let mut result = HashMap::new();
        for elem in cache.iter() {
            result.insert((*elem.0).clone(), (*elem.1).clone());
        }

        let result = SavedChallengeCache { cache: result };

        if let Ok(value) = serde_json::to_string(&result) {
            if file.write_all(value.as_bytes()).await.is_ok() {
                info!("CACHE DATA SAVED SUCCESSFULLY");
            } else {
                warn!(
                    "ERROR: UNABLE TO SAVE CACHE (CANNOT WRITE TO A FILE). ALL STATE WILL BE LOST"
                );
            }
        } else {
            info!("ERROR: UNABLE TO SAVE CACHE (CANNOT SERIALIZE). ALL STATE WILL BE LOST");
        }
    }
}
