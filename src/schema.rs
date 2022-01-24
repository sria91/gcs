table! {
    use diesel::sql_types::*;

    plateaus (id) {
        id -> Text,
        created_at -> Timestamp,
        x_max -> Integer,
        y_max -> Integer,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::direction::*;

    rovers (id) {
        id -> Text,
        created_at -> Timestamp,
        x -> Integer,
        y -> Integer,
        facing -> DirectionMapping,
        plateau_id -> Text,
    }
}

joinable!(rovers -> plateaus (plateau_id));

allow_tables_to_appear_in_same_query!(plateaus, rovers,);
