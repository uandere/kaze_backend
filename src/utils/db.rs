// src/utils/db.rs
use anyhow::{anyhow, Context};
use serde_json;
use sqlx::postgres::PgConnectOptions;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;

use crate::utils::eusign::{DocumentUnit, InternalPassport, TaxpayerCard};
use crate::utils::server_error::ServerError;

pub type DbPool = Pool<Postgres>;

impl<'r> sqlx::Decode<'r, Postgres> for TaxpayerCard {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        // Assuming TaxpayerCard is stored as JSON
        let json = <sqlx::types::Json<TaxpayerCard> as sqlx::Decode<Postgres>>::decode(value)?;
        Ok(json.0)

        // Or if stored as binary:
        // let bytes = <Vec<u8> as Decode<Postgres>>::decode(value)?;
        // serde_json::from_slice(&bytes).map_err(Into::into)
    }
}

impl sqlx::Type<Postgres> for TaxpayerCard {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // Use JSONB type if stored as JSON
        sqlx::postgres::PgTypeInfo::with_name("jsonb")

        // Or if stored as binary:
        // sqlx::postgres::PgTypeInfo::with_name("bytea")
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
pub async fn get_document_unit_from_db(pool: &DbPool, user_id: &str) -> Option<Arc<DocumentUnit>> {
    let record = sqlx::query(
        "SELECT taxpayer_card, internal_passport FROM document_units WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    record.map(|record| {
        Arc::new(DocumentUnit {
            taxpayer_card: record.get("taxpayer_card"),
            internal_passport: record.get("internal_passport"),
        })
    })
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
