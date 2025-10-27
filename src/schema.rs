// @generated automatically by Diesel CLI.

diesel::table! {
    castgc (id) {
        id -> Nullable<Integer>,
        key -> Text,
        value -> Text,
        last_access -> Timestamp,
    }
}

diesel::table! {
    key_to_school (id) {
        id -> Nullable<Integer>,
        key -> Text,
        school -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    castgc,
    key_to_school,
);
