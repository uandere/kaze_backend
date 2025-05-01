use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use chrono::Utc;
use moka::{
    future::{Cache, FutureExt},
    notification::{ListenerFuture, RemovalCause},
    Expiry,
};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::UnboundedSender,
};
use tracing::*;

use super::{
    db::{self, SignatureEntry},
    server_error::ServerError,
};

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
    pub housing_id: String,
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

pub fn build_cache(
    pool: Arc<db::DbPool>,
    sender: UnboundedSender<SignatureEntry>,
) -> AgreementProposalCache {
    let expiry = AgreedAndSignedExpiry;

    // In this function we move the entry to the permanent database.
    let eviction_listener = move |key: Arc<AgreementProposalKey>,
                                  _val: Arc<AgreementProposalValue>,
                                  cause|
          -> ListenerFuture {
        if cause != RemovalCause::Expired {
            return (async {}).boxed();
        }
        let pool = pool.clone();
        let sender = sender.clone();

        async move {
        let res = db::create_agreement(
                    &pool,
                    &db::Agreement {
                        tenant_id: key.tenant_id.clone(),
                        landlord_id: key.landlord_id.clone(),
                        date: Utc::now().date_naive(),
                        housing_id: key.housing_id.clone(),
                    },
                )
                .await;

        match res {
            Ok(_) => info!("agreement proposal with key = {:?} is removed from cache: both parties agreed and signed", key),
            Err(e) => error!("agreement proposal with key = {:?} is removed from cache, but was not added to database: {:?}", key, e),
        }

        // extracting signatures and sending them to signature handler 
        match db::remove_signature_entry(&pool, &key.tenant_id, &key.landlord_id, &key.housing_id).await {
            Ok(maybe_entry) => {
                match maybe_entry {
                    Some(entry) => {
                        if let Err(e) = sender.send(entry) {
                            error!("the signature request wasn't fulfilled: couldn't send the signature entry to signature handler: {}", e);
                        }
                    },
                    None => {
                        error!("the signature request wasn't fulfilled: no such signature entry in the db");
                    },
                }
            },
            Err(e) => {
                error!("the signature request wasn't fulfilled: couldn't get signatures from the db: {:?}", e);
            },
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
        return Err(anyhow!(
            "ERROR: UNABLE TO LOAD CACHE. FILE NOT FOUND: {} OR DEFAULT",
            cache_file_location
        )
        .into());
    };

    if let Some(mut file) = file_to_load_cache {
        // Read the file contents
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents).await {
            return Err(anyhow!("ERROR: UNABLE TO READ CACHE FILE. ERROR: {}", e).into());
        }

        // Deserialize JSON content
        let saved_cache: SavedChallengeCache = match serde_json::from_str(&contents) {
            Ok(data) => data,
            Err(e) => return Err(anyhow!("ERROR: INVALID CACHE FILE FORMAT. ERROR: {}", e).into()),
        };

        // Populate the cache
        for entry in saved_cache.cache {
            cache.insert(entry.key, Arc::new(entry.value)).await;
        }

        info!("CACHE IMPORTED SUCCESSFULLY",);
        Ok(())
    } else {
        Err(anyhow!("ERROR: UNABLE TO LOAD CACHE FILE.".to_string()).into())
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
        // saving as a vector of entries
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
