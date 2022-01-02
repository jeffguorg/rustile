table! {
    public_key (fingerprint) {
        fingerprint -> Char,
        user -> Char,
    }
}

table! {
    user (uuid) {
        uuid -> Char,
        username -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Timestamp,
    }
}

joinable!(public_key -> user (user));

allow_tables_to_appear_in_same_query!(
    public_key,
    user,
);
