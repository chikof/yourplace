use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres, Transaction, query, query_as};
use uuid::Uuid;

use crate::db;
use crate::utils::error::{DatabaseError, ModelResult};

#[derive(FromRow, Serialize)]
pub struct CoinTransactionResult {
    pub id: String,
    pub user_id: String,
    pub amount: i64,
    pub source: String,
    pub reference: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct PurchaseCoinsRequest {
    pub coins: i32,
    pub payment_reference: Option<String>,
}

impl CoinTransactionResult {
    pub async fn apply_purchase(
        user_id: &str,
        request: PurchaseCoinsRequest,
    ) -> ModelResult<(Self, i64)> {
        if request.coins <= 0 {
            return Err(DatabaseError::InvalidInput("coins must be greater than zero"));
        }

        let mut tx = db!().begin().await?;
        let user_uuid = Uuid::parse_str(user_id)?;
        let balance = Self::increment_balance(&mut tx, &user_uuid, request.coins).await?;
        let transaction = Self::record_transaction(
            &mut tx,
            &user_uuid,
            request.coins,
            "purchase",
            request.payment_reference,
        )
        .await?;
        tx.commit().await?;
        Ok((transaction, balance))
    }

    pub async fn reward_pixels_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        user_id: &Uuid,
        reward: i32,
        reference: Option<String>,
    ) -> ModelResult<i64> {
        let balance = Self::increment_balance(tx, user_id, reward).await?;
        let _ = Self::record_transaction(tx, user_id, reward, "placement", reference).await?;
        Ok(balance)
    }

    pub async fn list_for_user(user_id: &str) -> ModelResult<Vec<Self>> {
        let transactions = query_as!(
            Self,
            r#"
                SELECT id::text, user_id::text, amount, source, reference, created_at
                FROM coin_transactions
                WHERE user_id = $1
                ORDER BY created_at DESC
            "#,
            Uuid::parse_str(user_id)?
        )
        .fetch_all(db!())
        .await?;

        Ok(transactions)
    }

    pub async fn balance_for_user(user_id: &str) -> ModelResult<i64> {
        let record = query!(
            r#"
                SELECT coins
                FROM users
                WHERE id = $1
            "#,
            Uuid::parse_str(user_id)?
        )
        .fetch_one(db!())
        .await?;

        Ok(record.coins)
    }

    async fn increment_balance(
        tx: &mut Transaction<'_, Postgres>,
        user_id: &Uuid,
        delta: i32,
    ) -> ModelResult<i64> {
        let record = query!(
            r#"
                UPDATE users
                SET coins = coins + $1
                WHERE id = $2
                RETURNING coins
            "#,
            delta as i64,
            user_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(record.coins)
    }

    async fn record_transaction(
        tx: &mut Transaction<'_, Postgres>,
        user_id: &Uuid,
        amount: i32,
        source: &str,
        reference: Option<String>,
    ) -> ModelResult<Self> {
        let transaction = query_as!(
            Self,
            r#"
                INSERT INTO coin_transactions (
                    user_id,
                    amount,
                    source,
                    reference
                )
                VALUES ($1, $2, $3, $4)
                RETURNING id::text, user_id::text, amount, source, reference, created_at
            "#,
            user_id,
            amount as i64,
            source,
            reference
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(transaction)
    }
}
