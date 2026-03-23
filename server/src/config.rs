use actix_web::cookie::Key;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use std::env;

use crate::AppError;

#[derive(Clone)]
pub struct DiscordConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub redirect_url: RedirectUrl,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
}

#[derive(Clone)]
pub struct AppConfig {
    pub discord: DiscordConfig,
    pub cookie_key: Key,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let client_id = ClientId::new(env::var("DISCORD_CLIENT_ID")?);
        let client_secret = ClientSecret::new(env::var("DISCORD_CLIENT_SECRET")?);
        let redirect_url = RedirectUrl::new(env::var("DISCORD_REDIRECT_URL")?)?;

        let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())?;
        let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?;

        let cookie_key = env::var("COOKIE_SECRET")
            .ok()
            .map(|secret| Key::from(secret.as_bytes()))
            .unwrap_or_else(|| Key::generate());

        Ok(Self {
            discord: DiscordConfig {
                client_id,
                client_secret,
                redirect_url,
                auth_url,
                token_url,
            },
            cookie_key,
        })
    }

    pub fn oauth_client(&self) -> BasicClient {
        BasicClient::new(
            self.discord.client_id.clone(),
            Some(self.discord.client_secret.clone()),
            self.discord.auth_url.clone(),
            Some(self.discord.token_url.clone()),
        )
        .set_redirect_uri(self.discord.redirect_url.clone())
    }
}
