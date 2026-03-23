use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};

use database::{CommunityMembershipResult, CompressedPixel, PixelCreation, PixelModel};

use crate::AppError;

macros_utils::routes! {
    route place_pixels,
    route fetch_canvas,

    on "/canvas"
}

fn require_identity(identity: &Identity) -> Result<String, AppError> {
    identity.id().map_err(|_| AppError::AuthorizationError)?.ok_or(AppError::AuthorizationError)
}

#[post("/{community_id}")]
pub async fn place_pixels(
    path: web::Path<String>,
    payload: web::Json<PixelCreation>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let community_id = path.into_inner();

    CommunityMembershipResult::require_role(&community_id, &user_id, &["owner", "member"]).await?;

    let mut creation = payload.into_inner();
    creation.user_id = user_id;
    creation.community_id = Some(community_id.clone());

    let pixels = PixelModel::create_new(creation).await?;
    Ok(HttpResponse::Ok().json(pixels))
}

#[get("/{community_id}")]
pub async fn fetch_canvas(
    path: web::Path<String>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let community_id = path.into_inner();
    let user_id = require_identity(&identity)?;

    CommunityMembershipResult::require_role(&community_id, &user_id, &["owner", "member"]).await?;

    let pixels = PixelModel::list_for_community(&community_id).await?;
    let compressed: Vec<CompressedPixel> = PixelModel::compress(&pixels);
    Ok(HttpResponse::Ok().json(compressed))
}
