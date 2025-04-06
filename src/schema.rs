diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    notes (id) {
        id -> Text,
        user_id -> Text,
        title -> Text,
        content -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(notes -> users (user_id));
diesel::allow_tables_to_appear_in_same_query!(users, notes);