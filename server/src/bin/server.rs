use actix_identity::IdentityMiddleware;
use actix_session::{
    SessionMiddleware, config::CookieContentSecurity, storage::CookieSessionStore,
};
use actix_web::cookie::Key;
use actix_web::web::get;
use actix_web::{App, HttpResponse, HttpServer, web};
use flexi_logger::Logger;
use logger::format_log;
use server::{AppError, config::AppConfig, routes, state::AppState};

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    Logger::try_with_str("info")? //
        .format(format_log)
        .log_to_stdout()
        .start()?;

    let config = AppConfig::from_env()?;
    let oauth_client = config.oauth_client();
    let cookie_key = config.cookie_key.clone();

    HttpServer::new(move || {
        let state = AppState::new(oauth_client.clone());
        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), cookie_key.clone())
                .cookie_http_only(true)
                .cookie_secure(false)
                .cookie_content_security(CookieContentSecurity::Private)
                .build();

        App::new() //
            .app_data(web::Data::new(state.clone()))
            .wrap(IdentityMiddleware::builder().build())
            .wrap(session_middleware)
            .route("/", get().to(HttpResponse::Ok))
            .configure(routes::routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
