use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use diesel::sql_types;
use log::Level;
use models::*;
use schema::{artists, event_artists, events, organization_users, organizations, venues};
use std::borrow::Cow;
use std::collections::HashMap;
use time::Duration;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};
use validators::*;

#[derive(Associations, Identifiable, Queryable, AsChangeset)]
#[belongs_to(Organization)]
#[derive(Clone, QueryableByName, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Venue)]
#[table_name = "events"]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub status: String,
    pub publish_date: Option<NaiveDateTime>,
    pub redeem_date: Option<NaiveDateTime>,
    pub fee_in_cents: Option<i64>,
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
    pub top_line_info: Option<String>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
    #[column_name = "min_ticket_price_cache"]
    pub min_ticket_price: Option<i64>,
    #[column_name = "max_ticket_price_cache"]
    pub max_ticket_price: Option<i64>,
    pub video_url: Option<String>,
}

#[derive(Default, Insertable, Serialize, Deserialize, Validate)]
#[table_name = "events"]
pub struct NewEvent {
    pub name: String,
    pub organization_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    #[serde(default = "NewEvent::default_status", skip_deserializing)]
    pub status: String,
    pub publish_date: Option<NaiveDateTime>,
    pub redeem_date: Option<NaiveDateTime>,
    pub fee_in_cents: Option<i64>,
    #[validate(url(message = "Promo image URL is invalid"))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub promo_image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
    pub min_ticket_price_cache: Option<i64>,
    pub max_ticket_price_cache: Option<i64>,
    #[validate(length(
        max = "100",
        message = "Top line info must be at most 100 characters long"
    ))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub top_line_info: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    #[validate(url(message = "Video URL is invalid"))]
    pub video_url: Option<String>,
}

#[derive(AsChangeset)]
#[table_name = "events"]
pub struct EventMinMaxCache {
    pub min_ticket_price_cache: Option<i64>,
    pub max_ticket_price_cache: Option<i64>,
}

impl NewEvent {
    pub fn commit(&self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        self.validate()?;

        diesel::insert_into(events::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new event")
    }

    pub fn default_status() -> String {
        EventStatus::Draft.to_string()
    }
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "events"]
pub struct EventEditableAttributes {
    pub name: Option<String>,
    pub venue_id: Option<Uuid>,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub publish_date: Option<NaiveDateTime>,
    pub redeem_date: Option<NaiveDateTime>,
    pub fee_in_cents: Option<i64>,
    #[validate(url(message = "Promo image URL is invalid"))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub promo_image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
    pub cancelled_at: Option<NaiveDateTime>,
    #[validate(length(
        max = "100",
        message = "Top line info must be at most 100 characters long"
    ))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub top_line_info: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    #[validate(url(message = "Video URL is invalid"))]
    pub video_url: Option<String>,
}

impl Event {
    pub fn create(
        name: &str,
        status: EventStatus,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        publish_date: Option<NaiveDateTime>,
    ) -> NewEvent {
        NewEvent {
            name: name.into(),
            status: status.to_string(),
            organization_id,
            venue_id,
            event_start,
            door_time,
            publish_date,
            ..Default::default()
        }
    }

    pub fn status(&self) -> Result<EventStatus, EnumParseError> {
        self.status.parse::<EventStatus>()
    }

    pub fn update_cache(&self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        let ticket_types = TicketType::find_by_event_id(self.id, conn)?;

        let mut has_prices = false;
        let mut min_ticket_price_cache: i64 = std::i64::MAX;
        let mut max_ticket_price_cache: i64 = 0;
        for ticket_type in ticket_types {
            for ticket_pricing in ticket_type.valid_ticket_pricing(conn)? {
                has_prices = true;

                if ticket_pricing.price_in_cents < min_ticket_price_cache {
                    min_ticket_price_cache = ticket_pricing.price_in_cents.clone();
                }
                if ticket_pricing.price_in_cents > max_ticket_price_cache {
                    max_ticket_price_cache = ticket_pricing.price_in_cents.clone();
                }
            }
        }

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update event",
            diesel::update(self)
                .set((
                    EventMinMaxCache {
                        min_ticket_price_cache: if has_prices {
                            Some(min_ticket_price_cache)
                        } else {
                            Some(0)
                        },
                        max_ticket_price_cache: if has_prices {
                            Some(max_ticket_price_cache)
                        } else {
                            Some(0)
                        },
                    },
                    events::updated_at.eq(dsl::now),
                )).get_result(conn),
        )
    }

    pub fn update(
        &self,
        attributes: EventEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Event, DatabaseError> {
        attributes.validate()?;
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update event",
            diesel::update(self)
                .set((attributes, events::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn publish(self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        if self.status()? == EventStatus::Published {
            return Event::find(self.id, conn);
        }

        let mut errors = ValidationErrors::new();
        match self.venue_id {
            Some(venue_id) => {
                let venue = Venue::find(venue_id, conn)?;
                venue.validate_for_publish()?;
            }
            None => {
                let mut validation_error =
                    create_validation_error("required", "Event can't be published without a venue");
                validation_error.add_param(Cow::from("event_id"), &self.id);
                errors.add("venue_id", validation_error);
            }
        }
        if !errors.is_empty() {
            return Err(errors.into());
        }

        diesel::update(&self)
            .set((
                events::status.eq(EventStatus::Published.to_string()),
                events::publish_date.eq(dsl::now.nullable()),
                events::updated_at.eq(dsl::now),
            )).execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not publish record")?;

        Event::find(self.id, conn)
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Event, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading event",
            events::table.find(id).first::<Event>(conn),
        )
    }

    pub fn cancel(self, conn: &PgConnection) -> Result<Event, DatabaseError> {
        diesel::update(&self)
            .set(events::cancelled_at.eq(dsl::now.nullable()))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update event")
    }

    pub fn find_all_events_for_venue(
        venue_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading event via venue",
            events::table
                .filter(events::venue_id.eq(venue_id))
                .order_by(events::name)
                .load(conn),
        )
    }

    pub fn find_all_events_for_organization(
        organization_id: Uuid,
        past_or_upcoming: PastOrUpcoming,
        page: u32,
        limit: u32,
        conn: &PgConnection,
    ) -> Result<paging::Payload<EventSummaryResult>, DatabaseError> {
        #[derive(QueryableByName)]
        struct Total {
            #[sql_type = "sql_types::BigInt"]
            total: i64,
        };

        let mut total: Vec<Total> = diesel::sql_query(
            r#"
            SELECT CAST(count(*) as bigint) as total
            FROM events e
            WHERE e.organization_id = $1
            AND CASE WHEN $2 THEN e.event_start >= now() ELSE e.event_start < now() END;
        "#,
        ).bind::<sql_types::Uuid, _>(organization_id)
        .bind::<sql_types::Bool, _>(past_or_upcoming == PastOrUpcoming::Upcoming)
        .get_results(conn)
        .to_db_error(
            ErrorCode::QueryError,
            "Could not get total events for organization",
        )?;

        let mut paging = Paging::new(page, limit);
        paging.total = total.remove(0).total as u64;

        let results = Event::find_summary_data(
            Some(organization_id),
            None,
            Some(past_or_upcoming),
            page,
            limit,
            conn,
        )?;
        Ok(Payload {
            paging,
            data: results,
        })
    }

    pub fn summary(&self, conn: &PgConnection) -> Result<EventSummaryResult, DatabaseError> {
        let mut results = Event::find_summary_data(None, Some(self), None, 0, 100, conn)?;
        Ok(results.remove(0))
    }

    fn find_summary_data(
        organization_id: Option<Uuid>,
        event: Option<&Event>,
        past_or_upcoming: Option<PastOrUpcoming>,
        page: u32,
        limit: u32,
        conn: &PgConnection,
    ) -> Result<Vec<EventSummaryResult>, DatabaseError> {
        use diesel::sql_types::Nullable as N;

        let organization_id = match event {
            Some(e) => e.organization_id,
            None => organization_id
                .expect("Either organization_id or event must be used when calling this method"),
        };

        #[derive(QueryableByName)]
        struct R {
            #[sql_type = "sql_types::Uuid"]
            id: Uuid,
            #[sql_type = "sql_types::Text"]
            name: String,
            #[sql_type = "sql_types::Uuid"]
            organization_id: Uuid,
            #[sql_type = "N<sql_types::Uuid>"]
            venue_id: Option<Uuid>,
            #[sql_type = "N<sql_types::Text>"]
            venue_name: Option<String>,
            #[sql_type = "sql_types::Timestamp"]
            created_at: NaiveDateTime,
            #[sql_type = "N<sql_types::Timestamp>"]
            event_start: Option<NaiveDateTime>,
            #[sql_type = "N<sql_types::Timestamp>"]
            door_time: Option<NaiveDateTime>,
            #[sql_type = "sql_types::Text"]
            status: String,
            #[sql_type = "N<sql_types::Text>"]
            promo_image_url: Option<String>,
            #[sql_type = "N<sql_types::Text>"]
            additional_info: Option<String>,
            #[sql_type = "N<sql_types::Text>"]
            top_line_info: Option<String>,
            #[sql_type = "N<sql_types::Integer>"]
            age_limit: Option<i32>,
            #[sql_type = "N<sql_types::Timestamp>"]
            cancelled_at: Option<NaiveDateTime>,
            #[sql_type = "N<sql_types::BigInt>"]
            min_price: Option<i64>,
            #[sql_type = "N<sql_types::BigInt>"]
            max_price: Option<i64>,
            #[sql_type = "N<sql_types::Timestamp>"]
            publish_date: Option<NaiveDateTime>,
            #[sql_type = "N<sql_types::Timestamp>"]
            on_sale: Option<NaiveDateTime>,
            #[sql_type = "N<sql_types::BigInt>"]
            sales_total_in_cents: Option<i64>,
        }

        let query_events = include_str!("../queries/find_all_events_for_organization.sql");

        jlog!(Level::Debug, "Fetching summary data for event");
        let events: Vec<R> = diesel::sql_query(query_events)
            .bind::<sql_types::Uuid, _>(organization_id)
            .bind::<N<sql_types::Bool>, _>(past_or_upcoming.map(|p| p == PastOrUpcoming::Upcoming))
            .bind::<sql_types::BigInt, _>((page * limit) as i64)
            .bind::<sql_types::BigInt, _>(limit as i64)
            .bind::<N<sql_types::Uuid>, _>(event.map(|e| e.id))
            .get_results(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load events for organization",
            )?;

        let query_ticket_types =
            include_str!("../queries/find_all_events_for_organization_ticket_type.sql");;


        jlog!(Level::Debug, "Fetching summary data for ticket types");

        let ticket_types: Vec<EventSummaryResultTicketType> = diesel::sql_query(query_ticket_types)
            .bind::<sql_types::Uuid, _>(organization_id)
            .bind::<N<sql_types::Bool>, _>(past_or_upcoming.map(|p| p == PastOrUpcoming::Upcoming))
            .bind::<sql_types::Nullable<sql_types::Uuid>, _>(event.map(|e| e.id))
            .get_results(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load events' ticket types for organization",
            )?;

        let results: Vec<EventSummaryResult> = events
            .into_iter()
            .map(|r| {
                let venue = if let (Some(venue_id), Some(venue_name)) =
                    (r.venue_id.as_ref(), r.venue_name.as_ref())
                {
                    Some(VenueInfo {
                        id: *venue_id,
                        name: venue_name.to_string(),
                    })
                } else {
                    None
                };

                let event_id = r.id;
                let mut result = EventSummaryResult {
                    id: r.id,
                    name: r.name,
                    organization_id: r.organization_id,
                    venue,
                    created_at: r.created_at,
                    event_start: r.event_start,
                    door_time: r.door_time,
                    status: r.status,
                    promo_image_url: r.promo_image_url,
                    additional_info: r.additional_info,
                    top_line_info: r.top_line_info,
                    age_limit: r.age_limit.map(|i| i as u32),
                    cancelled_at: r.cancelled_at,
                    max_ticket_price: r.max_price.map(|i| i as u32),
                    min_ticket_price: r.min_price.map(|i| i as u32),
                    publish_date: r.publish_date,
                    on_sale: r.on_sale,
                    total_tickets: 0,
                    sold_unreserved: 0,
                    sold_held: 0,
                    tickets_open: 0,
                    tickets_held: 0,
                    sales_total_in_cents: r.sales_total_in_cents.unwrap_or(0) as u32,
                    ticket_types: vec![],
                };

                for ticket_type in ticket_types.iter().filter(|tt| tt.event_id == event_id) {
                    let mut ticket_type = ticket_type.clone();
                    ticket_type.sales_total_in_cents =
                        Some(ticket_type.sales_total_in_cents.unwrap_or(0));
                    result.total_tickets += ticket_type.total as u32;
                    result.sold_unreserved += ticket_type.sold_unreserved as u32;
                    result.sold_held += ticket_type.sold_held as u32;
                    result.tickets_open += ticket_type.open as u32;
                    result.tickets_held += ticket_type.held as u32;
                    result.ticket_types.push(ticket_type);
                }

                result
            }).collect();

        Ok(results)
    }

    pub fn get_sales_by_date_range(
        &self,
        start_utc: NaiveDate,
        end_utc: NaiveDate,
        conn: &PgConnection,
    ) -> Result<Vec<DayStats>, DatabaseError> {
        jlog!(
            Level::Debug,
            &format!("Fetching sales data by dates {} and {}", start_utc, end_utc)
        );

        if start_utc > end_utc {
            return Err(DatabaseError::new(
                ErrorCode::InternalError,
                Some("Sales data start date must come before end date".to_string()),
            ));
        }

        let query = r#"
                SELECT CAST(o.order_date as Date) as date,
                cast(COALESCE(sum(oi.unit_price_in_cents * oi.quantity), 0) AS bigint) as sales,
                CAST( COALESCE(SUM(CASE WHEN oi.item_type = 'Tickets' THEN oi.quantity ELSE 0 END), 0)  as BigInt) as ticket_count
                FROM order_items oi
                INNER JOIN orders o ON oi.order_id = o.id
                WHERE oi.event_id = $1
                AND o.status = 'Paid'
                AND o.order_date >= $2
                AND o.order_date <= $3
                GROUP BY CAST(o.order_date as Date)
                ORDER BY CAST(o.order_date as Date) desc
                "#;

        #[derive(QueryableByName)]
        struct R {
            #[sql_type = "sql_types::Date"]
            date: NaiveDate,
            #[sql_type = "sql_types::Nullable<sql_types::BigInt>"]
            sales: Option<i64>,
            #[sql_type = "sql_types::Nullable<sql_types::BigInt>"]
            ticket_count: Option<i64>,
        }

        let summary: Vec<R> = diesel::sql_query(query)
            .bind::<sql_types::Uuid, _>(self.id)
            .bind::<sql_types::Timestamp, NaiveDateTime>(start_utc.and_hms(0, 0, 0))
            .bind::<sql_types::Timestamp, NaiveDateTime>(end_utc.and_hms(23, 59, 59))
            .get_results(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load calculate sales for event",
            )?;

        let mut map = HashMap::<NaiveDate, R>::new();
        for s in summary {
            map.insert(s.date, s);
        }

        let mut result = vec![];
        let n = end_utc.signed_duration_since(start_utc).num_days();
        for s in 0..=n {
            let date = start_utc + Duration::days(s);

            match map.get(&date) {
                Some(map_data) => result.push(DayStats {
                    date: map_data.date,
                    revenue_in_cents: map_data.sales.unwrap_or(0),
                    ticket_sales: map_data.ticket_count.unwrap_or(0),
                }),
                None => result.push(DayStats {
                    date,
                    revenue_in_cents: 0,
                    ticket_sales: 0,
                }),
            }
        }

        Ok(result)
    }

    pub fn guest_list(
        &self,
        query: &str,
        conn: &PgConnection,
    ) -> Result<Vec<RedeemableTicket>, DatabaseError> {
        let q = include_str!("../queries/retrieve_guest_list.sql");

        diesel::sql_query(q)
            .bind::<sql_types::Uuid, _>(self.id)
            .bind::<sql_types::Text, _>(query)
            .load::<RedeemableTicket>(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load guest list")
    }

    pub fn search(
        query_filter: Option<String>,
        region_id: Option<Uuid>,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        status_filter: Option<Vec<EventStatus>>,
        user: Option<User>,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, DatabaseError> {
        let query_like = match query_filter {
            Some(n) => format!("%{}%", n),
            None => "%".to_string(),
        };
        let mut query = events::table
            .left_join(venues::table.on(events::venue_id.eq(venues::id.nullable())))
            .inner_join(organizations::table.on(organizations::id.eq(events::organization_id)))
            .left_join(
                organization_users::table
                    .on(organization_users::organization_id.eq(organizations::id)),
            ).left_join(
                event_artists::table
                    .inner_join(
                        artists::table.on(event_artists::artist_id
                            .eq(artists::id)
                            .and(artists::name.ilike(query_like.clone()))),
                    ).on(events::id.eq(event_artists::event_id)),
            ).filter(
                events::name
                    .ilike(query_like.clone())
                    .or(venues::id
                        .is_not_null()
                        .and(venues::name.ilike(query_like.clone()))).or(artists::id.is_not_null()),
            ).filter(
                events::event_start
                    .gt(start_time
                        .unwrap_or_else(|| NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0))),
            ).filter(
                events::event_start
                    .lt(end_time
                        .unwrap_or_else(|| NaiveDate::from_ymd(3970, 1, 1).and_hms(0, 0, 0))),
            ).select(events::all_columns)
            .distinct()
            .order_by(events::event_start.asc())
            .then_order_by(events::name.asc())
            .into_boxed();

        match user {
            Some(user) => {
                // Admin results include all drafts across organizations
                if !user
                    .get_global_scopes()
                    .contains(&Scopes::OrgAdmin.to_string())
                {
                    query = query.filter(
                        events::status
                            .ne(EventStatus::Draft.to_string())
                            .or(organizations::owner_user_id.eq(user.id))
                            .or(organization_users::user_id.eq(user.id)),
                    );
                }
            }
            None => {
                query = query.filter(events::status.ne(EventStatus::Draft.to_string()));
            }
        }

        if let Some(statuses) = status_filter {
            let statuses: Vec<String> = statuses
                .into_iter()
                .map(|status| status.to_string())
                .collect();
            query = query.filter(events::status.eq_any(statuses));
        }

        if let Some(region_id) = region_id {
            query = query.filter(venues::region_id.eq(region_id));
        }

        let result = query.load(conn);

        DatabaseError::wrap(ErrorCode::QueryError, "Unable to load all events", result)
    }

    pub fn add_artist(&self, artist_id: Uuid, conn: &PgConnection) -> Result<(), DatabaseError> {
        EventArtist::create(self.id, artist_id, 0, None)
            .commit(conn)
            .map(|_| ())
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        Organization::find(self.organization_id, conn)
    }

    pub fn venue(&self, conn: &PgConnection) -> Result<Option<Venue>, DatabaseError> {
        match self.venue_id {
            Some(venue_id) => {
                let venue = Venue::find(venue_id, conn);
                match venue {
                    Ok(venue) => Ok(Some(venue)),
                    Err(e) => Err(e),
                }
            }
            None => Ok(None),
        }
    }

    pub fn add_ticket_type(
        &self,
        name: String,
        description: Option<String>,
        quantity: u32,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        wallet_id: Uuid,
        increment: Option<i32>,
        limit_per_person: i32,
        conn: &PgConnection,
    ) -> Result<TicketType, DatabaseError> {
        let asset_name = format!("{}.{}", self.name, &name);
        let ticket_type = TicketType::create(
            self.id,
            name,
            description,
            start_date,
            end_date,
            increment,
            limit_per_person,
        ).commit(conn)?;
        let asset = Asset::create(ticket_type.id, asset_name).commit(conn)?;
        TicketInstance::create_multiple(asset.id, 0, quantity, wallet_id, conn)?;
        Ok(ticket_type)
    }

    pub fn ticket_types(&self, conn: &PgConnection) -> Result<Vec<TicketType>, DatabaseError> {
        TicketType::find_by_event_id(self.id, conn)
    }

    pub fn issuer_wallet(&self, conn: &PgConnection) -> Result<Wallet, DatabaseError> {
        Wallet::find_default_for_organization(self.organization_id, conn)
    }

    pub fn for_display(self, conn: &PgConnection) -> Result<DisplayEvent, DatabaseError> {
        let venue: Option<DisplayVenue> = self.venue(conn)?.and_then(|venue| Some(venue.into()));

        Ok(DisplayEvent {
            id: self.id,
            name: self.name,
            event_start: self.event_start,
            door_time: self.door_time,
            promo_image_url: self.promo_image_url,
            additional_info: self.additional_info,
            top_line_info: self.top_line_info,
            venue,
            max_ticket_price: self.max_ticket_price,
            min_ticket_price: self.min_ticket_price,
            video_url: self.video_url,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayEvent {
    pub id: Uuid,
    pub name: String,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub top_line_info: Option<String>,
    pub venue: Option<DisplayVenue>,
    pub min_ticket_price: Option<i64>,
    pub max_ticket_price: Option<i64>,
    pub video_url: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct EventSummaryResult {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
    pub venue: Option<VenueInfo>,
    pub created_at: NaiveDateTime,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub status: String,
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub top_line_info: Option<String>,
    pub age_limit: Option<u32>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub min_ticket_price: Option<u32>,
    pub max_ticket_price: Option<u32>,
    pub publish_date: Option<NaiveDateTime>,
    pub on_sale: Option<NaiveDateTime>,
    pub total_tickets: u32,
    pub sold_unreserved: u32,
    pub sold_held: u32,
    pub tickets_open: u32,
    pub tickets_held: u32,
    pub sales_total_in_cents: u32,
    pub ticket_types: Vec<EventSummaryResultTicketType>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, QueryableByName)]
pub struct EventSummaryResultTicketType {
    #[sql_type = "sql_types::Uuid"]
    pub(crate) event_id: Uuid,
    #[sql_type = "sql_types::Text"]
    pub name: String,
    #[sql_type = "sql_types::BigInt"]
    pub min_price: i64,
    #[sql_type = "sql_types::BigInt"]
    pub max_price: i64,
    #[sql_type = "sql_types::BigInt"]
    pub total: i64,
    #[sql_type = "sql_types::BigInt"]
    pub sold_unreserved: i64,
    #[sql_type = "sql_types::BigInt"]
    pub sold_held: i64,
    #[sql_type = "sql_types::BigInt"]
    pub open: i64,
    #[sql_type = "sql_types::BigInt"]
    pub held: i64,
    #[sql_type = "sql_types::Nullable<sql_types::BigInt>"]
    pub sales_total_in_cents: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DayStats {
    pub date: NaiveDate,
    pub revenue_in_cents: i64,
    pub ticket_sales: i64,
}
