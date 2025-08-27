use actix_web::{HttpResponse, Responder, get};

macros_utils::routes! {
    route route_hello_world,
}

#[get("")]
pub async fn route_hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}
