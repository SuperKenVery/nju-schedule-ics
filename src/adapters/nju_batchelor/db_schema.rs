// @generated automatically by Diesel CLI.

diesel::table! {
    castgc (id) {
        id -> Nullable<Integer>,
        key -> Text,
        value -> Text,
        last_access -> Timestamp,
    }
}
