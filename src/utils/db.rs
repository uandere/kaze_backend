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
        SELECT 
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
