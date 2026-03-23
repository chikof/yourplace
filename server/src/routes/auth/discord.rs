use actix_identity::Identity;
use actix_web::{HttpRequest, HttpResponse, cookie::Cookie, get, web};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, Scope};
use serde::Deserialize;

use database::{UserCreation, UserModel};

use crate::AppError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DiscordCallbackQuery {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct DiscordUser {
    id: String,
    username: String,
    email: Option<String>,
}

macros_utils::routes! {
    route discord_login,
    route discord_callback,
    route discord_logout,

    on "/discord"
}

#[get("/login")]
pub async fn discord_login(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let (auth_url, csrf) = data
        .oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    let cookie = Cookie::build("discord_oauth_state", csrf.secret().clone())
        .path("/")
        .http_only(true)
        .finish();

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .cookie(cookie)
        .finish())
}

#[get("/callback")]
pub async fn discord_callback(
    data: web::Data<AppState>,
    identity: Identity,
    query: web::Query<DiscordCallbackQuery>,
    request: HttpRequest,
) -> Result<HttpResponse, AppError> {
    if let Some(state_cookie) = request.cookie("discord_oauth_state") {
        if state_cookie.value() != query.state {
            return Err(AppError::AuthorizationError);
        }
    } else {
        return Err(AppError::AuthorizationError);
    }

    let token = data
        .oauth
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await?;

    let user: DiscordUser = data
        .http_client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await?
        .json()
        .await?;

    let email = user.email.clone().unwrap_or_else(|| format!("{}@discord", user.id));
    let username = user.username;

    let db_user = if let Some(existing) = UserModel::get_by_email(&email).await? {
        existing
    } else {
        UserModel::create_new(UserCreation { username, email }).await?
    };

    let user_id = db_user.id();
    identity.remember(user_id);

    Ok(HttpResponse::Ok().json(db_user.to_result()))
}

#[get("/logout")]
pub async fn discord_logout(identity: Identity) -> Result<HttpResponse, AppError> {
    identity.forget();
    Ok(HttpResponse::Ok().finish())
}
