use actix_identity::Identity;
use actix_web::{HttpResponse, get, post, web};

use database::{CommunityCreation, CommunityMembershipResult, CommunityModel, MembershipCreation};

use crate::AppError;

macros_utils::routes! {
    route create_community,
    route list_user_communities,
    route invite_member,
    route list_members,

    on "/communities"
}

fn require_identity(identity: &Identity) -> Result<String, AppError> {
    identity.id().map_err(|_| AppError::AuthorizationError)?.ok_or(AppError::AuthorizationError)
}

#[post("")]
pub async fn create_community(
    payload: web::Json<CommunityCreation>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;

    let mut creation = payload.into_inner();
    creation.owner_id = user_id.clone();

    let community = CommunityModel::create_new(creation).await?;

    CommunityMembershipResult::create_new(MembershipCreation {
        community_id: community.id.to_string(),
        user_id,
        role: Some("owner".to_string()),
    })
    .await?;

    Ok(HttpResponse::Created().json(community.to_result()))
}

#[get("")]
pub async fn list_user_communities(identity: Identity) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let communities = CommunityModel::list_for_user(&user_id).await?;

    let response: Vec<_> = communities.into_iter().map(|c| c.to_result()).collect();
    Ok(HttpResponse::Ok().json(response))
}

#[post("/{community_id}/members")]
pub async fn invite_member(
    path: web::Path<String>,
    payload: web::Json<MembershipCreation>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let community_id = path.into_inner();

    CommunityMembershipResult::require_role(&community_id, &user_id, &["owner"]).await?;

    let mut creation = payload.into_inner();
    creation.community_id = community_id.clone();

    let member = CommunityMembershipResult::create_new(creation).await?;
    Ok(HttpResponse::Created().json(member))
}

#[get("/{community_id}/members")]
pub async fn list_members(
    path: web::Path<String>,
    identity: Identity,
) -> Result<HttpResponse, AppError> {
    let user_id = require_identity(&identity)?;
    let community_id = path.into_inner();

    CommunityMembershipResult::require_role(&community_id, &user_id, &["owner", "member"]).await?;

    let members = CommunityMembershipResult::list_for_community(&community_id).await?;
    Ok(HttpResponse::Ok().json(members))
}
