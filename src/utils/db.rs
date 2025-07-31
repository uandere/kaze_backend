use anyhow::{anyhow, Context};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres, Row,
};
use uuid::Uuid;
use std::sync::Arc;
// use sqlx::types::Uuid;
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
    // enum to represent agreement state
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'agreement_state') THEN
                CREATE TYPE agreement_state AS ENUM (
                'not_initiated','initiated','rejected',
                'generated','half_signed','signed','expired'
                );
            END IF;
        END$$;
        "#,
    )
    .execute(pool)
    .await?;

    // Table for DocumentUnits
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

    // Table for Agreements
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agreements (
            tenant_id          UUID NOT NULL,
            landlord_id        UUID NOT NULL,
            housing_id         UUID NOT NULL,
            date               DATE NOT NULL DEFAULT NOW(),

            state              agreement_state NOT NULL DEFAULT 'not_initiated',
            action_by          UUID,
            half_signature     TEXT,
            tenant_signature   TEXT,
            landlord_signature TEXT,

            PRIMARY KEY (tenant_id, landlord_id, housing_id, date),

            CHECK (state <> 'initiated'    OR action_by IS NOT NULL),
            CHECK (state <> 'rejected'     OR action_by IS NOT NULL),
            CHECK (state <> 'half_signed'  OR (action_by IS NOT NULL AND half_signature IS NOT NULL)),
            CHECK (state <> 'signed'       OR (tenant_signature IS NOT NULL AND landlord_signature IS NOT NULL)),
            CHECK (state <> 'expired'      OR (tenant_signature IS NOT NULL AND landlord_signature IS NOT NULL))
        );
        "#
    ).execute(pool).await
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
pub async fn get_document_unit_from_db(
    pool: &DbPool,
    user_id: &str,
) -> Result<Arc<DocumentUnit>, ServerError> {
    let record = sqlx::query(
        r#"
        SELECT taxpayer_card, internal_passport
        FROM document_units
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match record {
        Some(row) => {
            let taxpayer_card: TaxpayerCard = row.try_get("taxpayer_card")?;
            let internal_passport: InternalPassport = row.try_get("internal_passport")?;
            Ok(Arc::new(DocumentUnit {
                taxpayer_card,
                internal_passport,
            }))
        }
        None => Err(anyhow!("no such entry in the db").into()),
    }
}

/// Delete a document unit from the database
pub async fn delete_document_unit(pool: &DbPool, user_id: Uuid) -> Result<bool, ServerError> {
    let result = sqlx::query(
        r#"
        DELETE FROM document_units
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await
    .context("Failed to delete document unit")?;

    Ok(result.rows_affected() > 0)
}

#[derive(Serialize, Deserialize)]
pub struct Agreement {
    pub tenant_id: Uuid,
    pub landlord_id: Uuid,
    pub housing_id: Uuid,
    pub date: NaiveDate,
    pub state: String,              // keep as String ⇒ maps 1‑to‑1 to enum labels
    pub action_by: Option<Uuid>,
    pub half_signature: Option<String>,
    pub tenant_signature: Option<String>,
    pub landlord_signature: Option<String>,
}

/// Create a new agreement in the database
pub async fn create_agreement(pool: &DbPool, agreement: &Agreement) -> Result<(), ServerError> {
    sqlx::query(
        r#"
        INSERT INTO agreements (
            tenant_id, 
            landlord_id, 
            housing_id, 
            date
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(&agreement.tenant_id)
    .bind(&agreement.landlord_id)
    .bind(&agreement.housing_id)
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
    housing_id: &str,
    date: &NaiveDate,
) -> Result<Option<Agreement>, ServerError> {
    let record = sqlx::query(
        r#"
        SELECT tenant_id, landlord_id, housing_id, date
        FROM agreements 
        WHERE tenant_id = $1
          AND landlord_id = $2
          AND housing_id = $3
          AND date = $4
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .bind(housing_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch the agreement")?;

    if let Some(row) = record {
        let tenant_id: Uuid = row.try_get("tenant_id")?;
        let landlord_id: Uuid = row.try_get("landlord_id")?;
        let housing_id: Uuid = row.try_get("housing_id")?;
        let date: NaiveDate = row.try_get("date")?;
        let state: String = row.try_get("state")?;
        let action_by: Option<Uuid> = row.try_get("action_by")?;
        let half_signature: Option<String> = row.try_get("half_signature")?;
        let tenant_signature: Option<String> = row.try_get("tenant_signature")?;
        let landlord_signature: Option<String> = row.try_get("landlord_signature")?;


        Ok(Some(Agreement {
            tenant_id,
            landlord_id,
            housing_id,
            date,
            state,
            action_by,
            half_signature,
            tenant_signature,
            landlord_signature,
        }))
    } else {
        Ok(None)
    }
}

/// Retrieve all agreements for a specific tenant
pub async fn get_agreements_for_tenant(
    pool: &DbPool,
    tenant_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        SELECT tenant_id, landlord_id, housing_id, date
        FROM agreements 
        WHERE tenant_id = $1
        ORDER BY date DESC
        "#,
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch tenant agreements")?;

    let mut agreements = Vec::new();
    for row in rows {
        let tenant_id: Uuid = row.try_get("tenant_id")?;
        let landlord_id: Uuid = row.try_get("landlord_id")?;
        let housing_id: Uuid = row.try_get("housing_id")?;
        let date: NaiveDate = row.try_get("date")?;
        let state: String = row.try_get("state")?;
        let action_by: Option<Uuid> = row.try_get("action_by")?;
        let half_signature: Option<String> = row.try_get("half_signature")?;
        let tenant_signature: Option<String> = row.try_get("tenant_signature")?;
        let landlord_signature: Option<String> = row.try_get("landlord_signature")?;



        agreements.push(Agreement {
            tenant_id,
            landlord_id,
            housing_id,
            date,
            state,
            action_by,
            half_signature,
            tenant_signature,
            landlord_signature,
        });
    }

    Ok(agreements)
}

/// Retrieve all agreements for a specific landlord
pub async fn get_agreements_for_landlord(
    pool: &DbPool,
    landlord_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        SELECT tenant_id, landlord_id, housing_id, date
        FROM agreements 
        WHERE landlord_id = $1
        ORDER BY date DESC
        "#,
    )
    .bind(landlord_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch landlord agreements")?;

    let mut agreements = Vec::new();
    for row in rows {
        let tenant_id: Uuid = row.try_get("tenant_id")?;
        let landlord_id: Uuid = row.try_get("landlord_id")?;
        let housing_id: Uuid = row.try_get("housing_id")?;
        let date: NaiveDate = row.try_get("date")?;
        let state: String = row.try_get("state")?;
        let action_by: Option<Uuid> = row.try_get("action_by")?;
        let half_signature: Option<String> = row.try_get("half_signature")?;
        let tenant_signature: Option<String> = row.try_get("tenant_signature")?;
        let landlord_signature: Option<String> = row.try_get("landlord_signature")?;



        agreements.push(Agreement {
            tenant_id,
            landlord_id,
            housing_id,
            date,
            state,
            action_by,
            half_signature,
            tenant_signature,
            landlord_signature,
        });
    }

    Ok(agreements)
}

pub async fn get_agreements_for_tenant_and_landlord(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
) -> Result<Vec<Agreement>, ServerError> {
    let rows = sqlx::query(
        r#"
        SELECT tenant_id, landlord_id, housing_id, date
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
    .context("Failed to fetch tenant-landlord agreements")?;

    let mut agreements = Vec::new();
    for row in rows {
        let tenant_id: Uuid = row.try_get("tenant_id")?;
        let landlord_id: Uuid = row.try_get("landlord_id")?;
        let housing_id: Uuid = row.try_get("housing_id")?;
        let date: NaiveDate = row.try_get("date")?;
        let state: String = row.try_get("state")?;
        let action_by: Option<Uuid> = row.try_get("action_by")?;
        let half_signature: Option<String> = row.try_get("half_signature")?;
        let tenant_signature: Option<String> = row.try_get("tenant_signature")?;
        let landlord_signature: Option<String> = row.try_get("landlord_signature")?;



        agreements.push(Agreement {
            tenant_id,
            landlord_id,
            housing_id,
            date,
            state,
            action_by,
            half_signature,
            tenant_signature,
            landlord_signature,
        });
    }

    Ok(agreements)
}

/// Delete an agreement from the database
pub async fn delete_agreement(
    pool: &DbPool,
    tenant_id: &str,
    landlord_id: &str,
    housing_id: &str,
    date: &NaiveDate,
) -> Result<bool, ServerError> {
    let result = sqlx::query(
        r#"
        DELETE FROM agreements 
        WHERE tenant_id = $1
          AND landlord_id = $2
          AND housing_id = $3
          AND date = $4
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .bind(housing_id)
    .bind(date)
    .execute(pool)
    .await
    .context("Failed to delete agreement")?;

    Ok(result.rows_affected() > 0)
}

/// Delete the latest agreement from the database
pub async fn delete_latest_agreement(
    pool: &DbPool,
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
) -> Result<bool, ServerError> {
    let result = sqlx::query(
        r#"
        DELETE FROM agreements
        WHERE tenant_id = $1
          AND landlord_id = $2
          AND housing_id = $3
          AND date = (
              SELECT MAX(date)
              FROM agreements
              WHERE tenant_id = $1
                AND landlord_id = $2
                AND housing_id = $3
          )
        "#,
    )
    .bind(tenant_id)
    .bind(landlord_id)
    .bind(housing_id)
    .execute(pool)
    .await
    .context("Failed to delete latest agreement")?;

    Ok(result.rows_affected() > 0)
}

pub struct SignatureEntry {
    pub tenant_id: Uuid,
    pub landlord_id: Uuid,
    pub housing_id: Uuid,
    pub tenant_signature: String,
    pub landlord_signature: String,
}

pub async fn persist_signature(
    pool: &DbPool,
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
    signed_by: Uuid,
    signature: String,
) -> Result<(), ServerError> {
    // decide which column to update and which new state to apply
    let (col, new_state) = if signed_by == tenant_id {
        ("tenant_signature", "half_signed")
    } else {
        ("landlord_signature", "signed") // second signer => fully signed
    };

    let query = format!(
        "UPDATE agreements
         SET {col} = $4,
             state = $5,
             action_by = COALESCE(action_by, $3)
         WHERE tenant_id = $1 AND landlord_id = $2 AND housing_id = $3
           AND state <> 'expired'"
    );

    sqlx::query(&query)
        .bind(tenant_id)
        .bind(landlord_id)
        .bind(housing_id)
        .bind(signature)
        .bind(new_state)
        .execute(pool)
        .await?;

    Ok(())
}
