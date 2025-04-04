use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use chrono::Utc;
use moka::{
    future::{Cache, FutureExt},
    notification::ListenerFuture,
    Expiry,
};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::*;

use super::{db, server_error::ServerError};

pub const CACHE_SAVE_LOCATION_DEFAULT: &str = "checkpoint/cache.json";

pub type AgreementProposalCache = Cache<AgreementProposalKey, Arc<AgreementProposalValue>>;

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: AgreementProposalKey,
    pub value: AgreementProposalValue,
}

#[derive(Deserialize, Serialize)]
pub struct SavedChallengeCache {
    pub cache: Vec<CacheEntry>,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct AgreementProposalKey {
    pub tenant_id: String,
    pub landlord_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AgreementProposalValue {
    pub tenant_confirmed: bool,
    pub landlord_confirmed: bool,
    pub tenant_signed: bool,
    pub landlord_signed: bool,
}

pub struct AgreedAndSignedExpiry;

impl Expiry<AgreementProposalKey, Arc<AgreementProposalValue>> for AgreedAndSignedExpiry {
    fn expire_after_update(
        &self,
        _: &AgreementProposalKey,
        value: &Arc<AgreementProposalValue>,
        _: std::time::Instant,
        _: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        if value.landlord_confirmed
            && value.landlord_signed
            && value.tenant_confirmed
            && value.tenant_signed
        {
            Some(Duration::from_secs(0))
        } else {
            None
        }
    }
}

pub fn build_cache(pool: Arc<db::DbPool>) -> AgreementProposalCache {
    let expiry = AgreedAndSignedExpiry;

    // In this function we move the entry to the permanent database.
    let eviction_listener = move |key: Arc<AgreementProposalKey>,
                                  _val: Arc<AgreementProposalValue>,
                                  _c|
          -> ListenerFuture {
        let pool = pool.clone();

        async move {
        // Check if agreement already exists
        let existing = db::get_agreement(
            &pool,
            &key.tenant_id,
            &key.landlord_id,
            &Utc::now().date_naive(),
        )
        .await;

        let res = match existing {
            Ok(Some(_)) => {
                // Agreement already exists, no need to create again
                warn!("Agreement already exists in database, skipping insert");
                Ok(())
            }
            _ => {
                // Create new agreement
                db::create_agreement(
                    &pool,
                    &db::Agreement {
                        tenant_id: key.tenant_id.clone(),
                        landlord_id: key.landlord_id.clone(),
                        date: Utc::now().date_naive(),
                    },
                )
                .await
            }
        };

        match res {
            Ok(_) => info!("agreement proposal with key = {:?} is removed from cache: both parties agreed and signed", key),
            Err(e) => error!("agreement proposal with key = {:?} is removed from cache, but was not added to database: {:?}", key, e),
        }
    }
    .boxed()
    };

    Cache::builder()
        .expire_after(expiry)
        .async_eviction_listener(eviction_listener)
        .build()
}

/// A helper function used to resume the state of the server's cache from the JSON file.
pub async fn populate_cache_from_file(
    cache_file_location: &str,
    cache: &AgreementProposalCache,
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
        for entry in saved_cache.cache {
            cache.insert(entry.key, Arc::new(entry.value)).await;
        }

        info!("CACHE IMPORTED SUCCESSFULLY",);
        Ok(())
    } else {
        Err(ServerError(anyhow!(
            "ERROR: UNABLE TO LOAD CACHE FILE.".to_string()
        )))
    }
}

pub async fn save_cache_to_a_file(cache_save_location: &str, cache: AgreementProposalCache) {
    // Ensure the directory exists
    if let Some(parent_dir) = std::path::Path::new(&cache_save_location).parent() {
        if let Err(e) = fs::create_dir_all(parent_dir).await {
            warn!(
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
        // Create a vector of entries instead of a HashMap
        let mut entries = Vec::new();
        for elem in cache.iter() {
            entries.push(CacheEntry {
                key: (*elem.0).clone(),
                value: (*elem.1).clone(),
            });
        }

        let result = SavedChallengeCache { cache: entries };

        match serde_json::to_string(&result) {
            Ok(value) => {
                if file.write_all(value.as_bytes()).await.is_ok() {
                    info!("CACHE DATA SAVED SUCCESSFULLY");
                } else {
                    warn!(
                        "ERROR: UNABLE TO SAVE CACHE (CANNOT WRITE TO A FILE). ALL STATE WILL BE LOST"
                    );
                }
            }
            Err(e) => {
                info!(
                    "ERROR: UNABLE TO SAVE CACHE (CANNOT SERIALIZE). ALL STATE WILL BE LOST: {}",
                    e
                );
            }
        }
    }
}
