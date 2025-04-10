// src/utils/db.rs
use anyhow::{anyhow, Context};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Decode, Pool, Postgres, Row,
};
use std::sync::Arc;

use crate::utils::eusign::{DocumentUnit, InternalPassport, TaxpayerCard};
use crate::utils::server_error::ServerError;

pub type DbPool = Pool<Postgres>;

impl<'r> sqlx::Decode<'r, Postgres> for TaxpayerCard {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let json = <sqlx::types::Json<TaxpayerCard> as sqlx::Decode<Postgres>>::decode(value)?;
        Ok(json.0)
    }
}

impl sqlx::Type<Postgres> for TaxpayerCard {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("jsonb")
    }
}

// Do the same for InternalPassport
impl<'r> sqlx::Decode<'r, Postgres> for InternalPassport {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let json = <sqlx::types::Json<InternalPassport> as sqlx::Decode<Postgres>>::decode(value)?;
        Ok(json.0)
    }
}

impl sqlx::Type<Postgres> for InternalPassport {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("jsonb")
    }
}

/// Initialize the database connection pool using the provided connection string
pub async fn init_db_pool(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
) -> Result<DbPool, ServerError> {
    PgPoolOptions::new()
        .connect_with(
            PgConnectOptions::new()
                .host(host)
                .port(port)
                .username(username)
                .database(database)
                .password(password),
        )
        .await
        .map_err(|e| anyhow!("{e}").into())
}

/// Perform initial database setup - create tables if they don't exist
pub async fn setup_db(pool: &DbPool) -> Result<(), ServerError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS document_units (
            user_id TEXT PRIMARY KEY,
            taxpayer_card JSONB NOT NULL,
            internal_passport JSONB NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create document_units table")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agreements (
            tenant_id TEXT NOT NULL,
            landlord_id TEXT NOT NULL,
            date DATE NOT NULL DEFAULT NOW(),
            PRIMARY KEY (tenant_id, landlord_id, date)
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create agreements table")?;

    // TODO: add housing_id for correct handling of different objects between same parties
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS signatures (
            tenant_id TEXT NOT NULL,
            landlord_id TEXT NOT NULL,
            tenant_signature TEXT,
            landlord_signature TEXT,
            PRIMARY KEY (tenant_id, landlord_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create signatures table")?;

    Ok(())
}

/// Store document unit in the database
pub async fn store_document_unit(
    pool: &DbPool,
    user_id: &str,
    unit: &DocumentUnit,
) -> Result<(), ServerError> {
    let taxpayer_json =
        serde_json::to_value(&unit.taxpayer_card).context("Failed to serialize taxpayer card")?;

    let passport_json = serde_json::to_value(&unit.internal_passport)
        .context("Failed to serialize internal passport")?;

    sqlx::query(
        r#"
        INSERT INTO document_units (user_id, taxpayer_card, internal_passport)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            taxpayer_card = $2,
            internal_passport = $3,
            created_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(taxpayer_json)
    .bind(passport_json)
    .execute(pool)
    .await
    .context("Failed to insert document unit")?;

    Ok(())
}

/// Retrieve document unit from the database
pub async fn get_document_unit_from_db(pool: &DbPool, user_id: &str) -> Result<Arc<DocumentUnit>, ServerError> {
    let record = sqlx::query(
        "SELECT taxpayer_card, internal_passport FROM document_units WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match record {
        Some(record) => Ok(Arc::new(DocumentUnit {
            taxpayer_card: record.try_get("taxpayer_card")?,
            internal_passport: record.try_get("internal_passport")?,
        })),
        None => Err(anyhow!("no such entry in the db").into()),
    }
}

/// Delete a document unit from the database
pub async fn delete_document_unit(pool: &DbPool, user_id: &str) -> Result<bool, ServerError> {
    // Using query_as instead of the query! macro to avoid compile-time verification issues
    let result = sqlx::query("DELETE FROM document_units WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .context("Failed to delete document unit")?;

    Ok(result.rows_affected() > 0)
}

#[derive(Serialize, Deserialize, Decode)]
pub struct Agreement {
    pub tenant_id: String,
    pub landlord_id: String,
    pub date: NaiveDate,
}

/// Create a new agreement in the database
pub async fn create_agreement(pool: &DbPool, agreement: &Agreement) -> Result<(), ServerError> {
    sqlx::query(
        r#"
        INSERT INTO agreements (
            tenant_id, 
            landlord_id, 
            date
        )
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(&agreement.tenant_id)
    .bind(&agreement.landlord_id)
    .bind(agreement.date)
    .execute(pool)
    .await
    .context("Failed to insert agreement")?;

    Ok(())
}

/// Retrieve a specific agreement from the database
pub async fn get_agreement(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
    date: &NaiveDate,
) -> Result<Option<Agreement>, ServerError> {
    let record = sqlx::query(
        r#"
        SELECT 
            tenant_id, 
            landlord_id, 
            date
        FROM agreements 
        WHERE tenant_id = $1 AND landlord_id = $2 AND date = $3
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch the agreement")?;

    Ok(record.map(|row| Agreement {
        tenant_id: row.get("tenant_id"),
        landlord_id: row.get("landlord_id"),
        date: row.get("date"),
    }))
}

/// Retrieve all agreements for a specific tenant
pub async fn get_agreements_for_tenant(
    pool: &DbPool,
    tenant_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            tenant_id, 
            landlord_id, 
            date
        FROM agreements 
        WHERE tenant_id = $1
        ORDER BY date DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch tenant agreements")?;

    let agreements = rows
        .into_iter()
        .map(|row| Agreement {
            tenant_id: row.get("tenant_id"),
            landlord_id: row.get("landlord_id"),
            date: row.get("date"),
        })
        .collect();

    Ok(agreements)
}

/// Retrieve all agreements for a specific landlord
pub async fn get_agreements_for_landlord(
    pool: &DbPool,
    landlord_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        INSERT INTO 
            tenant_id, 
            landlord_id, 
            date
        FROM agreements 
        WHERE landlord_id = $1
        ORDER BY date DESC
        "#,
    )
    .bind(landlord_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch landlord agreements")?;

    let agreements = rows
        .into_iter()
        .map(|row| Agreement {
            tenant_id: row.get("tenant_id"),
            landlord_id: row.get("landlord_id"),
            date: row.get("date"),
        })
        .collect();

    Ok(agreements)
}

pub async fn get_agreements_for_tenant_and_landlord(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            tenant_id,
            landlord_id,
            date
        FROM agreements
        WHERE tenant_id = $1
        AND landlord_id = $2
        ORDER BY date DESC
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch tenant agreements")?;

    let agreements = rows
        .into_iter()
        .map(|row| Agreement {
            tenant_id: row.get("tenant_id"),
            landlord_id: row.get("landlord_id"),
            date: row.get("date"),
        })
        .collect();

    Ok(agreements)
}

/// Delete an agreement from the database
pub async fn delete_agreement(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
    date: &NaiveDate,
) -> Result<bool, ServerError> {
    let result = sqlx::query(
        r#"
        DELETE FROM agreements 
        WHERE tenant_id = $1 AND landlord_id = $2 AND date = $3
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .bind(date)
    .execute(pool)
    .await
    .context("Failed to delete agreement")?;

    Ok(result.rows_affected() > 0)
}

pub struct SignatureEntry {
    pub tenant_id: String,
    pub landlord_id: String,
    pub tenant_signature: String,
    pub landlord_signature: String,
}

/// Create a signature entry for the specific agreement.
pub async fn create_signature_entry(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
) -> Result<bool, ServerError> {
    let result = sqlx::query(
        r#"
        INSERT INTO signatures (
            tenant_id, 
            landlord_id,
            tenant_signature,
            landlord_signature,
        )
        VALUES ($1, $2, "", "")
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .execute(pool)
    .await
    .context("Failed to create signature entry")?;

    Ok(result.rows_affected() > 0)
}

/// Add a signature to the signature entry.
pub async fn add_signature(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
    signed_by: &str,
    signature: String,
) -> Result<bool, ServerError> {
    // Figure out which column to set
    let col_to_set = if signed_by == tenant_id {
        "tenant_signature"
    } else {
        "landlord_signature"
    };

    let query = format!(
        r#"
        INSERT INTO signatures (tenant_id, landlord_id, tenant_signature, landlord_signature)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (tenant_id, landlord_id)
        DO UPDATE SET {0} = EXCLUDED.{0}
        "#,
        col_to_set
    );

    let (tenant_sig, landlord_sig) = if col_to_set == "tenant_signature" {
        (signature, "".to_string())
    } else {
        ("".to_string(), signature)
    };

    let result = sqlx::query(&query)
        .bind(tenant_id)
        .bind(landlord_id)
        .bind(tenant_sig)
        .bind(landlord_sig)
        .execute(pool)
        .await
        .context("Failed to upsert signature")?;

    // rows_affected() should be 1 for either the insert or the update.
    // If it returns 0, then no row was changed at all.
    Ok(result.rows_affected() > 0)
}

/// Remove a signature entry completely and return the deleted entry if found.
pub async fn remove_signature_entry(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
) -> Result<Option<SignatureEntry>, ServerError> {
    // First, retrieve the signature entry that's about to be deleted
    let signature_entry = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT tenant_id, landlord_id, tenant_signature, landlord_signature
        FROM signatures
        WHERE tenant_id = $1
          AND landlord_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch signature entry before removal")?;

    // Now delete the entry
    let result = sqlx::query(
        r#"
        DELETE FROM signatures
        WHERE tenant_id = $1
          AND landlord_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .execute(pool)
    .await
    .context("Failed to remove signature entry")?;

    // If we found and deleted an entry, convert it to a SignatureEntry struct
    if result.rows_affected() > 0 {
        if let Some((tenant_id, landlord_id, tenant_signature, landlord_signature)) =
            signature_entry
        {
            return Ok(Some(SignatureEntry {
                tenant_id,
                landlord_id,
                tenant_signature,
                landlord_signature,
            }));
        }
    }

    // No rows were deleted
    Ok(None)
}
