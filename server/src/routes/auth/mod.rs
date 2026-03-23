mod discord;

macros_utils::routes! {
    load discord,

    on "/auth"
}
