mod auth;
mod canvas;
mod community;
mod hello;
mod payments;

macros_utils::routes! {
    load auth,
    load community,
    load canvas,
    load payments,
    load hello,

    on "/api"
}
