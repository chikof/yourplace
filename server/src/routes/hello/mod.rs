mod world;

macros_utils::routes! {
    load world,

    on "/hello"
}
