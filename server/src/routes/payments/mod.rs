use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};

use database::{CoinTransactionResult, PurchaseCoinsRequest};

use crate::AppError;

macros_utils::routes! {
    route purchase_coins,
    route balance,
    route transactions,

    on "/payments"
}

fn require_identity(identity: &Identity) -> Result<String, AppError> {
    identity.id().map_err(|_| AppError::AuthorizationError)?.ok_or(AppError::AuthorizationError)
}

#[post("/purchase")]
pub async fn purchase_coins(
    payload: web::Json<PurchaseCoinsRequest>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let (transaction, balance) =
        CoinTransactionResult::apply_purchase(&user_id, payload.into_inner()).await?;

    Ok(HttpResponse::Ok().json(PurchaseResponse { transaction, balance }))
}

#[get("/balance")]
pub async fn balance(identity: Identity) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let balance = CoinTransactionResult::balance_for_user(&user_id).await?;

    Ok(HttpResponse::Ok().json(BalanceResponse { coins: balance }))
}

#[get("/transactions")]
pub async fn transactions(identity: Identity) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let transactions = CoinTransactionResult::list_for_user(&user_id).await?;

    Ok(HttpResponse::Ok().json(transactions))
}

#[derive(serde::Serialize)]
pub struct PurchaseResponse {
    pub transaction: CoinTransactionResult,
    pub balance: i64,
}

#[derive(serde::Serialize)]
pub struct BalanceResponse {
    pub coins: i64,
}
