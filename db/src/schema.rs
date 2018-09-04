table! {
    artists (id) {
        id -> Uuid,
        name -> Text,
        bio -> Text,
        website_url -> Nullable<Text>,
        youtube_video_urls -> Array<Text>,
        facebook_username -> Nullable<Text>,
        instagram_username -> Nullable<Text>,
        snapchat_username -> Nullable<Text>,
        soundcloud_username -> Nullable<Text>,
        bandcamp_username -> Nullable<Text>,
    }
}

table! {
    event_artists (id) {
        id -> Uuid,
        event_id -> Uuid,
        artist_id -> Uuid,
        rank -> Int4,
        set_time -> Nullable<Timestamp>,
    }
}

table! {
    event_interest (id) {
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
        venue_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        event_start -> Nullable<Timestamp>,
        door_time -> Nullable<Timestamp>,
        status -> Text,
        publish_date -> Nullable<Timestamp>,
        promo_image_url -> Nullable<Text>,
        additional_info -> Nullable<Text>,
        age_limit -> Nullable<Int4>,
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
    fee_schedule_ranges (id) {
        id -> Uuid,
        fee_schedule_id -> Uuid,
        min_price -> Int8,
        fee -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    fee_schedules (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
    }
}

table! {
    order_items (id) {
        id -> Uuid,
        order_id -> Uuid,
        item_type -> Text,
        ticket_type_id -> Uuid,
        quantity -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    orders (id) {
        id -> Uuid,
        user_id -> Uuid,
        status -> Text,
        order_type -> Text,
        created_at -> Timestamp,
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
        postal_code -> Nullable<Text>,
        phone -> Nullable<Text>,
        fee_schedule_id -> Nullable<Uuid>,
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
    price_points (id) {
        id -> Uuid,
        ticket_type_id -> Uuid,
        name -> Text,
        status -> Text,
        price_in_cents -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    regions (id) {
        id -> Uuid,
        name -> Text,
    }
}

table! {
    ticket_allocations (id) {
        id -> Uuid,
        event_id -> Uuid,
        tari_asset_id -> Nullable<Text>,
        created_at -> Timestamp,
        synced_on -> Nullable<Timestamp>,
        ticket_delta -> Int8,
    }
}

table! {
    ticket_types (id) {
        id -> Uuid,
        event_id -> Uuid,
        name -> Text,
        status -> Text,
        created_at -> Timestamp,
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
        region_id -> Nullable<Uuid>,
        name -> Text,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        phone -> Nullable<Text>,
    }
}

joinable!(event_artists -> artists (artist_id));
joinable!(event_artists -> events (event_id));
joinable!(event_interest -> events (event_id));
joinable!(event_interest -> users (user_id));
joinable!(events -> organizations (organization_id));
joinable!(events -> venues (venue_id));
joinable!(external_logins -> users (user_id));
joinable!(fee_schedule_ranges -> fee_schedules (fee_schedule_id));
joinable!(order_items -> orders (order_id));
joinable!(order_items -> ticket_types (ticket_type_id));
joinable!(orders -> users (user_id));
joinable!(organization_invites -> organizations (organization_id));
joinable!(organization_users -> organizations (organization_id));
joinable!(organization_users -> users (user_id));
joinable!(organization_venues -> organizations (organization_id));
joinable!(organization_venues -> venues (venue_id));
joinable!(organizations -> fee_schedules (fee_schedule_id));
joinable!(organizations -> users (owner_user_id));
joinable!(price_points -> ticket_types (ticket_type_id));
joinable!(ticket_allocations -> events (event_id));
joinable!(ticket_types -> events (event_id));
joinable!(venues -> regions (region_id));

allow_tables_to_appear_in_same_query!(
    artists,
    event_artists,
    event_interest,
    events,
    external_logins,
    fee_schedule_ranges,
    fee_schedules,
    order_items,
    orders,
    organization_invites,
    organizations,
    organization_users,
    organization_venues,
    price_points,
    regions,
    ticket_allocations,
    ticket_types,
    users,
    venues,
);
