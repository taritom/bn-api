table! {
    artists (id) {
        id -> Uuid,
        name -> Text,
    }
}

table! {
    event_artists (id) {
        id -> Uuid,
        event_id -> Uuid,
        artist_id -> Uuid,
        rank -> Int4,
    }
}

table! {
    event_histories (id) {
        id -> Uuid,
        event_id -> Uuid,
        order_id -> Uuid,
        user_id -> Uuid,
        protocol_reference_hash -> Varchar,
    }
}

table! {
    event_likes (id) {
        id -> Uuid,
        event_id -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    events (id) {
        id -> Uuid,
        name -> Text,
        organization_id -> Uuid,
        venue_id -> Uuid,
        created_at -> Timestamp,
        ticket_sell_date -> Timestamp,
        event_start -> Timestamp,
    }
}

table! {
    external_logins (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        site -> Text,
        access_token -> Text,
        external_user_id -> Text,
    }
}

table! {
    orders (id) {
        id -> Uuid,
        user_id -> Uuid,
        event_id -> Uuid,
    }
}

table! {
    organization_invites (id) {
        id -> Uuid,
        organization_id -> Uuid,
        inviter_id -> Uuid,
        user_email -> Text,
        create_at -> Timestamp,
        security_token -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        status_change_at -> Nullable<Timestamp>,
        accepted -> Nullable<Int2>,
    }
}

table! {
    organizations (id) {
        id -> Uuid,
        owner_user_id -> Uuid,
        name -> Text,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        country -> Nullable<Text>,
        zip -> Nullable<Text>,
        phone -> Nullable<Text>,
    }
}

table! {
    organization_users (id) {
        id -> Uuid,
        organization_id -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    organization_venues (id) {
        id -> Uuid,
        organization_id -> Uuid,
        venue_id -> Uuid,
    }
}

table! {
    users (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        hashed_pw -> Text,
        password_modified_at -> Timestamp,
        created_at -> Timestamp,
        last_used -> Nullable<Timestamp>,
        active -> Bool,
        role -> Array<Text>,
        password_reset_token -> Nullable<Uuid>,
        password_reset_requested_at -> Nullable<Timestamp>,
    }
}

table! {
    venues (id) {
        id -> Uuid,
        name -> Text,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        country -> Nullable<Text>,
        zip -> Nullable<Text>,
        phone -> Nullable<Text>,
    }
}

joinable!(event_artists -> artists (artist_id));
joinable!(event_artists -> events (event_id));
joinable!(event_histories -> events (event_id));
joinable!(event_histories -> orders (order_id));
joinable!(event_histories -> users (user_id));
joinable!(event_likes -> events (event_id));
joinable!(event_likes -> users (user_id));
joinable!(events -> organizations (organization_id));
joinable!(events -> venues (venue_id));
joinable!(external_logins -> users (user_id));
joinable!(orders -> events (event_id));
joinable!(orders -> users (user_id));
joinable!(organization_invites -> organizations (organization_id));
joinable!(organization_users -> organizations (organization_id));
joinable!(organization_users -> users (user_id));
joinable!(organization_venues -> organizations (organization_id));
joinable!(organization_venues -> venues (venue_id));
joinable!(organizations -> users (owner_user_id));

allow_tables_to_appear_in_same_query!(
    artists,
    event_artists,
    event_histories,
    event_likes,
    events,
    external_logins,
    orders,
    organization_invites,
    organizations,
    organization_users,
    organization_venues,
    users,
    venues,
);
