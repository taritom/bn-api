use chrono::prelude::*;
use diesel::pg::upsert::on_constraint;
use diesel::prelude::*;
use diesel::{dsl, PgConnection};
use schema::analytics_page_views;
use utils::errors::*;
use uuid::Uuid;

#[derive(Queryable, Identifiable, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "analytics_page_views"]
pub struct PageView {
    pub id: Uuid,
    pub date: NaiveDate,
    pub hour: NaiveTime,
    pub event_id: Uuid,
    pub source: String,
    pub medium: String,
    pub term: String,
    pub content: String,
    pub platform: String,
    pub campaign: String,
    pub url: String,
    pub code: String,
    pub client_id: String,
    pub user_agent: String,
    pub ip_address: String,
    pub count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub referrer: String,
}

impl PageView {
    pub fn create(
        date: NaiveDateTime,
        event_id: Uuid,
        source: String,
        medium: String,
        term: String,
        content: String,
        platform: String,
        campaign: String,
        url: String,
        client_id: String,
        code: String,
        ip_address: String,
        user_agent: String,
        referrer: String,
    ) -> NewPageView {
        NewPageView {
            date: date.date(),
            hour: NaiveTime::from_hms(date.time().hour(), 0, 0),
            event_id,
            source,
            medium,
            term,
            content,
            platform,
            campaign,
            url,
            client_id,
            code,
            user_agent,
            ip_address,
            referrer,
            count: 1,
        }
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "analytics_page_views"]
pub struct NewPageView {
    pub date: NaiveDate,
    pub hour: NaiveTime,
    pub event_id: Uuid,
    pub source: String,
    pub medium: String,
    pub term: String,
    pub content: String,
    pub platform: String,
    pub campaign: String,
    pub url: String,
    pub code: String,
    pub client_id: String,
    pub user_agent: String,
    pub ip_address: String,
    pub count: i64,
    pub referrer: String,
}

impl NewPageView {
    pub fn commit(self, conn: &PgConnection) -> Result<PageView, DatabaseError> {
        use schema::*;

        diesel::insert_into(analytics_page_views::table)
            .values(&self)
            .on_conflict(on_constraint("analytics_page_views_unique"))
            .do_update()
            .set((
                analytics_page_views::count.eq(analytics_page_views::count + 1),
                analytics_page_views::updated_at.eq(dsl::now),
            ))
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not update/insert page view analytics")
    }
}
